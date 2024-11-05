use std::{collections::HashSet, ops::RangeInclusive, str::FromStr};

use anyhow::{self, Context};
use xmas::{display_result, point2d::Point2D};
use regex_static::{lazy_regex, Regex, once_cell::sync::Lazy};

static SENSOR_REGEX: Lazy<Regex> = lazy_regex!(r"x=(-?\d+).*y=(-?\d+).*beacon.*x=(-?\d+).*y=(-?\d+)");

#[derive(Clone, Debug, PartialEq, Eq)]
struct Sensor {
    position: Point2D,
    closest_beacon: Point2D,
}

impl Sensor {
    fn distance_to_beacon(&self) -> usize {
        self.position.manhattan_distance(self.closest_beacon)
    }

    fn get_range_at_row(&self, y: isize) -> Option<RangeInclusive<isize>> {
        let distance = self.distance_to_beacon();
        let y_diff = y.abs_diff(self.position.1);
        if y_diff > distance {
            return None;
        }

        let extend = distance - y_diff;
        // println!("Row {y}: Point {} extends {extend} (Distance: {})", self.position, self.distance_to_beacon());
        Some((self.position.0 - extend as isize)..=(self.position.0 + extend as isize))
    }

    fn does_triangulate(&self, other: &Self) -> bool {
        let distance = self.position.manhattan_distance(other.position);
        let expected_distance = self.distance_to_beacon() + other.distance_to_beacon() + 1;
        if distance.abs_diff(expected_distance) <= 2 {
            println!("{} -> {}, distance: {}, expected: {}", self.position, other.position, distance, expected_distance);
        }
        distance == expected_distance
    }

    fn get_positions_at_distance(&self, distance: usize) -> impl Iterator<Item = Point2D> + '_ {
        const STAGES: [Point2D; 4] = [Point2D(1, 1), Point2D(-1, 1), Point2D(-1, -1), Point2D(1, -1)];

        // println!("Getting positions for {} at distance {}", self.position, distance);

        let starting_position = self.position + Point2D(0, -(distance as isize));
        let mut cur_pos = starting_position;
        let mut stage = 0;
        let mut amount = 0;
        std::iter::from_fn(move || {
            if stage >= STAGES.len() {
                return None;
            }

            if distance == 0 {
                stage = STAGES.len();
                return Some(starting_position);
            }

            let pos = cur_pos;
            let move_towards = STAGES[stage];
            cur_pos += move_towards;
            amount += 1;
            if amount >= distance {
                amount = 0;
                stage += 1;
            }

            // println!(" - Position {}", pos);
            Some(pos)
        })
    }

    fn get_outer_positions(&self) -> impl Iterator<Item = Point2D> + '_ {
        self.get_positions_at_distance(self.distance_to_beacon() + 1)
    }
    
    fn is_in_range(&self, point: Point2D) -> bool {
        let distance = self.position.manhattan_distance(point);
        distance <= self.distance_to_beacon()
    }
}

impl FromStr for Sensor {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let regex = SENSOR_REGEX.captures(s).context("Invalid string format")?;
        let position = Point2D(
            regex.get(1).context("Position x not found")?.as_str().parse().unwrap(),
            regex.get(2).context("Position y not found")?.as_str().parse().unwrap(),
        );
        let closest_beacon = Point2D(
            regex.get(3).context("Beacon x not found")?.as_str().parse().unwrap(),
            regex.get(4).context("Beacon y not found")?.as_str().parse().unwrap(),
        );
        Ok(Self { position, closest_beacon })
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

    let sensors = input.lines().map(Sensor::from_str).collect::<Result<Vec<_>, _>>()?;
    // println!("{:#?}", sensors);

    const CHECK_ROW: isize = 2_000_000;
    // const CHECK_ROW: isize = 10;
    let result = get_non_beacon_count(&sensors, CHECK_ROW);

    display_result(&result);
    Ok(())
}

fn get_non_beacon_count(sensors: &Vec<Sensor>, row: isize) -> usize {
    let ranges = get_ranges_at_row(&sensors, row).collect::<Vec<_>>();

    let min = match ranges.iter().map(|r| *r.start()).min() {
        Some(i) => i,
        None => return 0,
    };
    let max = ranges.iter().map(|r| *r.end()).max().unwrap();
    
    let result = (min..=max)
        .filter(|i| ranges.iter().any(|r| r.contains(i)))
        .count() - 1;

    result
}

fn part_2() -> anyhow::Result<()> {
    println!("Part 2:");
    let input = std::fs::read_to_string("./input.txt").context("Error reading input file.")?;

    let sensors = input.lines().map(Sensor::from_str).collect::<Result<Vec<_>, _>>()?;
    let space = find_beacon_space(&sensors).expect("No beacon space found");

    let result = space.0 * 4000000 + space.1;

    display_result(&result);
    Ok(())
}

fn get_ranges_at_row(sensors: &Vec<Sensor>, row: isize) -> impl Iterator<Item = RangeInclusive<isize>> + '_ {
    sensors.iter().flat_map(move |s| s.get_range_at_row(row))
}

fn find_beacon_space(sensors: &Vec<Sensor>) -> Option<Point2D> {
    const RANGE: RangeInclusive<isize> = 0..=4_000_000;

    for row in RANGE {
        let mut ranges = get_ranges_at_row(sensors, row).collect::<Vec<_>>();
        ranges.sort_by_key(|r| *r.start());

        // println!("Pre-proccessed ranges:");
        // println!("{:?}", ranges);

        let mut merge_index = if ranges.len() >= 2 { ranges.len() - 2 } else { 0 };
        loop {
            if ranges.len() < 2 {
                break;
            }

            let lhs = ranges[merge_index].clone();
            let rhs = ranges[merge_index + 1].clone();
            if let Some(new_range) = try_merge_ranges(lhs, rhs) {
                ranges.remove(merge_index + 1);
                ranges[merge_index] = new_range;
                if ranges.len() < 2 {
                    break;
                }
                merge_index = ranges.len() - 2;
            } else if merge_index > 0 {
                merge_index -= 1;
            } else {
                break;
            }
        }

        if ranges.len() == 2 && ranges[0].end().abs_diff(*ranges[1].start()) == 2 {
            let found = Point2D(*ranges[0].end() + 1, row);
            println!("Found point at: {}", found);
            println!("Ranges:");
            println!("{:?}", ranges);
            return Some(found);
        }
    }
    None
}

fn try_merge_ranges(lhs: RangeInclusive<isize>, rhs: RangeInclusive<isize>) -> Option<RangeInclusive<isize>> {
    let (smaller, greater) = if lhs.start() <= rhs.start() {
        (&lhs, &rhs)
    } else {
        (&rhs, &lhs)
    };
    if smaller.contains(greater.start()) {
        let new_range = *smaller.start()..=*smaller.end().max(greater.end());
        // println!("Trying to merge: {:?} and {:?} = {:?}", lhs, rhs, new_range);
        Some(new_range)
    } else {
        None
    }
}
