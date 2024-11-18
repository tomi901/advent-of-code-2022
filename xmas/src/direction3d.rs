use crate::point3d::Point3D;
use enum_map::Enum;
use Direction3D::*;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Enum)]
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

    pub fn inverse(&self) -> Direction3D {
        match self {
            Up => Down,
            Left => Right,
            Down => Up,
            Right => Left,
            Front => Back,
            Back => Front,
        }
    }
}

impl From<Direction3D> for Point3D {
    fn from(value: Direction3D) -> Self {
        value.as_point()
    }
}

impl TryFrom<Point3D> for Direction3D {
    type Error = &'static str;

    fn try_from(value: Point3D) -> Result<Self, Self::Error> {
        Ok(match value {
            Point3D(x, 0, 0) if x != 0 => if x > 0 { Direction3D::Right } else { Direction3D::Left },
            Point3D(0, y, 0) if y != 0 => if y > 0 { Direction3D::Up } else { Direction3D::Down },
            Point3D(0, 0, z) if z != 0 => if z > 0 { Direction3D::Front } else { Direction3D::Back },
            _ => return Err("Cannot convert vector to direction"),
        })
    }
}
