use core::str;
use std::{fmt::Display, str::FromStr};
use thiserror::Error;

use crate::point2d::Point2D;

#[derive(Debug, PartialEq)]
pub struct Map2D {
    map: Vec<u8>,
    width: usize,
    height: usize,
}

impl Map2D {
    pub fn new_filled(size: Point2D, tile: u8) -> Self {
        let width = size.0 as usize;
        let height = size.1 as usize;
        let map = vec![tile; width * height];
        Self {
            map,
            width,
            height,
        }
    }

    pub fn new_with_default_tiles(size: Point2D) -> Self {
        Self::new_filled(size, Default::default())
    }

    pub fn parse_and_add_row(&mut self, line: &str) -> Result<(), ParseMapError> {
        if line.len() != self.width {
            return Err(ParseMapError::InconsistentRowSize { current: line.len(), expected: self.width });
        }
        self.map.extend(line.bytes());
        self.height += 1;
        Ok(())
    }


    pub fn is_inside(&self, point: Point2D) -> bool {
        point.0 >= 0 && point.1 >= 0 && (point.0 as usize) < self.width && (point.1 as usize) < self.height
    }

    pub fn set_tile(&mut self, point: Point2D, tile: u8) -> bool {
        if let Some(index) = self.get_index(point) {
            self.map[index] = tile;
            true
        } else {
            false
        }
    }

    pub fn get_tile(&self, point: Point2D) -> Option<&u8> {
        self.get_index(point).and_then(|i| self.map.get(i))
    }

    pub fn get_tile_mut(&mut self, point: Point2D) -> Option<&mut u8> {
        self.get_index(point).and_then(|i| self.map.get_mut(i))
    }

    pub fn get_index(&self, point: Point2D) -> Option<usize> {
        self.is_inside(point).then(|| point.0 as usize + (point.1 as usize * self.width))
    }

    pub fn iter_points(&self) -> impl Iterator<Item = Point2D> + '_ {
        (0..(self.height as isize))
            .flat_map(|y| (0..(self.width as isize)).map(move |x| Point2D(x, y)))
    }

    pub fn iter(&self) -> impl Iterator<Item = &u8> + '_ {
        self.map.iter()
    }

    pub fn row(&self, index: usize) -> &[u8] {
        let start = index * self.width;
        let end = start + self.width;
        &self.map[start..end]
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ParseMapError {
    #[error("Can't parse an empty string to a Map2D")]
    EmptyString,
    #[error("Inconsistent row size. Current: {current} Expected: {expected}")]
    InconsistentRowSize { current: usize, expected: usize },
}

impl FromStr for Map2D {
    type Err = ParseMapError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(ParseMapError::EmptyString);
        }

        let map = Vec::with_capacity(s.len());
        let mut lines = s.lines();
        
        let first_line = lines.next().unwrap();
        let width = first_line.len();

        let mut map = Self { map, width, height: 0 };
        map.parse_and_add_row(first_line)?;
        for line in lines {
            map.parse_and_add_row(line)?;
        }

        Ok(map)
    }
}

impl Display for Map2D {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let lines = (0..self.height).map(|y| {
            let from = y * self.width;
            &self.map[from..(from + self.width)]
        });
        for line in lines {
            writeln!(f, "{}", String::from_utf8_lossy(line))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn builds_map_correctly() {
        let map = Map2D::new_with_default_tiles(Point2D(20, 10));

        assert_eq!(map.width, 20);
        assert_eq!(map.height, 10);
        assert_eq!(map.map.len(), 20 * 10);
    }

    #[rstest]
    #[case(Point2D(20, 10), Point2D(0, 0), Some(0))]
    #[case(Point2D(20, 10), Point2D(4, 5), Some(104))]
    #[case(Point2D(20, 10), Point2D(-1, 0), None)]
    #[case(Point2D(20, 10), Point2D(0, -1), None)]
    #[case(Point2D(20, 10), Point2D(20, 0), None)]
    #[case(Point2D(20, 10), Point2D(0, 10), None)]
    fn index_is_equal_to_expected(
        #[case] map_size: Point2D,
        #[case] point: Point2D,
        #[case] expected: Option<usize>,
    ) {
        let map = Map2D::new_with_default_tiles(map_size);
        let index = map.get_index(point);

        assert_eq!(index, expected);
    }

    #[test]
    fn parses_map_correctly() {
        const MAP: &str = concat!(
            "0123\n",
            "4567\n",
            "89AB\n",
        );

        let map = Map2D::from_str(MAP).unwrap();
        assert_eq!(map.width, 4);
        assert_eq!(map.height, 3);
    }

    #[test]
    fn parse_map_returns_empty_error() {
        let result = Map2D::from_str("");
        assert_eq!(result, Err(ParseMapError::EmptyString));
    }

    #[test]
    fn parse_map_returns_inconsistent_lines_error() {
        const MAP: &str = concat!(
            "0123\n",
            "457\n",
            "89AB\n",
        );

        let result = Map2D::from_str(MAP);
        assert_eq!(result, Err(ParseMapError::InconsistentRowSize { current: 3, expected: 4 }))
    }
}
