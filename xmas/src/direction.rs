use crate::point2d::Point2D;
use Direction::*;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Direction {
    North,
    East,
    South,
    West,
}


pub const DIRECTIONS: [Direction; 4] = [North, East, South, West];

impl Direction {
    pub fn as_point(&self) -> Point2D {
        match self {
            North => Point2D(0, -1),
            East => Point2D(1, 0),
            South => Point2D(0, 1),
            West => Point2D(-1, 0),
        }
    }

    pub fn combined(&self, other: Self) -> Point2D {
        self.as_point() + other.as_point()
    }
}

impl From<Direction> for Point2D {
    fn from(value: Direction) -> Self {
        value.as_point()
    }
}
