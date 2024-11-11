use std::{fmt::Display, ops};
use crate::direction::Direction;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Hash)]
pub struct Point2D(pub isize, pub isize);

impl Point2D {
    pub const ZERO: Self = Point2D(0, 0);

    pub fn manhattan_magnitude(&self) -> usize {
        self.0.unsigned_abs() + self.1.unsigned_abs()
    }

    pub fn manhattan_distance(&self, towards: Point2D) -> usize {
        (*self - towards).manhattan_magnitude()
    }

    pub fn sqr_magnitude(&self) -> usize {
        (self.0 * self.0 + self.1 * self.1) as usize
    }

    pub fn max(&self, other: Self) -> Self {
        Point2D(self.0.max(other.0), self.1.max(other.1))
    }

    pub fn min(&self, other: Self) -> Self {
        Point2D(self.0.min(other.0), self.1.min(other.1))
    }

    pub fn try_get_direction(&self) -> Option<(Direction, usize)> {
        match self {
            Point2D(0, 0) => None,
            Point2D(0, y) => Some((if *y > 0 { Direction::Down } else { Direction::Up }, y.unsigned_abs())),
            Point2D(x, 0) => Some((if *x > 0 { Direction::Left } else { Direction::Right }, x.unsigned_abs())),
            _ => None,
        }
    }

    pub fn try_get_direction_towards(&self, target: Point2D) -> Option<(Direction, usize)> {
        (target - *self).try_get_direction()
    }
}

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

impl ops::Sub<Point2D> for Point2D {
    type Output = Point2D;

    fn sub(self, rhs: Point2D) -> Self::Output {
        Point2D(self.0 - rhs.0, self.1 - rhs.1)
    }
}

impl ops::SubAssign<Point2D> for Point2D {
    fn sub_assign(&mut self, rhs: Point2D) {
        *self = *self - rhs
    }
}

impl From<(isize, isize)> for Point2D {
    fn from(value: (isize, isize)) -> Self {
        Self(value.0, value.1)
    }
}

impl From<[isize; 2]> for Point2D {
    fn from(value: [isize; 2]) -> Self {
        Self(value[0], value[1])
    }
}
