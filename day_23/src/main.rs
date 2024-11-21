use std::{collections::{HashMap, HashSet}, str::FromStr};

use anyhow::{self, Context};
use xmas::{direction::{Direction::{self, *}, QuarterRotation, DIRECTIONS_8}, display_result, point2d::Point2D};

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
    let input = std::fs::read_to_string("./input.txt").context("Error reading input file.")?;
    let mut elves = ElvesMap::from_str(&input)?;

    let result = elves.count_rounds_until_stopping();

    display_result(&result);
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
        // println!("Bounds {:?}, size is {} or {}", (min, max), size, count);
        count as usize - self.elves.len()
    }

    pub fn count_rounds_until_stopping(&mut self) -> usize {
        let mut rounds = 0;
        loop {
            let moved = self.move_round(rounds);
            rounds += 1;
            if !moved {
                return rounds;
            }
        }
    }

    pub fn move_many_rounds(&mut self, rounds: usize) {
        for i in 0..rounds {
            // self.display();
            // println!();
            self.move_round(i);
        }
        // self.display();
    }

    pub fn move_round(&mut self, round_i: usize) -> bool {
        let proposed_moves = self.elves.iter()
            .filter(|&elf| self.should_move(*elf))
            .flat_map(|&elf| self.propose_move(elf, round_i).map(|p| (elf, p)))
            .collect::<HashMap<_, _>>();
        // for (elf, proposed) in &proposed_moves {
        //     if elf != proposed {
        //         println!("Elf {elf} proposed {proposed}");
        //     }
        // }
        if proposed_moves.len() == 0 {
            return false;
        }

        let mut moves_count = HashMap::<Point2D, usize>::new();
        for proposed in proposed_moves.values() {
            let entry = moves_count.entry(*proposed).or_default();
            *entry += 1;
        }

        // dbg!(&proposed_moves);
        // dbg!(&moves_count);
        let mut any_moved = false;
        for (elf, proposed) in proposed_moves
            .iter()
            .filter(|&(_, proposed)| {
                let count = *moves_count.get(proposed).unwrap_or(&0);
                // println!("Count for {proposed} = {count}");
                count <= 1
            })
        {
            // println!("Moving {} -> {}", elf, proposed);
            let removed = self.elves.remove(elf);
            assert!(removed);
            let inserted = self.elves.insert(*proposed);
            assert!(inserted);
            any_moved = true;
        }
        any_moved
    }

    fn propose_move(&self, elf: Point2D, round_i: usize) -> Option<Point2D> {
        Self::check_dirs(round_i)
            .flat_map(|dir| self.can_move(elf, dir))
            .next()
    }

    fn check_dirs(round_i: usize) -> impl Iterator<Item = Direction> {
        (0..CHECK_DIRS.len())
            .map(move |i| (i + round_i) % CHECK_DIRS.len())
            .map(|i| CHECK_DIRS[i])
    }

    fn can_move(&self, elf: Point2D, dir: Direction) -> Option<Point2D> {
        let check_points = [
            dir.as_point(),
            dir.combined(dir.turn_rotation(QuarterRotation::Left)),
            dir.combined(dir.turn_rotation(QuarterRotation::Right)),
        ].map(|p| elf + p);
        if check_points.iter().all(|p| !self.elves.contains(p)) {
            Some(elf + dir.as_point())
        } else {
            None
        }
    }

    fn should_move(&self, elf: Point2D) -> bool {
        DIRECTIONS_8
            .iter()
            .map(|&p| elf + p)
            .any(|p| self.elves.contains(&p))
    }

    fn display(&self) {
        let (min, max) = self.bounds();
        for y in min.1..=max.1 {
            let line = (min.0..=max.0)
                .map(|x| if self.elves.contains(&Point2D(x, y)) { '#' } else { '.' })
                .collect::<String>();
            println!("{line}");
        }
        let size = max - min + Point2D(1, 1);
        println!("Size: {size}");
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

const CHECK_DIRS: [Direction; 4] = [
    Up,
    Down,
    Left,
    Right,
];
