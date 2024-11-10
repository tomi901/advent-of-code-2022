use std::{cmp::max, collections::HashSet, str::FromStr};

use anyhow::{self, Context};
use xmas::{display_result, point2d::Point2D};

#[derive(Debug, Clone)]
struct RockShape {
    tiles: Vec<bool>,
    width: usize,
    height: usize,
}

impl RockShape {
    pub fn default_shapes() -> Vec<RockShape> {
        const PATTERNS: [&str; 5] = [
            "####",
            ".#.\n###\n.#.",
            "..#\n..#\n###",
            "#\n#\n#\n#",
            "##\n##",
        ];

        PATTERNS.into_iter()
            .map(Self::from_str)
            .collect::<Result<_, _>>()
            .unwrap()
    }

    pub fn local_points(&self) -> impl Iterator<Item = Point2D> + '_ {
        self.tiles.iter()
            .enumerate()
            .filter(|&(_, b)| *b)
            .map(|(i, _)| Point2D((i % self.width) as isize , -((i / self.width) as isize)))
    }
}

impl FromStr for RockShape {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();

        let first_line = lines.next().context("Empty string")?.trim_end();
        let width = first_line.len();
        let mut tiles: Vec<_> = first_line.bytes().map(|b| b == b'#').collect();

        let mut height = 1;
        while let Some(line) = lines.next() {
            tiles.extend(line.bytes().map(|b| b == b'#'));
            height += 1;
        }

        Ok(Self {
            tiles,
            width,
            height,
        })
    }
}

struct RockInstance<'a> {
    shape: &'a RockShape,
    position: Point2D,
}

impl<'a> RockInstance<'a> {
    pub fn move_towards(&self, direction: Point2D) -> Self {
        Self {
            position: self.position + direction,
            shape: self.shape,
        }
    }

    pub fn upper_bound(&self) -> isize {
        self.position.1
    }

    pub fn lower_bound(&self) -> isize {
        self.position.1 - self.shape.height as isize + 1
    }

    pub fn left_bound(&self) -> isize {
        self.position.0
    }

    pub fn right_bound(&self) -> isize {
        self.position.0 + self.shape.width as isize - 1
    }

    pub fn check_collision(&self, map: &RockFormation) -> bool {
        if self.lower_bound() <= 0 ||
            self.left_bound() <= 0 ||
            self.right_bound() >= map.right_wall()
        {
            return true;
        }

        self.world_points()
            .any(|p| map.tiles.contains(&p))
    }

    pub fn local_points(&self) -> impl Iterator<Item = Point2D> + '_ {
        self.shape.local_points()
    }

    pub fn world_points(&self) -> impl Iterator<Item = Point2D> + '_ {
        self.local_points()
            .map(|p| p + self.position)
    }
}

impl<'a> Clone for RockInstance<'a> {
    fn clone(&self) -> Self {
        Self { shape: self.shape, position: self.position.clone() }
    }
}

#[derive(Debug, Clone, Copy)]
enum StreamDirection {
    Left,
    Right,
}

impl StreamDirection {
    pub fn pattern_from_str(s: &str) -> Result<Vec<Self>, anyhow::Error> {
        s.trim().bytes()
            .map(|b| match b {
                b'<' => Ok(Self::Left),
                b'>' => Ok(Self::Right),
                _ => Err(anyhow::anyhow!("Invalid direction: {:?}", char::from_u32(b as u32))),
            })
            .collect()
    }

    pub fn as_movement(&self) -> Point2D {
        match self {
            Self::Left => Point2D(-1, 0),
            Self::Right => Point2D(1, 0),
        }
    }
}

#[derive(Debug, Clone)]
struct RockFormation {
    tiles: HashSet<Point2D>,
    height: usize,

    shapes: Vec<RockShape>,
    use_shape: usize,

    stream_pattern: Vec<StreamDirection>,
    use_stream: usize,
}

impl RockFormation {
    fn new(stream_pattern: Vec<StreamDirection>) -> Self {
        let shapes = RockShape::default_shapes();
        Self {
            tiles: Default::default(),
            height: 0,
            shapes,
            use_shape: 0,
            stream_pattern,
            use_stream: 0,
        }
    }

    fn throw_many_rocks(&mut self, amount: usize) {
        for _ in 0..amount {
            self.throw_rock();
        }
    }

    fn throw_rock(&mut self) {
        const LEFT_MARGIN: isize = 3;
        const LOWER_MARGIN: isize = 3;

        let shape = &self.shapes[self.use_shape];
        let position = Point2D(LEFT_MARGIN, self.height as isize + LOWER_MARGIN + shape.height as isize);
        let mut rock = RockInstance { shape, position };
        
        // println!("Spawned rock[{}] @ {}", self.use_shape, position);
        loop {
            let direction = self.stream_pattern[self.use_stream];
            self.use_stream = (self.use_stream + 1) % self.stream_pattern.len();

            // let dir = match direction {
            //     StreamDirection::Left => ">",
            //     StreamDirection::Right => "<",
            // };
            // print!("Moved rock towards {dir}, result: ");

            let stream_moved_rock = rock.move_towards(direction.as_movement());
            if !stream_moved_rock.check_collision(self) {
                rock = stream_moved_rock;
                // println!("Moved")
            } else {
                // println!("Blocked")
            }

            let gravity_moved_rock = rock.move_towards(Point2D(0, -1));
            if gravity_moved_rock.check_collision(self) {
                // println!("Ground hit!");
                break;
            } else {
                rock = gravity_moved_rock;
                // println!("Rock falls 1 unit");
            }
        }

        self.height = max(self.height, rock.upper_bound() as usize);
        self.tiles.extend(rock.world_points());
        self.use_shape = (self.use_shape + 1) % self.shapes.len();
    }

    pub fn right_wall(&self) -> isize { 
        8 // Hardcoded
    }

    pub fn display_debug(&self) {
        for y in (1..=(self.height as isize)).rev() {
            print!("|");
            for x in 1..self.right_wall() {
                let point = Point2D(x, y);
                if self.tiles.contains(&point) {
                    print!("#");
                } else {
                    print!(".");
                }
            }
            println!("|");
        }

        print!("+");
        for _ in 1..self.right_wall() {
            print!("-");
        }
        println!("+");
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

    let pattern = StreamDirection::pattern_from_str(&input)?;
    let mut formation = RockFormation::new(pattern);

    formation.throw_many_rocks(2022);
    // println!("Map:");
    // formation.display_debug();
    let result = formation.height;

    display_result(&result);
    Ok(())
}

fn part_2() -> anyhow::Result<()> {
    println!("Part 2:");

    Ok(())
}
