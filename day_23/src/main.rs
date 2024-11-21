use std::{collections::{HashMap, HashSet}, str::FromStr};

use anyhow::{self, Context};
use once_cell::sync::Lazy;
use xmas::{direction::Direction::*, display_result, point2d::Point2D};

fn main() -> anyhow::Result<()> {
    part_1()?;
    println!();
    part_2()?;
    Ok(())
}

fn part_1() -> anyhow::Result<()> {
    println!("Part 1:");
    let input = std::fs::read_to_string("./input.txt").context("Error reading input file.")?;
    let mut elves = ElvesMap::from_str(&input)?;

    elves.move_many_rounds(10);
    let result = elves.empty_count();

    display_result(&result);
    Ok(())
}

fn part_2() -> anyhow::Result<()> {
    println!("Part 2:");

    Ok(())
}

struct ElvesMap {
    elves: HashSet<Point2D>,
}

impl ElvesMap {
    pub fn bounds(&self) -> (Point2D, Point2D) {
        let min = self.elves.iter().cloned().reduce(|p1, p2| p1.min(p2)).unwrap();
        let max = self.elves.iter().cloned().reduce(|p1, p2| p1.max(p2)).unwrap();
        (min, max)
    }

    pub fn empty_count(&self) -> usize {
        let (min, max) = self.bounds();
        let size = max - min + Point2D(1, 1);
        let count = size.0 * size.1;
        println!("Bounds {:?}, size is {} or {}", (min, max), size, count);
        count as usize - self.elves.len()
    }

    pub fn move_many_rounds(&mut self, rounds: usize) {
        for i in 0..rounds {
            self.empty_count();
            self.move_round(i);
        }
    }

    pub fn move_round(&mut self, round_i: usize) {
        let proposed_moves = self.elves.iter()
            .cloned()
            .map(|elf| (elf, self.propose_move(elf, round_i)))
            .collect::<HashMap<_, _>>();

        let mut moves_count = HashMap::<Point2D, usize>::new();
        for proposed in proposed_moves.values() {
            let entry = moves_count.entry(*proposed).or_default();
            *entry += 1;
        }

        for (elf, proposed) in proposed_moves
            .iter()
            .filter(|(_, p)| moves_count.get(p).is_some_and(|&count| count <= 1))
        {
            let removed = self.elves.remove(elf);
            assert!(removed);
            let inserted = self.elves.insert(*proposed);
            assert!(inserted);
        }
    }

    fn propose_move(&self, elf: Point2D, round_i: usize) -> Point2D {
        Self::check_dirs(round_i)
            .flat_map(|dir| self.can_move(elf, dir))
            .next()
            .unwrap_or(elf)
    }

    fn check_dirs(round_i: usize) -> impl Iterator<Item = &'static CheckDir> {
        (0..CHECK_DIRS.len())
            .map(move |i| (i + round_i) % CHECK_DIRS.len())
            .map(|i| &CHECK_DIRS[i])
    }

    fn can_move(&self, elf: Point2D, dir: &CheckDir) -> Option<Point2D> {
        for point in dir.map(|p| elf + p) {
            if self.elves.contains(&point) {
                return None;
            }
        }
        return Some(dir[0]);
    }
}

impl FromStr for ElvesMap {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut elves = HashSet::new();
        for (y, line) in s.lines().enumerate() {
            for (x, _) in line.chars().enumerate().filter(|&(_, c)| c == '#') {
                elves.insert(Point2D(x as isize, y as isize));
            }
        }
        Ok(Self { elves })
    }
}

type CheckDir = [Point2D; 3];

static CHECK_DIRS: Lazy<[CheckDir; 4]> = Lazy::new(|| [
    [Up.as_point(), Up.combined(Right), Up.combined(Left)],
    [Down.as_point(), Up.combined(Right), Up.combined(Left)],
    [Left.as_point(), Up.combined(Up), Up.combined(Down)],
    [Right.as_point(), Up.combined(Up), Up.combined(Down)],
]);
