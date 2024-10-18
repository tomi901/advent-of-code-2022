use std::{fmt::Display, ops};


#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Hash)]
pub struct Point2D(pub isize, pub isize);

impl Point2D {
    pub const ZERO: Self = Point2D(0, 0);
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
