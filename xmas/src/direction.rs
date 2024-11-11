use crate::point2d::Point2D;
use Direction::*;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Direction {
    Up,
    Left,
    Down,
    Right,
}


pub const DIRECTIONS: [Direction; 4] = [Up, Left, Down, Right];

impl Direction {
    pub fn as_point(&self) -> Point2D {
        match self {
            Up => Point2D(0, -1),
            Left => Point2D(1, 0),
            Down => Point2D(0, 1),
            Right => Point2D(-1, 0),
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
