use std::{collections::HashSet, str::FromStr};

use xmas::{direction::Direction, display_result, point2d::Point2D};

#[derive(Debug, Default, Clone)]
struct RopeSegment {
    head: Point2D,
    tail_relative: Point2D,
}

impl RopeSegment {
    pub fn at_starting_position() -> Self {
        Self::default()
    }

    pub fn tail_world(&self) -> Point2D {
        self.head + self.tail_relative
    }

    pub fn move_towards(&self, direction: Direction) -> Self {
        let new_head = self.head + direction.as_point();
        let new_tail_relative = self.tail_relative - direction.as_point();

        Self { head: new_head, tail_relative: Self::correct_tail_relative(new_tail_relative) }
    }

    pub fn correct_tail_relative(relative: Point2D) -> Point2D {
        // Could clamp, but some weird behaviour happens because of it
        // (1, 2) should be transformed to (0, 1), but never to (1, 1)
        match relative {
            Point2D(x, y) if x < -1 && y < -1 => Point2D(-1, -1),
            Point2D(x, y) if x < -1 && y > 1 => Point2D(-1, 1),
            Point2D(x, y) if x > 1 && y < -1 => Point2D(1, -1),
            Point2D(x, y) if x > 1 && y > 1 => Point2D(1, 1),
            Point2D(x, _) if x < -1 => Point2D(-1, 0),
            Point2D(x, _) if x > 1 => Point2D(1, 0),
            Point2D(_, y) if y < -1 => Point2D(0, -1),
            Point2D(_, y) if y > 1 => Point2D(0, 1),
            _ => relative,
        }
    }
}

struct Rope {
    segments: Vec<Point2D>,
}

impl Rope {
    pub fn of_length(len: usize) -> Self {
        Self { segments: vec![Point2D::ZERO; len] }
    }

    pub fn move_towards(&mut self, direction: Direction) {
        let mut leading_pos = self.head_pos() + direction.as_point();
        *self.segments.first_mut().unwrap() = leading_pos;
        // println!("Moved head at: {}", leading_pos);

        for follower_pos in self.segments.iter_mut().skip(1) {
            let relative_pos = *follower_pos - leading_pos;
            let corrected_relative_pos = RopeSegment::correct_tail_relative(relative_pos);
            // println!("Correction: {} -> {}", relative_pos, corrected_relative_pos);
            if relative_pos == corrected_relative_pos {
                break; // No need to recalculate the other segments
            }

            let new_pos = leading_pos + corrected_relative_pos;
            *follower_pos = new_pos;
            leading_pos = new_pos;
        }
    }

    pub fn head_pos(&self) -> Point2D {
        *self.segments.first().unwrap()
    }

    pub fn tail_pos(&self) -> Point2D {
        *self.segments.last().unwrap()
    }
}

#[derive(Debug, Clone)]
struct Instruction {
    pub direction: Direction,
    pub amount: usize,
}

impl FromStr for Instruction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (dir_s, amount_s) = s.split_once(' ').expect("Should only have one space");
        let direction = match dir_s {
            "U" => Direction::North,
            "R" => Direction::East,
            "D" => Direction::South,
            "L" => Direction::West,
            c => panic!("Unknown direction: {}", c),
        };

        let amount = amount_s.parse().expect("Error parsing amount");
        Ok(Self { direction, amount })
    }
}

fn main() {
    // part_1();
    part_2();
}

fn part_1() {
    let input = std::fs::read_to_string("./input.txt").expect("Error reading input file.");
    
    let mut rope = RopeSegment::at_starting_position();
    let mut visited = HashSet::new();
    visited.insert(rope.tail_world());
    for instruction_result in input.lines().map(Instruction::from_str) {
        let instruction = instruction_result.expect("Error parsing instruction");
        for _ in 0..instruction.amount {
            rope = rope.move_towards(instruction.direction);
            if visited.insert(rope.tail_world()) {
                // println!("Added point: T{}--H{}", rope.tail_world(), rope.head);
            }
        }
    }

    display_result(&visited.len());
}

fn part_2() {
    let input = std::fs::read_to_string("./input.txt").expect("Error reading input file.");
    let instructions = input.lines().map(Instruction::from_str);
    
    let mut rope = Rope::of_length(10);
    let mut visited = HashSet::new();
    visited.insert(rope.tail_pos());

    for instruction_result in instructions {
        let instruction = instruction_result.expect("Error parsing instruction");
        for _ in 0..instruction.amount {
            rope.move_towards(instruction.direction);
            if visited.insert(rope.tail_pos()) {
                // println!("Added point: T{}--H{}", rope.tail_pos(), rope.head_pos());
            }
        }
    }

    display_result(&visited.len());
}
