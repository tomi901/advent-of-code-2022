use crate::point2d::Point2D;
use enum_map::Enum;
use Direction::*;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Enum)]
pub enum Direction {
    Up,
    Left,
    Down,
    Right,
}

pub const DIRECTIONS: [Direction; 4] = [Up, Left, Down, Right];
pub const DIRECTIONS_8: [Point2D; 8] = [
    Point2D(0, -1),
    Point2D(1, -1),
    Point2D(1, 0),
    Point2D(1, 1),
    Point2D(0, 1),
    Point2D(-1, 1),
    Point2D(-1, 0),
    Point2D(-1, -1),
];

impl Direction {
    pub fn as_point(&self) -> Point2D {
        match self {
            Up => Point2D(0, -1),
            Left => Point2D(-1, 0),
            Down => Point2D(0, 1),
            Right => Point2D(1, 0),
        }
    }

    pub fn combined(&self, other: Self) -> Point2D {
        self.as_point() + other.as_point()
    }

    pub fn turn(&self, relative_turn: Self) -> Self {
        let final_value = (self.value() + relative_turn.value()) as usize % DIRECTIONS.len();
        DIRECTIONS[final_value]
    }

    pub fn turn_rotation(&self, rot: QuarterRotation) -> Self {
        let final_value = (self.value() + rot.value()) as usize % DIRECTIONS.len();
        DIRECTIONS[final_value]
    }

    fn value(&self) -> u8 {
        match self {
            Up => 0,
            Left => 1,
            Down => 2,
            Right => 3,
        }
    }
}

impl From<Direction> for Point2D {
    fn from(value: Direction) -> Self {
        value.as_point()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuarterRotation {
    None,
    Right,
    TurnAround,
    Left,
}

impl QuarterRotation {
    fn value(&self) -> u8 {
        match self {
            Self::None => 0,
            Self::Left => 1,
            Self::TurnAround => 2,
            Self::Right => 3,
        }
    }
}
