use std::{collections::HashSet, str::FromStr};

use anyhow::{self, Context};
use xmas::{direction3d::DIRECTIONS_3D, display_result, point3d::Point3D};

fn main() -> anyhow::Result<()> {
    part_1()?;
    println!();
    part_2()?;
    Ok(())
}

fn part_1() -> anyhow::Result<()> {
    println!("Part 1:");
    let input = std::fs::read_to_string("./input.txt").context("Error reading input file.")?;

    let points: Vec<_> = input.lines().map(Point3D::from_str).collect::<Result<_, _>>()?;
    let result = calculate_surface_area(&points);

    display_result(&result);
    Ok(())
}

fn part_2() -> anyhow::Result<()> {
    println!("Part 2:");

    Ok(())
}

fn calculate_surface_area(points: &[Point3D]) -> usize {
    let mut already_placed = HashSet::new();
    let mut area = 0;
    for &point in points.iter() {
        area += DIRECTIONS_3D.len();
        area -= DIRECTIONS_3D.iter()
            .map(|d| point + d.as_point())
            .filter(|p| already_placed.contains(p))
            .count() * 2;
        already_placed.insert(point);
    }
    area
}
