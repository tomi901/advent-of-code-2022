use std::{fmt::Display, num::ParseIntError, ops, str::FromStr};

use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Hash)]
pub struct Point3D(pub isize, pub isize, pub isize);

impl Point3D {
    pub const ZERO: Self = Self(0, 0, 0);

    pub fn manhattan_magnitude(&self) -> usize {
        self.0.unsigned_abs() + self.1.unsigned_abs() + self.2.unsigned_abs()
    }

    pub fn manhattan_distance(&self, towards: Self) -> usize {
        (*self - towards).manhattan_magnitude()
    }

    pub fn sqr_magnitude(&self) -> usize {
        (self.0 * self.0 + self.1 * self.1 + self.2 * self.2) as usize
    }

    pub fn max(&self, other: Self) -> Self {
        Self(self.0.max(other.0), self.1.max(other.1), self.2.max(other.2))
    }

    pub fn min(&self, other: Self) -> Self {
        Self(self.0.min(other.0), self.1.min(other.1), self.2.min(other.2))
    }

    fn parse_segment(split: &mut std::str::Split<char>) -> Result<isize, ParsePoint3DError> {
        Ok(split.next()
            .ok_or(ParsePoint3DError::InvalidLength)?
            .trim()
            .parse::<isize>()?)
    }
}

impl Display for Point3D {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.0, self.1, self.2)
    }
}

impl ops::Add<Point3D> for Point3D {
    type Output = Point3D;

    fn add(self, rhs: Point3D) -> Self::Output {
        Self(self.0 + rhs.0, self.1 + rhs.1, self.2 + rhs.2)
    }
}

impl ops::AddAssign<Point3D> for Point3D {
    fn add_assign(&mut self, rhs: Point3D) {
        *self = *self + rhs
    }
}

impl ops::Sub<Point3D> for Point3D {
    type Output = Point3D;

    fn sub(self, rhs: Point3D) -> Self::Output {
        Self(self.0 - rhs.0, self.1 - rhs.1, self.2 - rhs.2)
    }
}

impl ops::SubAssign<Point3D> for Point3D {
    fn sub_assign(&mut self, rhs: Point3D) {
        *self = *self - rhs
    }
}

impl From<(isize, isize, isize)> for Point3D {
    fn from(value: (isize, isize, isize)) -> Self {
        Self(value.0, value.1, value.2)
    }
}

impl From<[isize; 3]> for Point3D {
    fn from(value: [isize; 3]) -> Self {
        Self(value[0], value[1], value[2])
    }
}

impl TryFrom<&[isize]> for Point3D {
    type Error = ParsePoint3DError;
    
    fn try_from(value: &[isize]) -> Result<Self, Self::Error> {
        if value.len() == 3 {
            Ok(Self(value[0], value[1], value[2]))
        } else {
            Err(ParsePoint3DError::InvalidLength)
        }
    }
}

impl FromStr for Point3D {
    type Err = ParsePoint3DError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split(',');
        let x = Self::parse_segment(&mut split)?;
        let y = Self::parse_segment(&mut split)?;
        let z = Self::parse_segment(&mut split)?;
        if split.next().is_some() {
            Err(ParsePoint3DError::InvalidLength)
        } else {
            Ok(Self(x, y, z))
        }
    }
}

#[derive(Debug, Clone, Error)]
pub enum ParsePoint3DError {
    #[error("should have 3 segments for Point3D")]
    InvalidLength,
    #[error("couldn't parse ints")]
    ParseInt(#[from] ParseIntError),
}
