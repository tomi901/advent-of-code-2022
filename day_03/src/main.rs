use std::{char, collections::HashSet, fmt::Display};
use cli_clipboard::{ClipboardContext, ClipboardProvider};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Item {
    id: u8,
}

impl Item {
    pub fn new(id: u8) -> Self {
        Self { id }
    }

    pub fn new_lookup(ids: &str) -> HashSet<Self> {
        Self::many_from_str(ids).collect()
    }

    pub fn many_from_str(ids: &str) -> impl Iterator<Item = Self> + '_ {
        ids.bytes().map(Self::new)
    }

    pub fn priority_score(&self) -> u8 {
        match self.id {
            id @ b'a'..=b'z' => (id - b'a') + 1,
            id @ b'A'..=b'Z' => (id - b'A') + 27,
            _ => 0,
        }
    }
}

fn main() {
    // part_1();
    part_2();
}

fn part_1() {
    let input = std::fs::read_to_string("./input.txt").expect("Error reading input file.");
    
    let mut result: u64 = 0;
    for line in input.lines() {
        let (l_half, r_half) = line.split_at(line.len() / 2);
        let (l_items, r_items) = (Item::new_lookup(l_half), Item::new_lookup(r_half));

        for item in l_items.intersection(&r_items) {
            let priority = item.priority_score();
            result += priority as u64;
        }
    }

    display_result(&result);
}

fn part_2() {
    let input = std::fs::read_to_string("./input.txt").expect("Error reading input file.");

    let mut result: u64 = 0;
    let mut groups = 0;
    const GROUP_SIZE: usize = 3;

    let mut common_items = HashSet::new();

    for line in input.lines() {
        if groups == 0 {
            common_items.extend(Item::many_from_str(line));
        } else {
            let current_lookup = Item::new_lookup(line);
            common_items.retain(|i| current_lookup.contains(i));
        }

        groups += 1;
        if groups < GROUP_SIZE {
            continue;
        }

        // println!("Processing group:");
        for item in common_items.drain() {
            let priority = item.priority_score() as u64;
            // println!("Found id {}: {} += {}", char::from_u32(item.id as u32).unwrap(), result, priority);
            result += priority;
        }
        groups = 0;
    }

    display_result(&result);
}

// TODO: Move this to common library crate
fn display_result<T: Display>(result: &T) {
    println!("Result:");
    let str_result = format!("{}", result);
    println!("{}", &str_result);

    let mut clipboard = ClipboardContext::new().unwrap();
    clipboard.set_contents(str_result.clone()).unwrap();
    println!("Copied result to clipboard!");
}
