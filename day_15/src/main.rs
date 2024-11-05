use std::{ops::RangeInclusive, str::FromStr};

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
        Some((self.position.0 - extend as isize)..=(self.position.0 + extend as isize))
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
    let ranges = sensors
        .iter()
        .flat_map(|s| s.get_range_at_row(CHECK_ROW))
        .collect::<Vec<_>>();

    let min = ranges.iter().map(|r| *r.start()).min().unwrap();
    let max = ranges.iter().map(|r| *r.end()).max().unwrap();
    
    let result = (min..=max)
        .filter(|i| ranges.iter().any(|r| r.contains(i)))
        .count() - 1;

    display_result(&result);
    Ok(())
}

fn part_2() -> anyhow::Result<()> {
    println!("Part 2:");

    Ok(())
}