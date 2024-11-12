use std::{collections::BinaryHeap, str::FromStr};

use anyhow::{self, Context};
use xmas::display_result;
use regex_static::{lazy_regex, Regex, once_cell::sync::Lazy};
use rayon::prelude::*;

static BLUEPRINT_REGEX: Lazy<Regex> = lazy_regex!(
    r"Blueprint (\d+).*ore.*(\d+) ore.*clay.*(\d+) ore.*obsidian.*(\d+) ore and (\d+) clay.*geode.*(\d+) ore and (\d+) obsidian"
);

type Minutes = u64;

#[derive(Debug, Clone)]
struct Blueprint {
    number: u64,
    ore_robot_ore_cost: u64,
    clay_robot_ore_cost: u64,
    obs_robot_ore_clay_cost: (u64, u64),
    geode_robot_ore_obsidian_cost: (u64, u64),
}

impl Blueprint {
    pub fn get_max_geodes_path(&self, initial_state: State) -> u64 {
        let mut max_geodes = 0;

        let mut open_list = BinaryHeap::new();
        open_list.push(initial_state.as_geode_ord());

        let mut nodes_considered = 0;

        while let Some(candidate) = open_list.pop() {
            nodes_considered += 1;
            if candidate.key > max_geodes {
                max_geodes = candidate.key;
                // println!("Found candidate: {}", max_geodes);
            }
            let candidate = candidate.value;

            // We could simplify this list by creating a ResourceList struct
            // and set the consumption and increase rates

            let create_ore_bot = candidate.ore
                .time_to_get(self.ore_robot_ore_cost)
                .and_then(|time| candidate.after_strict(time + 1))
                .map(|s| State {
                    ore: s.ore.consume(self.ore_robot_ore_cost).add_generation(1),
                    ..s
                });

            let create_clay_robot = candidate.ore
                .time_to_get(self.clay_robot_ore_cost)
                .and_then(|time| candidate.after_strict(time + 1))
                .map(|s| State {
                    ore: s.ore.consume(self.clay_robot_ore_cost),
                    clay: s.clay.add_generation(1),
                    ..s
                });

            let create_obsidian_robot = candidate.ore
                .time_to_get(self.obs_robot_ore_clay_cost.0)
                .and_then(|t1| candidate.clay.time_to_get(self.obs_robot_ore_clay_cost.1)
                    .map(|t2| t1.max(t2)))
                .and_then(|time| candidate.after_strict(time + 1))
                .map(|s| State {
                    ore: s.ore.consume(self.obs_robot_ore_clay_cost.0),
                    clay: s.clay.consume(self.obs_robot_ore_clay_cost.1),
                    obsidian: s.obsidian.add_generation(1),
                    ..s
                });

            let create_geode_robot = candidate.ore
                .time_to_get(self.geode_robot_ore_obsidian_cost.0)
                .and_then(|t1| candidate.obsidian.time_to_get(self.geode_robot_ore_obsidian_cost.1)
                    .map(|t2| t1.max(t2)))
                .and_then(|time| candidate.after_strict(time + 1))
                .map(|s| State {
                    ore: s.ore.consume(self.geode_robot_ore_obsidian_cost.0),
                    obsidian: s.obsidian.consume(self.geode_robot_ore_obsidian_cost.1),
                    geodes: s.geodes.add_generation(1),
                    ..s
                });

            let new_paths = create_ore_bot.into_iter()
                .chain(create_clay_robot)
                .chain(create_obsidian_robot)
                .chain(create_geode_robot)
                .map(State::as_geode_ord);
            open_list.extend(new_paths);
        }

        println!("Blueprint {} geodes {}, nodes considered {}", self.number, max_geodes, nodes_considered);
        max_geodes
    }
}

impl FromStr for Blueprint {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let captures = BLUEPRINT_REGEX.captures(s)
            .with_context(|| format!("Invalid blueprint string:\n{}", s))?;

        Ok(Self {
            number: parse_num(&captures, 1),
            ore_robot_ore_cost: parse_num(&captures, 2),
            clay_robot_ore_cost: parse_num(&captures, 3),
            obs_robot_ore_clay_cost: (parse_num(&captures, 4), parse_num(&captures, 5)),
            geode_robot_ore_obsidian_cost: (parse_num(&captures, 6), parse_num(&captures, 7)),
        })
    }
}

#[derive(Debug, Clone)]
struct State {
    time_left: u64,
    ore: Resource,
    clay: Resource,
    obsidian: Resource,
    geodes: Resource,
}

impl State {
    pub fn initial_state(time_left: u64) -> Self {
        Self {
            time_left,
            ore: Resource { amount: 0, generation_per_minute: 1 },
            clay: Resource { amount: 0, generation_per_minute: 0 },
            obsidian: Resource { amount: 0, generation_per_minute: 0 },
            geodes: Resource { amount: 0, generation_per_minute: 0 },
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
        })
    }

    pub fn after_strict(&self, time: Minutes) -> Option<Self> {
        self.after(time).filter(|s| s.time_left > 1)
    }

    pub fn final_geodes(&self) -> u64 {
        self.geodes.after(self.time_left).amount
    }

    pub fn as_geode_ord(self) -> KeyedOrd<Self, u64> {
        let geodes = self.final_geodes();
        KeyedOrd { value: self, key: geodes }
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
    part_1()?;
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
    let result = blueprints.par_iter()
        .map(|bp| bp.number * bp.get_max_geodes_path(State::initial_state(TIME)))
        .sum::<u64>();

    display_result(&result);
    Ok(())
}

fn part_2() -> anyhow::Result<()> {
    println!("Part 2:");

    Ok(())
}
