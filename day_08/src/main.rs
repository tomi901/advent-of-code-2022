use std::{fmt::Display, ops, str::FromStr};
use cli_clipboard::{ClipboardContext, ClipboardProvider};

#[derive(Clone, Copy, Debug)]
struct Point2D(pub isize, pub isize);

impl Display for Point2D {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.0, self.1)
    }
}

impl ops::Add<Point2D> for Point2D {
    type Output = Point2D;

    fn add(self, rhs: Point2D) -> Self::Output {
        Point2D(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl ops::AddAssign<Point2D> for Point2D {
    fn add_assign(&mut self, rhs: Point2D) {
        *self = *self + rhs
    }
}

struct Map2D {
    map: Vec<u8>,
    width: usize,
    height: usize,
}

impl Map2D {
    pub fn parse_and_add_row(&mut self, line: &str) -> Result<(), &'static str> {
        if line.len() != self.width {
            return Err("Invalid row length!");
        }
        self.map.extend(line.bytes());
        self.height += 1;
        Ok(())
    }

    pub fn is_inside(&self, point: Point2D) -> bool {
        point.0 >= 0 && point.1 >= 0 && (point.0 as usize) < self.width && (point.1 as usize) < self.height
    }

    pub fn get_tile(&self, point: Point2D) -> Option<&u8> {
        self.get_index(point).and_then(|i| self.map.get(i))
    }

    pub fn get_tile_mut(&mut self, point: Point2D) -> Option<&mut u8> {
        self.get_index(point).and_then(|i| self.map.get_mut(i))
    }

    pub fn get_index(&self, point: Point2D) -> Option<usize> {
        self.is_inside(point).then(|| point.0 as usize + (point.1 as usize * self.height))
    }

    pub fn points_iter(&self) -> impl Iterator<Item = Point2D> + '_ {
        (0..(self.height as isize))
            .flat_map(|y| (0..(self.width as isize)).map(move |x| Point2D(x, y)))
    }

    pub fn iter(&self) -> impl Iterator<Item = &u8> + '_ {
        self.map.iter()
    }
}

impl FromStr for Map2D {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err("Can't parse an empty string to a Map2D");
        }

        let map = Vec::with_capacity(s.len());
        let mut lines = s.lines();
        
        let first_line = lines.next().unwrap();
        let width = first_line.len();

        let mut map = Self { map, width, height: 0 };
        map.parse_and_add_row(first_line)?;
        for line in lines {
            map.parse_and_add_row(line)?;
        }

        Ok(map)
    }
}

struct ForestMap {
    map: Map2D,
}

impl FromStr for ForestMap {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let map = Map2D::from_str(s)?;
        Ok(ForestMap { map })
    }
}

impl ForestMap {
    pub fn is_visible_towards(&self, from: Point2D, towards: Point2D) -> bool {
        let target_height = *self.map.get_tile(from).unwrap();
        let mut checking_at = from;
        loop {
            checking_at += towards;
            let cur_height = match self.map.get_tile(checking_at) {
                Some(h) => *h,
                None => return true,
            };

            if cur_height >= target_height {
                return false;
            }
        }
    }

    pub fn is_visible(&self, from: Point2D) -> bool {
        const DIRECTIONS: [Point2D; 4] = [Point2D(0, 1), Point2D(1, 0), Point2D(0, -1), Point2D(-1, 0)];
        DIRECTIONS.iter().any(|&dir| self.is_visible_towards(from, dir))
    }

    pub fn visible_count(&self) -> usize {
        self.map.points_iter().filter(|&p| self.is_visible(p)).count()
    }

    pub fn get_visible_trees(&self, from: Point2D, towards: Point2D) -> usize {
        let target_height = *self.map.get_tile(from).unwrap();
        let mut count = 0;
        let mut checking_at = from;
        loop {
            checking_at += towards;
            let cur_height = match self.map.get_tile(checking_at) {
                Some(h) => *h,
                None => return count,
            };

            count += 1;
            if cur_height >= target_height {
                return count;
            }
        }
    }

    pub fn scenic_score_at(&self, from: Point2D) -> Option<usize> {
        const DIRECTIONS: [Point2D; 4] = [Point2D(0, 1), Point2D(1, 0), Point2D(0, -1), Point2D(-1, 0)];
        DIRECTIONS.iter()
            .map(|&dir| self.get_visible_trees(from, dir))
            .reduce(ops::Mul::mul)
    }

    pub fn find_best_scenic_score(&self) -> Option<usize> {
        self.map.points_iter().flat_map(|p| self.scenic_score_at(p)).max()
    }
}

fn main() {
    // part_1();
    part_2();
}

fn part_1() {
    let input = std::fs::read_to_string("./input.txt").expect("Error reading input file.");
    let map = ForestMap::from_str(&input).expect("Error parsing map");
    let result = map.visible_count();

    display_result(&result);
}

fn part_2() {
    let input = std::fs::read_to_string("./input.txt").expect("Error reading input file.");
    let map = ForestMap::from_str(&input).expect("Error parsing map");
    let result = map.find_best_scenic_score();

    display_result(&result.unwrap());
}

// TODO: Move this to common library crate
fn display_result<T: Display>(result: &T) {
    println!("Result:");
    let str_result = format!("{}", result);
    println!("{}", &str_result);

    let mut clipboard = ClipboardContext::new().unwrap();
    clipboard.set_contents(str_result.clone()).unwrap();
    println!("Copied result to clipboard!");
}
