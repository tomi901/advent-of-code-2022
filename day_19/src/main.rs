use std::{collections::BinaryHeap, fmt::Write, rc::Rc, str::FromStr};

use anyhow::{self, Context};
use enum_map::{Enum, EnumMap};
use xmas::display_result;
use regex_static::{lazy_regex, Regex, once_cell::sync::Lazy};
use rayon::prelude::*;
use ResourceType::*;

static BLUEPRINT_REGEX: Lazy<Regex> = lazy_regex!(
    r"Blueprint (\d+).*ore.*(\d+) ore.*clay.*(\d+) ore.*obsidian.*(\d+) ore and (\d+) clay.*geode.*(\d+) ore and (\d+) obsidian"
);

type Minutes = u64;
type ResourceList = EnumMap<ResourceType, u64>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
enum ResourceType {
    Ore,
    Clay,
    Obsidian,
    Geode,
}

const RESOURCE_TYPES: [ResourceType; 4] =
    [ResourceType::Ore, ResourceType::Clay, ResourceType::Obsidian, ResourceType::Geode];

#[derive(Debug, Clone)]
struct Blueprint {
    number: u64,
    ore_robot_ore_cost: u64,
    clay_robot_ore_cost: u64,
    obs_robot_ore_clay_cost: (u64, u64),
    geode_robot_ore_obsidian_cost: (u64, u64),
    costs: EnumMap<ResourceType, ResourceList>,
    max_costs: ResourceList,
}

impl Blueprint {
    pub fn get_max_geodes_path(&self, initial_state: State) -> u64 {
        let total_time = initial_state.time_left;
        let mut max_geodes = 0;
        let mut best_node = None;

        let mut open_list = BinaryHeap::new();
        open_list.push(initial_state.as_geode_ord());

        let mut nodes_considered = 0;

        while let Some(candidate_with_score) = open_list.pop() {
            nodes_considered += 1;
            let candidate = Rc::new(candidate_with_score.value);
            if candidate_with_score.key > max_geodes {
                max_geodes = candidate_with_score.key;
                // println!("Found candidate: {}\n{:#?}", max_geodes, candidate.as_path());
                best_node = Some(candidate.clone());
            }

            // We could simplify this list by creating a ResourceList struct
            // and set the consumption and increase rates

            let create_ore_bot = candidate.ore
                .time_to_get(self.ore_robot_ore_cost)
                .filter(|_| candidate.could_create_more(Ore, self))
                .and_then(|time| candidate.after(time + 1))
                .map(|s| State {
                    ore: s.ore.consume(self.ore_robot_ore_cost).add_generation(1),
                    created_robot: Ore,
                    ..s
                });

            let create_clay_robot = candidate.ore
                .time_to_get(self.clay_robot_ore_cost)
                .filter(|_| candidate.could_create_more(Clay, self))
                .and_then(|time| candidate.after(time + 1))
                .map(|s| State {
                    ore: s.ore.consume(self.clay_robot_ore_cost),
                    clay: s.clay.add_generation(1),
                    created_robot: Clay,
                    ..s
                });

            let create_obsidian_robot = candidate.ore
                .time_to_get(self.obs_robot_ore_clay_cost.0)
                .filter(|_| candidate.could_create_more(Obsidian, self))
                .and_then(|t1| candidate.clay.time_to_get(self.obs_robot_ore_clay_cost.1)
                    .map(|t2| t1.max(t2)))
                .and_then(|time| candidate.after(time + 1))
                .map(|s| State {
                    ore: s.ore.consume(self.obs_robot_ore_clay_cost.0),
                    clay: s.clay.consume(self.obs_robot_ore_clay_cost.1),
                    obsidian: s.obsidian.add_generation(1),
                    created_robot: Obsidian,
                    ..s
                });

            let create_geode_robot = candidate.ore
                .time_to_get(self.geode_robot_ore_obsidian_cost.0)
                .and_then(|t1| candidate.obsidian.time_to_get(self.geode_robot_ore_obsidian_cost.1)
                    .map(|t2| t1.max(t2)))
                .and_then(|time| candidate.after(time + 1))
                .map(|s| State {
                    ore: s.ore.consume(self.geode_robot_ore_obsidian_cost.0),
                    obsidian: s.obsidian.consume(self.geode_robot_ore_obsidian_cost.1),
                    geodes: s.geodes.add_generation(1),
                    created_robot: Geode,
                    ..s
                });

            let new_paths = create_ore_bot.into_iter()
                .chain(create_clay_robot)
                .chain(create_obsidian_robot)
                .chain(create_geode_robot)
                .map(|s| State {
                    previous: Some(candidate.clone()),
                    ..s
                })
                .map(State::as_geode_ord);
            open_list.extend(new_paths);
        }

        // println!("Blueprint {} geodes {}, nodes considered {}\nPath:\n{:#?}", self.number, max_geodes, nodes_considered,
        //     best_node.unwrap().as_path());
        // if self.number == 1 {
        //     let mut message = format!("Blueprint {}:\n", self.number);
        //     for (i, node) in best_node.unwrap().as_path().iter().enumerate() {
        //         writeln!(message, "{}. Minute {} constructed {:?}:", i + 1, total_time - node.time_left, node.created_robot).unwrap();
        //         writeln!(message, " - Ore: {} + {} / m", node.ore.amount, node.ore.generation_per_minute).unwrap();
        //         writeln!(message, " - Clay: {} + {} / m", node.clay.amount, node.clay.generation_per_minute).unwrap();
        //         writeln!(message, " - Obsidian: {} + {} / m", node.obsidian.amount, node.obsidian.generation_per_minute).unwrap();
        //         writeln!(message, " - Geodes: {} + {} / m", node.geodes.amount, node.geodes.generation_per_minute).unwrap();
        //     }
        //     println!("{}", message);
        // }

        max_geodes
    }
}

impl FromStr for Blueprint {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let captures = BLUEPRINT_REGEX.captures(s)
            .with_context(|| format!("Invalid blueprint string:\n{}", s))?;

        let mut costs = EnumMap::<ResourceType, ResourceList>::default();
        costs[Ore][Ore] = parse_num(&captures, 2);
        costs[Clay][Ore] = parse_num(&captures, 3);
        costs[Obsidian][Ore] = parse_num(&captures, 4);
        costs[Obsidian][Clay] = parse_num(&captures, 5);
        costs[Geode][Ore] = parse_num(&captures, 6);
        costs[Geode][Obsidian] = parse_num(&captures, 7);
        // println!("costs: {:#?}", costs);

        let max_costs = costs.values()
            .cloned()
            .reduce(|a, b| a.into_iter().map(|(k, v)| (k, v.max(b[k]))).collect::<ResourceList>())
            .unwrap_or_default();
        println!("max_costs: {:?}", max_costs);

        Ok(Self {
            number: parse_num(&captures, 1),
            ore_robot_ore_cost: costs[Ore][Ore],
            clay_robot_ore_cost: costs[Clay][Ore],
            obs_robot_ore_clay_cost: (costs[Obsidian][Ore], costs[Obsidian][Clay]),
            geode_robot_ore_obsidian_cost: (costs[Geode][Ore], costs[Geode][Obsidian]),
            costs,
            max_costs,
        })
    }
}

#[derive(Debug, Clone)]
struct State {
    time_left: u64,
    // stock: ResourceList,
    // generating: ResourceList,
    ore: Resource,
    clay: Resource,
    obsidian: Resource,
    geodes: Resource,
    created_robot: ResourceType,
    previous: Option<Rc<Self>>,
}

impl State {
    pub fn initial_state(time_left: u64) -> Self {
        // let mut generating = ResourceList::default();
        // generating[ResourceType::Ore] = 1;
        Self {
            time_left,
            ore: Resource { amount: 0, generation_per_minute: 1 },
            clay: Resource { amount: 0, generation_per_minute: 0 },
            obsidian: Resource { amount: 0, generation_per_minute: 0 },
            geodes: Resource { amount: 0, generation_per_minute: 0 },
            created_robot: ResourceType::Ore,
            // generating,
            // stock: Default::default(),
            previous: None,
        }
    }

    pub fn after(&self, time: Minutes) -> Option<Self> {
        if time > self.time_left {
            return None;
        }

        Some(Self {
            time_left: self.time_left - time,
            ore: self.ore.after(time),
            clay: self.clay.after(time),
            obsidian: self.obsidian.after(time),
            geodes: self.geodes.after(time),
            created_robot: self.created_robot,
            previous: None,
        })
    }

    pub fn final_geodes(&self) -> u64 {
        self.geodes.after(self.time_left).amount
    }

    pub fn as_geode_ord(self) -> KeyedOrd<Self, u64> {
        let geodes = self.final_geodes();
        KeyedOrd { value: self, key: geodes }
    }

    pub fn could_create_more(&self, res: ResourceType, bp: &Blueprint) -> bool {
        let amount = self.amount_of(res);
        amount <= bp.max_costs[res] + 1
    }

    pub fn amount_of(&self, res: ResourceType) -> u64 {
        match res {
            Ore => self.ore.amount,
            Clay => self.clay.amount,
            Obsidian => self.obsidian.amount,
            Geode => self.geodes.amount,
        }
    }

    pub fn as_path(&self) -> Vec<Self> {
        let mut cur = self;
        let mut path = Vec::new();
        loop {
            let mut node = cur.clone();
            node.previous = None;
            path.push(node);

            match &cur.previous {
                Some(previous) => cur = previous.as_ref(),
                None => break,
            }
        }
        path.reverse();
        path
    }
}

#[derive(Debug, Clone)]
struct KeyedOrd<T, K> {
    value: T,
    key: K,
}

impl<T, K: PartialEq> PartialEq for KeyedOrd<T, K> {
    fn eq(&self, other: &Self) -> bool {
        self.key.eq(&other.key)
    }
}

impl<T, K: PartialOrd> PartialOrd for KeyedOrd<T, K> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.key.partial_cmp(&other.key)
    }
}

impl<T, K: Eq> Eq for KeyedOrd<T, K> {
}

impl<T, K: Ord> Ord for KeyedOrd<T, K> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.key.cmp(&other.key)
    }
}

#[derive(Debug, Clone, Copy)]
struct Resource {
    amount: u64,
    generation_per_minute: Minutes,
}

impl Resource {
    pub fn time_to_get(&self, amount: u64) -> Option<Minutes> {
        // println!("Wanting to get {}, currently have {} @ {} / minute", amount, self.amount, self.generation_per_minute);
        if self.amount >= amount {
            return Some(0);
        }

        if self.generation_per_minute == 0 {
            return None;
        }

        let needed = amount - self.amount;
        Some(needed.div_ceil(self.generation_per_minute))
    }

    pub fn after(&self, time: Minutes) -> Self {
        Self {
            amount: self.amount + (self.generation_per_minute * time),
            generation_per_minute: self.generation_per_minute,
        }
    }

    pub fn consume(&self, amount: u64) -> Self {
        Self {
            amount: self.amount - amount,
            generation_per_minute: self.generation_per_minute,
        }
    }

    pub fn add_generation(&self, amount: u64) -> Self {
        Self {
            amount: self.amount,
            generation_per_minute: self.generation_per_minute + amount,
        }
    }
}

fn parse_num(captures: &regex::Captures<'_>, group: usize) -> u64 {
    captures.get(group).unwrap().as_str().parse::<u64>().unwrap()
}

fn main() -> anyhow::Result<()> {
    // part_1()?;
    println!();
    part_2()?;
    Ok(())
}

fn part_1() -> anyhow::Result<()> {
    println!("Part 1:");
    let input = std::fs::read_to_string("./input.txt").context("Error reading input file.")?;

    let blueprints: Vec<_> = input.lines().map(Blueprint::from_str).collect::<Result<_, _>>()?;
    // println!("{:?}", blueprints);

    const TIME: Minutes = 24;
    let results = blueprints.par_iter()
        .map(|bp| (bp.number, bp.get_max_geodes_path(State::initial_state(TIME))))
        .collect::<Vec<_>>();

    for (n, geodes) in results.iter() {
        println!("Blueprint {}: {} geode/s = {} quality", n, geodes, n * geodes);
    }

    let result = results
        .into_iter()
        .map(|(n, geodes)| n * geodes)
        .sum::<u64>();

    display_result(&result);
    Ok(())
}

fn part_2() -> anyhow::Result<()> {
    println!("Part 2:");
    let input = std::fs::read_to_string("./input.txt").context("Error reading input file.")?;

    let blueprints: Vec<_> = input.lines().take(3).map(Blueprint::from_str).collect::<Result<_, _>>()?;
    // println!("{:?}", blueprints);

    const TIME: Minutes = 32;
    let results = blueprints.par_iter()
        .map(|bp| (bp.number, bp.get_max_geodes_path(State::initial_state(TIME))))
        .collect::<Vec<_>>();

    for (n, geodes) in results.iter() {
        println!("Blueprint {}: {} geode/s", n, geodes);
    }

    let result = results
        .into_iter()
        .map(|(_, geodes)| geodes)
        .reduce(|a, b| a * b)
        .unwrap();

    display_result(&result);
    Ok(())
}
