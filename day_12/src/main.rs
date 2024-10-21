use std::str::FromStr;
use pathfinding::prelude::astar;
use xmas::{direction::DIRECTIONS, display_result, map2d::{Map2D, ParseMapError}, point2d::Point2D};

struct NavigationMap {
    start: Point2D,
    destination: Point2D,
    map: Map2D,
}

impl FromStr for NavigationMap {
    type Err = ParseMapError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let map = Map2D::from_str(s)?;
        Ok(Self::new(map))
    }
}

impl NavigationMap {
    pub fn new(mut map: Map2D) -> Self {
        let start = map.iter_points()
            .find(|&p| map.get_tile(p).is_some_and(|&t| t == b'S'))
            .expect("No starting point found!");
        *map.get_tile_mut(start).unwrap() = b'a';

        let destination = map.iter_points()
            .find(|&p| map.get_tile(p).is_some_and(|&t| t == b'E'))
            .expect("No destination point found!");
        *map.get_tile_mut(destination).unwrap() = b'z';

        Self {
            start,
            destination,
            map,
        }
    }

    pub fn find_path(&self) -> Option<(Vec<Point2D>, usize)> {
        self.find_path_from(self.start)
    }

    pub fn find_path_from(&self, start: Point2D) -> Option<(Vec<Point2D>, usize)> {
        const MOVE_COST: usize = 1;
        let destination = self.destination;
        astar(
            &start,
            |&from| self.find_navigatable_tiles(from).map(|p| (p, MOVE_COST)),
            |cur| cur.manhattan_distance(destination),
            |&cur| cur == destination,
        )
    }

    pub fn lowest_points(&self) -> impl Iterator<Item = Point2D> + '_ {
        self.map.iter_points()
            .filter(|&p| self.map.get_tile(p).is_some_and(|&t| t == b'a'))
    }

    fn find_navigatable_tiles(&self, from: Point2D) -> impl Iterator<Item = Point2D> + '_ {
        const MAX_CLIMB_HEIGHT: u8 = 1;
        let cur_height = *self.map.get_tile(from).expect("from must be inside the map");
        let target_height = cur_height + MAX_CLIMB_HEIGHT;

        DIRECTIONS.iter()
            .map(move |d| from + d.as_point())
            .filter(move |&p| self.map.get_tile(p).is_some_and(|&tile_height| tile_height <= target_height))
    }
}

fn main() {
    part_1();
    part_2();
}

fn part_1() {
    let input = std::fs::read_to_string("./input.txt").expect("Error reading input file.");
    let map = NavigationMap::from_str(&input).unwrap();

    let (_, cost) = map.find_path().unwrap();
    println!("Shortest cost: {cost}");

    display_result(&cost);
}

fn part_2() {
    let input = std::fs::read_to_string("./input.txt").expect("Error reading input file.");
    let map = NavigationMap::from_str(&input).unwrap();

    let (start, _, cost) = map.lowest_points()
        .flat_map(|start| map.find_path_from(start).map(|(path, cost)| (start, path, cost)))
        .min_by_key(|(_, path, _)| path.len())
        .unwrap();

    println!("Shortest found at: {start}, cost: {cost}");

    display_result(&cost);
}
