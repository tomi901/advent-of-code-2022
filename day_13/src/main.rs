use std::{cmp::Ordering, fmt::Display, str::FromStr};

use anyhow::{anyhow, Context};
use xmas::display_result;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Packet {
    Value(i64),
    List(Vec<Self>),
}

impl Packet {
    pub fn single_value_list(value: i64) -> Self {
        Self::List(vec![Self::Value(value)])
    }
}

impl Display for Packet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Packet::Value(i) => write!(f, "{}", i),
            Packet::List(vec) => {
                let mut first = true;
                write!(f, "[")?;
                for value in vec {
                    if !first {
                        write!(f, ",")?;
                    }
                    write!(f, "{}", value)?;
                    first = false;
                }
                write!(f, "]")
            },
        }
    }
}

impl FromStr for Packet {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let json: serde_json::Value = serde_json::from_str(s)?;
        json.try_into()
    }
}

impl TryFrom<serde_json::Value> for Packet {
    type Error = anyhow::Error;

    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        Ok(match value {
            serde_json::Value::Number(number) => Self::Value(
                number.as_i64().context("Cannot transform value to int")?
            ),
            serde_json::Value::Array(vec) => {
                let values = vec.into_iter()
                    .map(Self::try_from)
                    .collect::<Result<Vec<_>, _>>()?;
                Self::List(values)
            },
            _ => Err(anyhow!("Unknown case: {:?}", value))?,
        })
    }
}

impl PartialOrd for Packet {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Packet {
    fn cmp(&self, other: &Self) -> Ordering {
        // println!(" - Compare {self} vs {other}");
        match (self, other) {
            (Packet::Value(left), Packet::Value(right)) => left.cmp(right),
            (Packet::Value(left), Packet::List(_)) => Self::single_value_list(*left).cmp(other),
            (Packet::List(_), Packet::Value(right)) => self.cmp(&Self::single_value_list(*right)),
            (Packet::List(left), Packet::List(right)) => {
                let mut left_iter = left.iter();
                let mut right_iter = right.iter();
                loop {
                    let result = match (left_iter.next(), right_iter.next()) {
                        (None, None) => return Ordering::Equal,
                        (None, Some(_)) => return Ordering::Less,
                        (Some(_), None) => return Ordering::Greater,
                        (Some(l), Some(r)) => l.cmp(r),
                    };

                    if result != Ordering::Equal {
                        return result;
                    }
                }
            },
        }
        /* This allocates less, but it fails on some cases
        match self {
            Packet::Value(left) => match other {
                Packet::Value(right) => left.cmp(right),
                Packet::List(vec) => {
                    if let Some(first) = vec.first() {
                        self.cmp(first)
                    } else {
                        Ordering::Greater
                    }
                },
            },
            Packet::List(left) => match other {
                Packet::Value(_) => {
                    if let Some(first) = left.first() {
                        first.cmp(other)
                    } else {
                        Ordering::Less
                    }
                },
                Packet::List(right) => {
                    let mut left_iter = left.iter();
                    let mut right_iter = right.iter();
                    loop {
                        let result = match (left_iter.next(), right_iter.next()) {
                            (None, None) => return Ordering::Equal,
                            (None, Some(_)) => return Ordering::Less,
                            (Some(_), None) => return Ordering::Greater,
                            (Some(l), Some(r)) => l.cmp(r),
                        };

                        if result != Ordering::Equal {
                            return result;
                        }
                    }
                },
            },
        }
        */
    }
}

fn main() -> anyhow::Result<()> {
    part_1()?;
    part_2()?;
    Ok(())
}

fn part_1() -> anyhow::Result<()> {
    println!("Part 1:");
    let input = std::fs::read_to_string("./input.txt").context("Error reading input file.")?;

    let mut lines = input.lines();
    let mut result = 0;
    let mut index = 0;
    loop {
        index += 1;

        let (left, _left_line) = match lines.next() {
            Some(line) if line.is_empty() => break,
            Some(line) => (Packet::from_str(line)?, line),
            None => break,
        };
        let right_line = lines.next().ok_or_else(|| anyhow!("No line found for right value"))?;
        let right = right_line.parse::<Packet>()?;

        // println!();
        // println!("Processing index {index}");
        // println!("Comparing:");
        // println!("{_left_line}");
        // println!("{right_line}");

        let comparison = left.cmp(&right);
        let is_in_order = comparison.is_le();
        // println!("Result: {comparison:?} = {is_in_order}");
        if is_in_order {
            // println!();
            // println!("Adding: {}", index);
            // println!("{}", _left_line);
            // println!("vs");
            // println!("{}", right_line);
            result += index;
        } else {
            // println!();
            // println!("Didn't add: {}", index);
            // println!("{}", _left_line);
            // println!("vs");
            // println!("{}", right_line);
        }
        
        lines.next();
    }

    println!();
    display_result(&result);
    Ok(())
}

fn part_2() -> anyhow::Result<()> {
    println!("Part 2:");

    Ok(())
}
