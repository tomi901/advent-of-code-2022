use std::{fmt::Display, ops::RangeInclusive, str::FromStr};
use cli_clipboard::{ClipboardContext, ClipboardProvider};

struct AssignmentPair(RangeInclusive<u64>, RangeInclusive<u64>);

impl FromStr for AssignmentPair {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn get_range(range_s: &str) -> Result<RangeInclusive<u64>, ()> {
            let (l_s, r_s) = range_s.split_once('-').expect("No - found");
            let (l, r) = (l_s.parse().unwrap(), r_s.parse().unwrap());
            Ok(l..=r)
        }

        let (l_s, r_s) = s.split_once(',').expect("No , found");
        let (left, right) = (get_range(l_s)?, get_range(r_s)?);
        Ok(Self(left, right))
    }
}

impl AssignmentPair {
    pub fn overlap_exists(&self) -> bool {
        Self::range_overlaps(&self.0, &self.1) || Self::range_overlaps(&self.1, &self.0)
    }

    fn range_overlaps(outer: &RangeInclusive<u64>, inner: &RangeInclusive<u64>) -> bool {
        inner.start() >= outer.start() && inner.end() <= outer.end()
    }

    pub fn partial_overlap_exists(&self) -> bool {
        Self::range_partially_overlaps(&self.0, &self.1) || Self::range_partially_overlaps(&self.1, &self.0)
    }

    fn range_partially_overlaps(outer: &RangeInclusive<u64>, inner: &RangeInclusive<u64>) -> bool {
        outer.contains(inner.start()) || outer.contains(inner.end())
    }
}

fn main() {
    // part_1();
    part_2();
}

fn part_1() {
    let input = std::fs::read_to_string("./input.txt").expect("Error reading input file.");
    
    let result = input.lines()
        .map(AssignmentPair::from_str)
        .flat_map(|a| a.ok())
        .filter(AssignmentPair::overlap_exists)
        .count();

    display_result(&result);
}

fn part_2() {
    let input = std::fs::read_to_string("./input.txt").expect("Error reading input file.");
    
    let result = input.lines()
        .map(AssignmentPair::from_str)
        .flat_map(|a| a.ok())
        .filter(AssignmentPair::partial_overlap_exists)
        .count();

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
