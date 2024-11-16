use std::{cmp::Reverse, collections::{BinaryHeap, HashMap, HashSet}, rc::Rc, str::FromStr};

use anyhow::{self, Context};
use regex_static::{lazy_regex, Regex, once_cell::sync::Lazy};
use xmas::display_result;

static VALVE_REGEX: Lazy<Regex> = lazy_regex!(r"Valve (\S+) .*rate=(\d+).*valves?(.*)");

type ValveId = String;
const START_ID: &str = "AA";

#[derive(Debug, Clone)]
struct Valve {
    id: ValveId,
    rate: usize,
    leads_to: Vec<ValveId>,
    path_costs: HashMap<ValveId, usize>,
}

impl FromStr for Valve {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re_match = VALVE_REGEX.captures(s)
            .with_context(|| format!("Invalid valve info format: {}", s))?;
        let id = re_match.get(1).unwrap().as_str().to_string();
        let rate = re_match.get(2).unwrap().as_str().parse::<usize>()?;
        let leads_to = re_match
            .get(3).unwrap().as_str()
            .split(',')
            .map(|s| s.trim().to_string())
            .collect::<Vec<_>>();
        Ok(Valve { id, rate, leads_to, path_costs: Default::default() })
    }
}

#[derive(Debug, Clone)]
struct ValveSystem {
    valves: HashMap<ValveId, Valve>,
    ordered_valves: Vec<ValveId>,
}

impl ValveSystem {
    pub fn try_new(valves: impl Iterator<Item = Valve>) -> Result<Self, anyhow::Error> {
        let valves_map: HashMap<String, Valve> = valves
            .map(|v| (v.id.clone(), v))
            .collect();
        if !valves_map.contains_key(START_ID) {
            Err(anyhow::anyhow!("No valve with id {} found", START_ID))?;
        }

        let mut ordered_valves = valves_map.keys().cloned().collect::<Vec<_>>();
        ordered_valves.sort_by_cached_key(|v| Reverse(valves_map[v].rate));

        let mut system = ValveSystem {
            valves: valves_map,
            ordered_valves,
        };
        system.recalculate_paths();
        // println!("Created valve system:");
        // println!("{:#?}", system);
        Ok(system)
    }

    fn recalculate_paths(&mut self) {
        let mut costs = Vec::new();
        let valid_targets = self.valves
            .values()
            .filter(|v| v.rate > 0)
            .map(|v| v.id.as_str())
            .collect::<Vec<_>>();
        for from in self.valves.values() {
            let path_costs: HashMap<_, _> = valid_targets
                .iter()
                .map(|to| self.calculate_path_cost(&from.id, to).map(|cost| (to.to_string(), cost)).unwrap())
                .collect();
            costs.push((from.id.clone(), path_costs));
        }

        for (from, path_costs) in costs {
            let valve = self.valves.get_mut(&from).unwrap();
            valve.path_costs = path_costs;
        }
    }

    fn calculate_path_cost(&self, from: &str, to: &str) -> Option<usize> {
        // println!("Calculating path {from} -> {to}");

        let mut closed_list = HashSet::new();
        // This probably could've been a Queue
        let mut open_list = BinaryHeap::<PathBreadcrumbPriority>::new();
        open_list.push(PathBreadcrumbPriority(PathBreadcrumb { from: "", to: from, cost: 0 }));
        
        while !open_list.is_empty() {
            let candidate = open_list.pop().unwrap().0;
            if candidate.to == to {
                return Some(candidate.cost);
            }

            if closed_list.contains(&(candidate.from, candidate.to)) {
                continue;
            }
            closed_list.insert((candidate.from, candidate.to));

            let valve = self.valves.get(candidate.to).unwrap();
            let next_candidates = valve.leads_to.iter()
                .filter(|s| !closed_list.contains(&(candidate.to, s.as_str())))
                .map(|s| PathBreadcrumb {
                    from: candidate.to,
                    to: s.as_str(),
                    cost: candidate.cost + 1,
                })
                .map(PathBreadcrumbPriority);
            open_list.extend(next_candidates);

            // println!("{} candidates in queue...", open_list.len())
        }
        None
    }

    pub fn calculate_greatest_amount_of_pressure(&self, time_limit: usize) -> (usize, Vec<Rc<ValveBreadcrumb>>) {
        let mut max_pressure = 0;

        let mut found_nodes = Vec::new();
        let mut open_list = BinaryHeap::<ValveBreadcrumbPriority>::new();
        open_list.push(ValveBreadcrumb {
            from: "",
            to: START_ID,
            final_pressure: 0,
            time_left: time_limit,
            previous: None,
        }.into());

        while !open_list.is_empty() {
            let candidate = Rc::new(open_list.pop().unwrap().0);
            found_nodes.push(candidate.clone());

            if candidate.final_pressure > max_pressure {
                max_pressure = candidate.final_pressure;
                // println!("Found candidate pressure: {}", max_pressure);
            }

            open_list.extend(
                self.find_candidates(candidate).map(ValveBreadcrumbPriority),
            );
        }

        (max_pressure, found_nodes)
    }

    fn find_candidates<'a>(&'a self, cur: Rc<ValveBreadcrumb<'a>>) -> impl Iterator<Item = ValveBreadcrumb<'a>> + '_ {
        let from = cur.to;
        let valve = self.valves.get(from).unwrap();
        let time_left = cur.time_left;
        let final_pressure = cur.final_pressure;

        valve.path_costs
            .iter()
            .filter(move |(_, cost)| (*cost + 1) < time_left)
            .map(move |(id, cost)| {
                // println!("{} -> {} will cost {} for time left {}", from, id, cost, time_left);
                (self.valves.get(id).unwrap(), cost + 1, cur.clone())
            })
            .filter(|(id, _, previous)| !previous.has_visited(&id.id))
            .map(move |(target_valve, cost, previous)| {
                let new_time_left = time_left - cost;
                let add_pressure = target_valve.rate * new_time_left;
                ValveBreadcrumb {
                    from,
                    to: &target_valve.id,
                    time_left: new_time_left,
                    final_pressure: final_pressure + add_pressure,
                    previous: Some(previous),
                }
            })
    }

    pub fn calculate_greatest_pressure_with_elephant(&self, time_limit: usize) -> usize {
        let (_, paths) = self.calculate_greatest_amount_of_pressure(time_limit);
        println!("Testing combinations of {} path/s", paths.len());

        let mut final_paths = paths.iter().collect::<Vec<_>>();
        final_paths.sort_by_key(|bc| Reverse(bc.final_pressure));

        let mut max_pressure = 0;
        for (i, &path) in final_paths.iter().enumerate() {
            let used_nodes = path.traceback_iter()
                .map(|bc| bc.to)
                .filter(|&id| id != START_ID)
                .collect::<HashSet<_>>();
            let user_pressure = path.final_pressure;
            
            let elephant_paths = final_paths[i..]
                .iter()
                .filter(|e_path| e_path.traceback_iter().all(|node| !used_nodes.contains(node.to)));
            for &e_path in elephant_paths {
                let total_pressure = user_pressure + e_path.final_pressure;
                if total_pressure > max_pressure {
                    max_pressure = total_pressure;
                    println!("Found candidate pressure: {} ({} + {})", max_pressure, user_pressure, e_path.final_pressure);
                }
                break;
            }
        }
        max_pressure
    }
}

impl FromStr for ValveSystem {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let valves: Vec<Valve> = s.lines()
            .map(Valve::from_str)
            .collect::<Result<_, _>>()?;
        
        Self::try_new(valves.into_iter())
    }
}

#[derive(Debug, Clone)]
struct PathBreadcrumb<'a> {
    from: &'a str,
    to: &'a str,
    cost: usize,
}

#[derive(Debug, Clone)]
struct PathBreadcrumbPriority<'a>(PathBreadcrumb<'a>);

impl<'a> PartialEq for PathBreadcrumbPriority<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.0.cost == other.0.cost
    }
}

impl<'a> Eq for PathBreadcrumbPriority<'a> {
}

impl<'a> PartialOrd for PathBreadcrumbPriority<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Ord for PathBreadcrumbPriority<'a> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cost.cmp(&other.0.cost).reverse()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ValveBreadcrumb<'a> {
    from: &'a str,
    to: &'a str,
    time_left: usize,
    final_pressure: usize,
    previous: Option<Rc<Self>>,
}

impl<'a> ValveBreadcrumb<'a> {
    pub fn has_visited(&self, id: &str) -> bool {
        let mut cur = self;
        loop {
            if cur.to == id {
                return true;
            }

            match &cur.previous {
                Some(previous) => cur = previous.as_ref(),
                None => return false,
            }
        }
    }

    pub fn traceback_iter(&self) -> impl Iterator<Item = &Self> + '_ {
        let mut cur = Some(self);
        std::iter::from_fn(move || {
            let item = cur;
            if let Some(_item) = item {
                cur = _item.previous.as_deref();
            }
            item
        })
    }
}

#[derive(Debug, Clone)]
struct ValveBreadcrumbPriority<'a>(ValveBreadcrumb<'a>);

impl<'a> From<ValveBreadcrumb<'a>> for ValveBreadcrumbPriority<'a> {
    fn from(value: ValveBreadcrumb<'a>) -> Self {
        Self(value)
    }
}

impl<'a> PartialEq for ValveBreadcrumbPriority<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.0.final_pressure == other.0.final_pressure
    }
}

impl<'a> Eq for ValveBreadcrumbPriority<'a> {
}

impl<'a> PartialOrd for ValveBreadcrumbPriority<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Ord for ValveBreadcrumbPriority<'a> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.final_pressure.cmp(&other.0.final_pressure)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct UserPath<'a> {
    from: &'a str,
    to: &'a str,
    time_to_arrive: usize,
}

impl<'a> Default for UserPath<'a> {
    fn default() -> Self {
        Self { from: "", to: START_ID, time_to_arrive: 0 }
    }
}

fn main() -> anyhow::Result<()> {
    part_1()?;
    println!();
    part_2()?;
    Ok(())
}

fn part_1() -> anyhow::Result<()> {
    println!("Part 1:");
    let input = std::fs::read_to_string("./input.txt").context("Error reading input file.")?;

    let system = ValveSystem::from_str(&input)?;
    let (result, _) = system.calculate_greatest_amount_of_pressure(30);

    display_result(&result);
    Ok(())
}

fn part_2() -> anyhow::Result<()> {
    println!("Part 2:");
    let input = std::fs::read_to_string("./input.txt").context("Error reading input file.")?;

    let system = ValveSystem::from_str(&input)?;
    let result = system.calculate_greatest_pressure_with_elephant(26);

    display_result(&result);
    Ok(())
}
