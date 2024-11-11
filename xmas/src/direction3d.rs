use crate::point3d::Point3D;
use Direction3D::*;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Direction3D {
    Up,
    Left,
    Down,
    Right,
    Front,
    Back,
}


pub const DIRECTIONS_3D: [Direction3D; 6] = [Up, Left, Down, Right, Front, Back];

impl Direction3D {
    pub fn as_point(&self) -> Point3D {
        match self {
            Up => Point3D(0, -1, 0),
            Left => Point3D(1, 0, 0),
            Down => Point3D(0, 1, 0),
            Right => Point3D(-1, 0, 0),
            Front => Point3D(0, 0, 1),
            Back => Point3D(0, 0, -1),
        }
    }

    pub fn combined(&self, other: Self) -> Point3D {
        self.as_point() + other.as_point()
    }
}

impl From<Direction3D> for Point3D {
    fn from(value: Direction3D) -> Self {
        value.as_point()
    }
}
