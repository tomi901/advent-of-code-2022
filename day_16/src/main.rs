use std::{cmp::Reverse, collections::{BinaryHeap, HashMap, HashSet}, ops::Not, rc::Rc, str::FromStr};

use anyhow::{self, Context};
use regex_static::{lazy_regex, Regex, once_cell::sync::Lazy};
use xmas::display_result;

static VALVE_REGEX: Lazy<Regex> = lazy_regex!(r"Valve (\S+) .*rate=(\d+).*valves?(.*)");

type ValveId = String;
const START_ID: &str = "AA";

struct Valve {
    id: ValveId,
    rate: usize,
    leads_to: Vec<ValveId>,
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
        Ok(Valve { id, rate, leads_to })
    }
}

struct ValveSystem {
    valves: HashMap<ValveId, Valve>,
    ordered_valve_rates: Vec<(ValveId, usize)>,
}

impl ValveSystem {
    pub fn calculate_greatest_amount_of_pressure(&self, time_limit: usize) -> usize {
        let mut max_pressure = 0;

        let mut open_list = BinaryHeap::<BreadcrumbPriority>::new();
        let mut closed_list = HashSet::new();
        open_list.push(Breadcrumb::start(time_limit).into());
        while !open_list.is_empty() {
            let candidate = Rc::new(open_list.pop().unwrap().0);
            closed_list.insert(candidate.as_ref().clone());

            if candidate.optimistic_final_pressure <= max_pressure {
                continue;
            }

            // println!("Checking {} at minute {}...", candidate.valve, candidate.time);
            if candidate.final_pressure > max_pressure {
                max_pressure = candidate.final_pressure;
                // With the real input it won't finish but after some minutes the answer is shown here:
                println!("Found candidate pressure: {}", max_pressure);
            }

            if candidate.time_left > 0 {
                open_list.extend(
                    self.find_candidates(candidate)
                        .map(|b| self.recalculate_optimistic_final_pressure(b))
                        .filter(|b| !closed_list.contains(b))
                        .map(BreadcrumbPriority),
                );
            }
        }

        max_pressure
    }

    fn recalculate_optimistic_final_pressure(&self, breadcrumb: Breadcrumb) -> Breadcrumb {
        let mut time_left = breadcrumb.time_left;
        if time_left <= 1 {
            return Breadcrumb {
                optimistic_final_pressure: breadcrumb.final_pressure,
                ..breadcrumb
            };
        }

        let opened_valves = breadcrumb.trace_open_valves().collect::<HashSet<_>>();
        let try_valves = self.ordered_valve_rates
            .iter()
            .filter(|(v, _)| !opened_valves.contains(v));
        time_left -= 1;

        let mut optimistic_bonus = 0;
        for (_, rate) in try_valves {
            optimistic_bonus += time_left * rate;
            if time_left < 3 {
                break;
            }
            time_left -= 2;
        }
        // println!("optimistic_bonus: {optimistic_bonus}");

        Breadcrumb {
            optimistic_final_pressure: breadcrumb.final_pressure + optimistic_bonus,
            ..breadcrumb
        }
    }

    fn find_candidates(&self, cur: Rc<Breadcrumb>) -> impl Iterator<Item = Breadcrumb> + '_ {
        let valve = self.valves.get(&cur.valve).unwrap();
        let new_time_left = cur.time_left - 1;
        let open_action = cur.has_opened_cur()
            .not()
            .then(|| Breadcrumb {
                valve: cur.valve.clone(),
                time_left: new_time_left,
                action: Action::Open,
                final_pressure: cur.final_pressure + (new_time_left * valve.rate),
                optimistic_final_pressure: 0,
                previous: Some(cur.clone()),
            });

        let move_actions = valve.leads_to
            .iter()
            .map(move |id| Breadcrumb {
                valve: id.clone(),
                time_left: new_time_left,
                action: Action::Move,
                final_pressure: cur.final_pressure,
                optimistic_final_pressure: 0,
                previous: Some(cur.clone()),
            });

        open_action.into_iter().chain(move_actions)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Breadcrumb {
    valve: ValveId,
    time_left: usize,
    action: Action,
    final_pressure: usize,
    optimistic_final_pressure: usize,
    previous: Option<Rc<Self>>,
}

impl Breadcrumb {
    fn start(time_left: usize) -> Self {
        Self {
            valve: START_ID.to_string(),
            time_left,
            action: Action::Move,
            final_pressure: 0,
            optimistic_final_pressure: usize::MAX,
            previous: None,
        }
    }

    fn trace_open_valves(&self) -> impl Iterator<Item = &ValveId> + '_ {
        let mut cur = self;
        let mut done = false;
        std::iter::from_fn(move || {
            if done {
                return None;
            }
            loop {
                let id: Option<&String> = (cur.action == Action::Open).then_some(&cur.valve);

                match &cur.previous {
                    Some(previous) => cur = previous.as_ref(),
                    None => {
                        done = true;
                        return id;
                    },
                }

                if id.is_some() {
                    return id;
                }
            }
        })
    }

    fn has_opened(&self, id: &str) -> bool {
        self.trace_open_valves().any(|v| v == id)
    }

    fn has_opened_cur(&self) -> bool {
        self.has_opened(&self.valve)
    }
}

#[derive(Debug, Clone)]
struct BreadcrumbPriority(Breadcrumb);

impl From<Breadcrumb> for BreadcrumbPriority {
    fn from(value: Breadcrumb) -> Self {
        BreadcrumbPriority(value)
    }
}

impl PartialEq for BreadcrumbPriority {
    fn eq(&self, other: &Self) -> bool {
        self.0.final_pressure == other.0.final_pressure
    }
}

impl Eq for BreadcrumbPriority {
}

impl PartialOrd for BreadcrumbPriority {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BreadcrumbPriority {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.final_pressure.cmp(&other.0.final_pressure)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Action {
    Move,
    Open,
}

impl FromStr for ValveSystem {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let valves: HashMap<String, Valve> = s.lines()
            .map(Valve::from_str)
            .map(|v| v.map(|v| (v.id.clone(), v)))
            .collect::<Result<_, _>>()?;
        if !valves.contains_key(START_ID) {
            Err(anyhow::anyhow!("No valve with id {} found", START_ID))?;
        }

        let mut ordered_valve_rates = valves.values()
            .map(|v| (v.id.clone(), v.rate))
            .collect::<Vec<_>>();
        ordered_valve_rates.sort_by_key(|v| Reverse(v.1));

        Ok(ValveSystem { valves, ordered_valve_rates })
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

    Ok(())
}
