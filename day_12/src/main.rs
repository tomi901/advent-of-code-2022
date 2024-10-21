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
        const MOVE_COST: usize = 1;
        let destination = self.destination;
        astar(
            &self.start,
            |&from| self.find_navigatable_tiles(from).map(|p| (p, MOVE_COST)),
            |cur| cur.manhattan_distance(destination),
            |&cur| cur == destination,
        )
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

    let (path, _) = map.find_path().unwrap();
    let result = path.len() - 1;

    display_result(&result);
}

fn part_2() {
    
}
