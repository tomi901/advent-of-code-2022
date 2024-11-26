use std::{cmp::Reverse, collections::{BinaryHeap, HashMap, HashSet}, rc::Rc, str::FromStr};

use anyhow::Context;
use num::integer::lcm;
use xmas::{direction::{Direction, DIRECTIONS}, keyed_ord::KeyedOrd, map2d::Map2D, num::wrap_val, point2d::Point2D};

pub type Minutes = isize;

#[derive(Debug, Clone)]
pub struct BlizzardMap {
    map: Map2D,
    start_pos: Point2D,
    target_pos: Point2D,
    blizzards: Vec<Blizzard>,
    blizzard_loop_len: Minutes,
}

impl BlizzardMap {
    pub fn navigate(&self) -> Option<isize> {
        self.navigate_with_options(0, self.start_pos, self.target_pos)
    }

    pub fn navigate_back_and_forth(&self) -> Option<isize> {
        self.navigate_with_options(0, self.start_pos, self.target_pos)
            .and_then(|time| {
                println!("First trip took {}", time);
                match self.navigate_with_options(time, self.target_pos, self.start_pos) {
                    Some(back_time) => {
                        println!("Back trip took {}, time is now: {}", back_time, time + back_time);
                        Some(time + back_time)
                    },
                    None => None,
                }
            })
            .and_then(|time| {
                match self.navigate_with_options(time, self.start_pos, self.target_pos) {
                    Some(second_trip_time) => {
                        println!("Second trip took {}, time is now: {}", second_trip_time, time + second_trip_time);
                        Some(time + second_trip_time)
                    },
                    None => None,
                }
            })
    }

    pub fn navigate_with_options(&self, start_time: Minutes, from: Point2D, to: Point2D) -> Option<isize> {
        // println!("Trip: {} -> {} @ time {}", from, to, start_time);
        let mut open_list = BinaryHeap::new();
        open_list.push(Breadcrumb::new(from, start_time).as_priority_with_target(to));

        let mut closed_list = HashSet::new();

        // let mut i = 0;
        while let Some(candidate) = open_list.pop() {
            // if i % 100_000 == 0 {
            //     println!("Next cost is: {}", candidate.key.0);
            //     println!("Queue has {} element/s", open_list.len());
            //     println!("Distance: {}", candidate.value.pos.manhattan_distance(self.target_pos));
            // }
            // i += 1;

            if candidate.value.pos == to {
                return Some(candidate.value.time - start_time);
            }

            let explored = ExploredPoint {
                point: candidate.value.pos,
                time: candidate.value.time % self.blizzard_loop_len,
            };
            if closed_list.contains(&explored) {
                // println!("Already explored: {:?}", explored);
                continue;
            }
            closed_list.insert(explored);

            let candidate = Rc::new(candidate.value);

            let next_time = candidate.time + 1;
            let map_size = self.map.size();
            let next_blizzard_positions = self.blizzards.iter()
                .map(|b| b.position_after(next_time, map_size))
                .collect::<HashSet<_>>();
            
            let at_pos = candidate.pos;
            let adjacent = DIRECTIONS.iter()
                .map(|d| at_pos + d.as_point())
                .filter(|&p| self.map.get_tile(p).is_some_and(|t| t != &b'#'));

            let next_candidates = Some(at_pos).into_iter()
                .chain(adjacent)
                .filter(|p| !next_blizzard_positions.contains(&p))
                .map(|pos| Breadcrumb { pos, time: next_time, previous: Some(candidate.clone()) })
                .map(|bc| bc.as_priority_with_target(to));

            open_list.extend(next_candidates);
        }
        None
    }

    pub fn display_at(&self, time: Minutes) {
        println!("Map @ minute {}", time);
        let positions = self.blizzards
            .iter()
            .map(|b| b.after(time, self.map.size()))
            .map(|b| (b.origin, b))
            .collect::<HashMap<_, _>>();
        for (y, row) in self.map.rows_iter().enumerate() {
            let mut line = String::new();
            for (point, &tile) in row.iter().enumerate()
                .map(|(x, t)| (Point2D(x as isize, y as isize), t))
            {
                if tile == b'#' {
                    line.push('#');
                    continue;
                }

                let dir = match positions.get(&point) {
                    Some(b) => b.dir,
                    None => {
                        line.push('.');
                        continue;
                    },
                };

                line.push(match dir {
                    Direction::Up => '^',
                    Direction::Left => '<',
                    Direction::Down => 'v',
                    Direction::Right => '>',
                });
            }
            println!("{}", line);
        }
        println!();
    }

    pub fn blizzard_loop_len(&self) -> Minutes {
        self.blizzard_loop_len
    }
}

impl FromStr for BlizzardMap {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let map = Map2D::from_str(s)?;
        let start_pos = map.row(0)
            .iter()
            .position(|t| t == &b'.')
            .map(|x| Point2D(x as isize, 0))
            .context("No start position found.")?;
        let last_row_i = map.height() - 1;
        let target_pos = map.row(last_row_i)
            .iter()
            .position(|t| t == &b'.')
            .map(|x| Point2D(x as isize, last_row_i as isize))
            .context("No target position found.")?;

        let blizzards = map.iter_with_points()
            .flat_map(|(p, tile)| {
                let dir = match *tile {
                    b'^' => Direction::Up,
                    b'>' => Direction::Right,
                    b'v' => Direction::Down,
                    b'<' => Direction::Left,
                    _ => return None,
                };
                Some(Blizzard { origin: p, dir })
            })
            .collect();

        let blizzard_loop_len = lcm(map.width(), map.height()) as Minutes;
        println!("Blizzards loop each {} step/s ({} and {})", blizzard_loop_len, map.width(), map.height());

        Ok(Self { map, start_pos, target_pos, blizzards, blizzard_loop_len })
    }
}

#[derive(Debug, Clone)]
struct Blizzard {
    origin: Point2D,
    dir: Direction,
}

impl Blizzard {
    pub fn position_after(&self, time: Minutes, map_size: Point2D) -> Point2D {
        let rel_pos = self.origin - Point2D(1, 1);
        let adjusted_size = map_size - Point2D(2, 2);

        let pos = rel_pos + (self.dir.as_point() * time);
        Point2D(wrap_val(pos.0, adjusted_size.0), wrap_val(pos.1, adjusted_size.1))
            + Point2D(1, 1)
    }

    pub fn after(&self, time: Minutes, map_size: Point2D) -> Self {
        let position = self.position_after(time, map_size);
        Self { origin: position, dir: self.dir }
    }
}

struct Breadcrumb {
    pos: Point2D,
    time: Minutes,
    previous: Option<Rc<Breadcrumb>>,
}

impl Breadcrumb {
    pub fn new(pos: Point2D, time: Minutes) -> Self {
        Self { pos, time, previous: None }
    }

    pub fn as_priority_with_target(self, target: Point2D) -> KeyedOrd<Breadcrumb, Reverse<isize>> {
        let distance = self.pos.manhattan_distance(target);
        let ord = Reverse(distance as isize + self.time);
        KeyedOrd::new(self, ord)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ExploredPoint {
    point: Point2D,
    time: Minutes,
}
