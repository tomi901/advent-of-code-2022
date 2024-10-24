use std::{collections::HashMap, iter, str::FromStr};

use anyhow::Context;
use xmas::{display_result, map2d::Map2D, point2d::Point2D};

struct Line(Vec<Point2D>);

impl Line {
    fn point_from_str(s: &str) -> Result<Point2D, anyhow::Error> {
        let (left_s, right_s) = s.split_once(',').context("Expected , separator")?;
        Ok(Point2D(
            left_s.trim().parse()?,
            right_s.trim().parse()?,
        ))
    }

    fn get_max(&self, current: Option<Point2D>) -> Option<Point2D> {
        let mut cur = current.clone();
        for point in self.0.iter() {
            if let Some(_cur) = cur {
                cur = Some(point.max(_cur))
            } else {
                cur = Some(*point);
            }
        }
        cur
    }

    fn get_min(&self, current: Option<Point2D>) -> Option<Point2D> {
        let mut cur = current.clone();
        for point in self.0.iter() {
            if let Some(_cur) = cur {
                cur = Some(point.min(_cur))
            } else {
                cur = Some(*point);
            }
        }
        cur
    }

    fn points(&self) -> impl Iterator<Item = Point2D> + '_ {
        self.0.first()
            .into_iter()
            .cloned()
            .chain((1..self.0.len())
                .map(|i| (self.0[i - 1], self.0[i]))
                .flat_map(|(from, to)| Self::get_points(from, to))
            )
    }

    fn get_points(from: Point2D, to: Point2D) -> impl Iterator<Item = Point2D> {
        let towards = from.try_get_direction_towards(to);
        let mut cur = from;
        let mut step = 0;
        iter::from_fn(move || {
            let (dir, amount) = match towards {
                Some(v) => v,
                None => return None,
            };

            cur += dir.as_point();
            step += 1;
            if step <= amount {
                Some(cur)
            } else {
                None
            }
        })
    }
}

impl FromStr for Line {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let line = s.split("->")
            .map(str::trim)
            .map(Self::point_from_str)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self(line))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CollisionResult {
    Collision,
    Air,
    Outside,
}

struct CaveMap {
    map: Map2D,
    sand_source: Point2D,
}

impl FromStr for CaveMap {
    type Err = anyhow::Error;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lines = s.lines().map(Line::from_str).collect::<Result<Vec<_>, _>>()?;
        Self::try_new(lines)
    }
}

impl CaveMap {
    const SAND_SOURCE: Point2D = Point2D(500, 0);

    pub fn try_new(lines: Vec<Line>) -> Result<Self, anyhow::Error> {
        let mut option_min = None;
        let mut option_max = None;
        for line in lines.iter() {
            option_min = line.get_min(option_min);
            option_max = line.get_max(option_max);
        }
        let min = option_min.map(|p| Point2D(p.0, 0)).context("No min/max found")?;
        let max = option_max.context("No min/max found")? + Point2D(1, 1);

        let size = max - min;
        let mut map = Map2D::new_filled(size, b'.');

        for point in lines.iter().flat_map(Line::points).map(|p| p - min) {
            // println!("Marking point: {}", point);
            if !map.set_tile(point, b'#') {
                // return Err(anyhow!("Couldn't mark point {}", point));
            }
        }

        let sand_source = Self::SAND_SOURCE - min;
        if !map.set_tile(sand_source, b'+') {
            // return Err(anyhow!("Couldn't mark sand source point {}", sand_source));
        }

        // println!("Created map with size {}, min: {}, max: {}", size, min, max);
        // println!("{}", map);

        Ok(Self { map, sand_source })
    }

    pub fn spawn_sand(&mut self) -> bool {
        if self.try_collision(self.sand_source) == CollisionResult::Collision {
            panic!("Can't spawn while source is blocked")
        }

        const CHECK_POINTS: [Point2D; 3] = [Point2D(0, 1), Point2D(-1, 1), Point2D(1, 1)];
        let mut position = self.sand_source;

        loop {
            let mut did_move = false;
            for check_point in CHECK_POINTS.map(|p| position + p) {
                match self.try_collision(check_point) {
                    CollisionResult::Collision => continue,
                    CollisionResult::Air => {
                        position = check_point;
                        did_move = true;
                        break;
                    },
                    CollisionResult::Outside => return false,
                }
            }

            if !did_move {
                self.map.set_tile(position, b'O');
                return true;
            }
        }
    }

    pub fn try_collision(&self, at: Point2D) -> CollisionResult {
        match self.map.get_tile(at) {
            Some(b'#' | b'O') => CollisionResult::Collision,
            Some(_) => CollisionResult::Air,
            None => CollisionResult::Outside,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tile {
    Rock,
    Sand,
}

struct CaveHashMap {
    tiles: HashMap<Point2D, Tile>,
    floor: isize,
}

impl CaveHashMap {
    pub fn new(lines: Vec<Line>) -> Self {
        let mut tiles = HashMap::new();
        let mut floor = 0;
        for point in lines.iter().flat_map(Line::points) {
            tiles.insert(point, Tile::Rock);
            floor = floor.max(point.1);
        }
        floor += 2;
        Self { tiles, floor }
    }

    pub fn spawn_sand(&mut self) -> bool {
        if self.check_collision(CaveMap::SAND_SOURCE) {
            return false;
        }

        let mut position = CaveMap::SAND_SOURCE;
        const CHECK_POINTS: [Point2D; 3] = [Point2D(0, 1), Point2D(-1, 1), Point2D(1, 1)];

        loop {
            let mut did_move = false;
            for check_point in CHECK_POINTS.map(|p| position + p) {
                if check_point.1 < self.floor && !self.check_collision(check_point) {
                    position = check_point;
                    did_move = true;
                    break;
                }
            }

            if !did_move {
                self.tiles.insert(position, Tile::Sand);
                // println!("Positioned sand at: {}", position);
                return true;
            }
        }
    }

    fn check_collision(&self, point: Point2D) -> bool {
        self.tiles.contains_key(&point)
    }
}

impl FromStr for CaveHashMap {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lines = s.lines().map(Line::from_str).collect::<Result<Vec<_>, _>>()?;
        Ok(Self::new(lines))
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
    let mut map = CaveMap::from_str(&input)?;

    let mut result = 0;
    while map.spawn_sand() {
        result += 1;
    }

    // println!("Placed sand:");
    // println!("{}", map.map);

    display_result(&result);
    Ok(())
}

fn part_2() -> anyhow::Result<()> {
    println!("Part 2:");

    let input = std::fs::read_to_string("./input.txt").context("Error reading input file.")?;
    let mut map = CaveHashMap::from_str(&input)?;

    let mut result = 0;
    while map.spawn_sand() {
        result += 1;
    }

    display_result(&result);
    Ok(())
}
