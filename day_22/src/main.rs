use std::str::FromStr;

use anyhow::{self, Context};
use xmas::{direction::Direction, display_result, map2d::Map2D, point2d::Point2D};
use regex_static::{lazy_regex, Regex, once_cell::sync::Lazy};

fn main() -> anyhow::Result<()> {
    part_1()?;
    println!();
    part_2()?;
    Ok(())
}

fn part_1() -> anyhow::Result<()> {
    println!("Part 1:");
    let input = std::fs::read_to_string("./input.txt").context("Error reading input file.")?;

    let (map, movements) = input.split_once("\n\n").context("No movements found")?;
    let mut map = PasswordMap::from_str(map)?;
    let movements = Movement::many_from_str(movements)?;

    let (final_pos, final_dir) = map.travel(&movements);
    let result = calculate_result(final_pos, final_dir);

    map.map.set_tile(final_pos, b'X');
    // println!("Found solution at {final_pos} with dir {final_dir:?}:\n{}", map.map);

    display_result(&result);
    Ok(())
}

fn part_2() -> anyhow::Result<()> {
    println!("Part 2:");

    Ok(())
}

static MOVEMENT_REGEX: Lazy<Regex> = lazy_regex!(r"\d+|R|L");

fn calculate_result(position: Point2D, direction: Direction) -> u64 {
    let y_result = 1000 * (position.1.unsigned_abs() as u64 + 1);
    let x_result = 4 * (position.0.unsigned_abs() as u64 + 1);
    let dir = match direction {
        Direction::Right => 0,
        Direction::Down => 1,
        Direction::Left => 2,
        Direction::Up => 3,
    };
    x_result + y_result + dir
}

#[derive(Debug, Clone)]
enum Movement {
    Move(u64),
    Turn(Direction),
}

impl Movement {
    fn many_from_str(s: &str) -> Result<Vec<Self>, anyhow::Error> {
        MOVEMENT_REGEX
            .find_iter(s)
            .map(|m| Self::from_str(m.as_str()))
            .collect()
    }

    fn from_str(s: &str) -> Result<Self, anyhow::Error> {
        let movement = match s {
            "R" => Self::Turn(Direction::Right),
            "L" => Self::Turn(Direction::Left),
            _ => Self::Move(s.parse::<u64>().context("Not R, L or a unsigned int")?),
        };
        Ok(movement)
    }
}

struct PasswordMap {
    map: Map2D,
    start: Point2D,
}

impl PasswordMap {
    fn new(map: Map2D) -> Result<Self, anyhow::Error> {
        let start = map.row(0).iter()
            .position(|&tile| tile == b'.')
            .map(|x| Point2D(x as isize, 0))
            .context("No starting position")?;
        Ok(Self { map, start })
    }

    fn travel(&self, movements: &[Movement]) -> (Point2D, Direction) {
        let mut pos = self.start;
        let mut dir = Direction::Right;
        // println!("Starting at {pos}");
        for movement in movements {
            let mut steps = match movement {
                Movement::Move(steps) => *steps,
                Movement::Turn(turn) => {
                    dir = dir.turn(*turn);
                    continue;
                },
            };

            let mut would_be_at = pos;
            while steps > 0 {
                would_be_at += dir.as_point();
                would_be_at = self.wrap_pos(would_be_at);
                // println!("Testing: {would_be_at}");
                match self.map.get_tile(would_be_at).unwrap() {
                    b'.' => {
                        steps -= 1;
                        pos = would_be_at;
                        // println!("{}", pos);
                    },
                    b'#' => break,
                    _ => (),
                }
            }
        }
        (pos, dir)
    }

    fn wrap_pos(&self, point: Point2D) -> Point2D {
        let width = self.map.width() as isize;
        let height = self.map.height() as isize;
        Point2D(wrap_value(point.0, width), wrap_value(point.1, height))
    }
}

impl FromStr for PasswordMap {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let width = s.lines().map(|l| l.len()).max().unwrap_or(0) as isize;
        let height = s.lines().count() as isize;
        let mut map = Map2D::new_filled(Point2D(width, height), b' ');

        for (y, line) in s.lines().enumerate() {
            for (x, tile) in line.bytes().enumerate() {
                let point = Point2D(x as isize, y as isize);
                map.set_tile(point, tile);
            }
        }
        PasswordMap::new(map)
    }
}

fn wrap_value(val: isize, limit: isize) -> isize {
    match val % limit {
        res if res >= 0 => res,
        res => res + limit,
    }
}
