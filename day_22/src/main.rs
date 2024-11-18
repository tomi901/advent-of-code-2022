use std::str::FromStr;

use anyhow::{self, Context};
use enum_map::EnumMap;
use xmas::{direction::{Direction, QuarterRotation, DIRECTIONS}, direction3d::Direction3D, display_result, map2d::Map2D, point2d::Point2D};
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
    let input = std::fs::read_to_string("./input.txt").context("Error reading input file.")?;

    let (map, movements) = input.split_once("\n\n").context("No movements found")?;
    let map = PasswordMap::from_str(map)?;
    let mut cube_map = PasswordCubeMap::new(map, Point2D(4, 4))?;
    let movements = Movement::many_from_str(movements)?;

    let path = cube_map.travel(&movements);
    for (point, dir) in path {
        let tile = match dir {
            Direction::Up => b'^',
            Direction::Left => b'<',
            Direction::Down => b'v',
            Direction::Right => b'>',
        };

        cube_map.unfolded.map.set_tile(point, tile);
    }

    println!("Map:\n{}", cube_map.unfolded.map);

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

struct PasswordCubeMap {
    unfolded: PasswordMap,
    face_size: Point2D,
    face_map: EnumMap<Direction3D, Face>,
}

impl PasswordCubeMap {
    pub fn new(unfolded: PasswordMap, face_size: Point2D) -> anyhow::Result<Self> {
        let mut face_map_partial = EnumMap::<Direction3D, Option<Face>>::default();
        
        let mut face_queue = Vec::new();
        face_queue.push(Face {
            origin: unfolded.start,
            orientation: Orientation::default(),
        });

        while let Some(new_face) = face_queue.pop() {
            // dbg!((&new_face.orientation, new_face.origin));
            let face_side = new_face.orientation.normal;
            if face_map_partial[face_side].is_some() {
                continue;
            }

            for (dir, orientation) in DIRECTIONS.iter()
                .map(|&dir| (dir, new_face.orientation.move_towards(dir)))
                .filter(|(_, orientation)| face_map_partial[orientation.normal].is_none())
            {
                let new_point = new_face.origin + dir.as_point().scale(face_size);
                let tile = unfolded.map.get_tile(new_point);
                if tile.is_none() || tile.is_some_and(|&t| t == b' ') {
                    continue;
                }

                let face_for_queue = Face {
                    origin: new_point,
                    orientation,
                };
                face_queue.push(face_for_queue);
            }
            face_map_partial[face_side] = Some(new_face);
        }

        let face_map = face_map_partial.into_iter()
            .map(|(dir, face)| (dir, face.unwrap()))
            .collect();

        Ok(Self {
            unfolded,
            face_size,
            face_map,
        })
    }
    
    fn travel(&self, movements: &[Movement]) -> Vec<(Point2D, Direction)> {
        let mut pos = self.unfolded.start;
        let mut dir = Direction::Right;
        let mut orientation = Orientation::default();

        let mut path = Vec::new();

        for movement in movements {
            let steps = match movement {
                Movement::Move(steps) => *steps,
                Movement::Turn(direction) => {
                    dir = dir.turn(*direction);
                    continue;
                },
            };

            println!("Moving from {pos} {steps} steps with direction {dir:?}");

            for _ in 0..steps {
                path.push((pos, dir));
                let would_move_to = pos + dir.as_point();
                let face = &self.face_map[orientation.normal];
                let would_move_to_relative = would_move_to - face.origin;

                let move_to_face = match would_move_to_relative {
                    Point2D(x, _) if x < 0 => Some(Direction::Left),
                    Point2D(x, _) if x >= self.face_size.0 => Some(Direction::Right),
                    Point2D(_, y) if y < 0 => Some(Direction::Up),
                    Point2D(_, y) if y >= self.face_size.1 => Some(Direction::Down),
                    _ => None,
                };

                let (target_pos, target_dir, target_orientation) = match move_to_face {
                    Some(move_towards_face) => {
                        let new_orientation = orientation.move_towards(move_towards_face);
                        println!("Moving to {:?} with orientation {:?}", move_towards_face, new_orientation);

                        let new_face = &self.face_map[new_orientation.normal];
                        let turn = new_face.orientation.get_rotation(&new_orientation).unwrap();
                        let wrapped_pos = Point2D(
                            wrap_value(would_move_to_relative.0, self.face_size.0),
                            wrap_value(would_move_to_relative.1, self.face_size.1),
                        );
                        let adjusted_pos = wrapped_pos + new_face.origin;

                        println!("Translated to: {}", adjusted_pos);
                        (adjusted_pos, dir.turn_rotation(turn), new_orientation)
                    },
                    None => (would_move_to, dir, orientation.clone()),
                };

                if self.unfolded.map.get_tile(would_move_to) == Some(&b'#') {
                    break;
                }

                // println!("{target_pos}");
                pos = target_pos;
                dir = target_dir;
                orientation = target_orientation;
            }
        }

        path
    }
}

#[derive(Clone, Debug, Default)]
struct Face {
    origin: Point2D,
    orientation: Orientation,
}

#[derive(Debug, Clone)]
struct Orientation {
    normal: Direction3D,
    up_tangent: Direction3D,
    right_tangent: Direction3D,
}

impl Orientation {
    fn move_towards(&self, direction_2d: Direction) -> Self {
        match direction_2d {
            Direction::Up => Self {
                normal: self.up_tangent,
                up_tangent: self.normal.inverse(),
                right_tangent: self.right_tangent,
            },
            Direction::Left => Self {
                normal: self.right_tangent.inverse(),
                up_tangent: self.up_tangent,
                right_tangent: self.normal,
            },
            Direction::Down => Self {
                normal: self.up_tangent.inverse(),
                up_tangent: self.normal,
                right_tangent: self.right_tangent,
            },
            Direction::Right => Self {
                normal: self.right_tangent,
                up_tangent: self.up_tangent,
                right_tangent: self.normal.inverse(),
            },
        }
    }
    
    fn get_rotation(&self, new_orientation: &Orientation) -> Option<QuarterRotation> {
        if self.normal != new_orientation.normal {
            return None;
        }

        // dbg!((self, new_orientation));
        Some(match new_orientation.up_tangent {
            up if up == self.up_tangent => QuarterRotation::None,
            up if up == self.right_tangent => QuarterRotation::Right,
            up if up == self.up_tangent.inverse() => QuarterRotation::TurnAround,
            up if up == self.right_tangent.inverse() => QuarterRotation::Left,
            _ => unreachable!(),
        })
    }
}

impl Default for Orientation {
    fn default() -> Self {
        Self {
            normal: Direction3D::Front,
            up_tangent: Direction3D::Up,
            right_tangent: Direction3D::Right,
        }
    }
}

fn wrap_value(val: isize, limit: isize) -> isize {
    match val % limit {
        res if res >= 0 => res,
        res => res + limit,
    }
}
