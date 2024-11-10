use std::{cmp::min, collections::{BinaryHeap, HashMap, HashSet}, rc::Rc, str::FromStr};

use anyhow::{self, Context};
use regex_static::{lazy_regex, Regex, once_cell::sync::Lazy};
use xmas::display_result;
use yield_return::Yield;

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
}

impl ValveSystem {
    pub fn try_new(valves: impl Iterator<Item = Valve>) -> Result<Self, anyhow::Error> {
        let valves_map: HashMap<String, Valve> = valves
            .map(|v| (v.id.clone(), v))
            .collect();
        if !valves_map.contains_key(START_ID) {
            Err(anyhow::anyhow!("No valve with id {} found", START_ID))?;
        }

        let mut system = ValveSystem { valves: valves_map };
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

    pub fn calculate_greatest_amount_of_pressure(&self, time_limit: usize) -> usize {
        let mut max_pressure = 0;

        let mut closed_list = HashSet::new();
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
            if closed_list.contains(&candidate) {
                continue;
            }
            closed_list.insert(candidate.clone());

            if candidate.final_pressure > max_pressure {
                max_pressure = candidate.final_pressure;
                println!("Found candidate pressure: {}", max_pressure);
            }

            open_list.extend(
                self.find_candidates(candidate)
                    .filter(|b| !closed_list.contains(b))
                    .map(ValveBreadcrumbPriority),
            );
        }

        max_pressure
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
        let mut max_pressure = 0;

        let mut closed_list = HashSet::new();
        let mut open_list = BinaryHeap::<ElephantBreadcrumbPriority>::new();
        open_list.push(ElephantBreadcrumb {
            player: Default::default(),
            elephant: Default::default(),
            final_pressure: 0,
            time_left: time_limit,
            previous: None,
        }.into());

        while !open_list.is_empty() {
            let candidate = Rc::new(open_list.pop().unwrap().0);
            if closed_list.contains(&candidate) {
                continue;
            }
            closed_list.insert(candidate.clone());

            if candidate.final_pressure > max_pressure {
                max_pressure = candidate.final_pressure;
                println!("Found candidate pressure: {}", max_pressure);

                // let path = candidate.to_path();
                // println!("{:#?}", path);
            }

            open_list.extend(
                self.find_candidates_e(candidate)
                    .filter(|b| !closed_list.contains(b))
                    .map(ElephantBreadcrumbPriority),
            );

            // println!("debug break!");
            // loop {}
        }

        max_pressure
    }

    fn find_candidates_e<'a>(&'a self, cur: Rc<ElephantBreadcrumb<'a>>) -> impl Iterator<Item = ElephantBreadcrumb<'a>> + '_ {
        let time_left = cur.time_left;

        if cur.player.time_to_arrive > 0 && cur.elephant.time_to_arrive > 0 {
            unreachable!();
        }

        let previous = cur.clone();
        let new_breadcrumbs = Yield::new(|mut y| async move {
            if previous.player.time_to_arrive > 0 {
                let mut new_bc = previous.as_ref().clone();
                new_bc.previous = Some(previous);
                y.ret(new_bc).await;
                return;
            }

            let player_valve = self.valves.get(previous.player.to).unwrap();
            for (id, cost) in player_valve.path_costs
                .iter()
                .map(|(id, cost)| (id, cost + 1))
                .filter(move |(_, cost)| *cost < time_left)
            {
                if previous.has_visited(id) {
                    continue;
                }

                let mut new_bc = previous.as_ref().clone();
                let from = &previous.player.to;
                new_bc.previous = Some(previous.clone());
                new_bc.player = UserPath {
                    from,
                    to: &id,
                    time_to_arrive: cost,
                };
                let target_valve = self.valves.get(id).unwrap();
                let add_pressure = target_valve.rate * (time_left - cost);
                new_bc.final_pressure += add_pressure;
                y.ret(new_bc).await;
            }
        });

        let previous = cur.clone();
        Yield::new(|mut y| async move {
            if previous.elephant.time_to_arrive > 0 {
                for mut bc in new_breadcrumbs {
                    bc.substract_next_time_step();
                    // println!("Planned route:");
                    // println!("{:#?}", bc.to_path());
                    y.ret(bc).await;
                }
                return;
            }

            let elephant_valve = self.valves.get(previous.elephant.to).unwrap();
            for bc in new_breadcrumbs {
                for (to_id, cost) in elephant_valve.path_costs
                    .iter()
                    .map(|(id, cost)| (id, cost + 1))
                    .filter(move |(_, cost)| *cost < time_left)
                {
                    if bc.player.to == to_id || previous.has_visited(&to_id) {
                        continue;
                    }

                    let target_valve = self.valves.get(to_id).unwrap();
                    let mut new_bc = bc.clone();
                    let add_pressure = target_valve.rate * (time_left - cost);
                    new_bc.final_pressure += add_pressure;
                    
                    new_bc.elephant.from = previous.elephant.to;
                    new_bc.elephant.to = to_id;
                    new_bc.elephant.time_to_arrive = cost;

                    new_bc.substract_next_time_step();
                    // println!("Planned route:");
                    // println!("{:#?}", new_bc.to_path());
                    y.ret(new_bc).await;
                }
            }
        })
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ElephantBreadcrumb<'a> {
    player: UserPath<'a>,
    elephant: UserPath<'a>,
    time_left: usize,
    final_pressure: usize,
    previous: Option<Rc<Self>>,
}

impl<'a> ElephantBreadcrumb<'a> {
    pub fn has_visited(&self, id: &str) -> bool {
        let mut cur = self;
        loop {
            if cur.player.to == id || cur.elephant.to == id {
                return true;
            }

            match &cur.previous {
                Some(previous) => cur = previous.as_ref(),
                None => return false,
            }
        }
    }
    
    fn next_time_step(&self) -> usize {
        min(self.player.time_to_arrive, self.elephant.time_to_arrive)
    }

    fn substract_next_time_step(&mut self) {
        let next_time_step = self.next_time_step();
        // println!("Next time step in {}", next_time_step);
        self.player.time_to_arrive -= next_time_step;
        self.elephant.time_to_arrive -= next_time_step;
        self.time_left -= next_time_step;
    }

    pub fn to_path(&self) -> Vec<Self> {
        let mut path = vec![];
        let mut cur = self;
        loop {
            let mut new_node = cur.clone();
            new_node.previous = None;
            path.push(new_node);

            match &cur.previous {
                Some(previous) => cur = previous,
                None => break,
            }
        }

        path.reverse();
        path
    }
}

#[derive(Debug, Clone)]
struct ElephantBreadcrumbPriority<'a>(ElephantBreadcrumb<'a>);

impl<'a> From<ElephantBreadcrumb<'a>> for ElephantBreadcrumbPriority<'a> {
    fn from(value: ElephantBreadcrumb<'a>) -> Self {
        Self(value)
    }
}

impl<'a> PartialEq for ElephantBreadcrumbPriority<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.0.final_pressure == other.0.final_pressure
    }
}

impl<'a> Eq for ElephantBreadcrumbPriority<'a> {
}

impl<'a> PartialOrd for ElephantBreadcrumbPriority<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Ord for ElephantBreadcrumbPriority<'a> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.final_pressure.cmp(&other.0.final_pressure)
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
    let result = system.calculate_greatest_amount_of_pressure(30);

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
