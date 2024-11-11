use std::{collections::{HashSet, VecDeque}, str::FromStr};

use anyhow::{self, Context};
use xmas::{direction3d::DIRECTIONS_3D, display_result, point3d::Point3D};

#[derive(Debug, Clone)]
struct Bounds {
    min: Point3D,
    max: Point3D,
}

impl Bounds {
    pub fn is_inside(&self, point: Point3D) -> bool {
        // println!("Checking if {} is inside of {:?}", point, self);
        point.0 >= self.min.0 && point.1 >= self.min.1 && point.2 >= self.min.2 &&
        point.0 <= self.max.0 && point.1 <= self.max.1 && point.2 <= self.max.2
    }

    pub fn all_points_iter(&self) -> impl Iterator<Item = Point3D> + '_ {
        (self.min.0..=self.max.0)
            .flat_map(|x| (self.min.1..=self.max.1).map(move |y| (x, y)))
            .flat_map(|(x, y)| (self.min.2..=self.max.2).map(move |z| Point3D(x, y, z)))
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

    let points: Vec<_> = input.lines().map(Point3D::from_str).collect::<Result<_, _>>()?;
    let result = calculate_surface_area(&points);

    display_result(&result);
    Ok(())
}

fn part_2() -> anyhow::Result<()> {
    println!("Part 2:");
    let input = std::fs::read_to_string("./input.txt").context("Error reading input file.")?;

    let points: Vec<_> = input.lines().map(Point3D::from_str).collect::<Result<_, _>>()?;
    let result = calculate_exterior_surface_area(&points);

    display_result(&result);
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

fn calculate_exterior_surface_area(points: &[Point3D]) -> usize {
    let mut area = calculate_surface_area(points);
    let min = points.iter().cloned().reduce(|p1, p2| p1.min(p2)).unwrap();
    let max = points.iter().cloned().reduce(|p1, p2| p1.max(p2)).unwrap();
    let bounds = Bounds { min, max };
    // println!("{:?}", bounds);

    let filled_points = points.iter().cloned().collect::<HashSet<_>>();

    let mut checked_points = HashSet::new();
    'outer: for point in bounds.all_points_iter() {
        if checked_points.contains(&point) {
            continue;
        }
        checked_points.insert(point);

        if filled_points.contains(&point) {
            continue;
        }

        // println!("Checking point: {}", point);

        let mut remove_area = 0;
        let mut check_queue = VecDeque::new();
        check_queue.push_back(point);

        let mut already_checked = HashSet::new();
        already_checked.insert(point);

        while let Some(check_point) = check_queue.pop_front() {
            if !bounds.is_inside(check_point) {
                // println!("STOP!");
                continue 'outer;
            }
            checked_points.insert(check_point);

            for candidate in DIRECTIONS_3D.map(|p| p.as_point() + check_point) {
                if filled_points.contains(&candidate) {
                    remove_area += 1;
                    continue;
                }

                if !already_checked.contains(&candidate) {
                    check_queue.push_back(candidate);
                    already_checked.insert(candidate);
                }
            }
        }

        // println!("Removing area: {}", remove_area);
        area -= remove_area;
    }
    area
}
