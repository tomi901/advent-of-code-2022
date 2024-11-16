use std::{collections::HashMap, str::FromStr};

use anyhow::{self, Context};
use xmas::display_result;
use self::MonkeyResult::*;

const ROOT: &str = "root";
const HUMAN: &str = "humn";

fn main() -> anyhow::Result<()> {
    part_1()?;
    println!();
    part_2()?;
    Ok(())
}

fn part_1() -> anyhow::Result<()> {
    println!("Part 1:");
    let input = std::fs::read_to_string("./input.txt").context("Error reading input file.")?;

    let monkeys = MonkeyGroup::from_str(&input)?;
    let result = monkeys.eval(ROOT)?;

    display_result(&result);
    Ok(())
}

fn part_2() -> anyhow::Result<()> {
    println!("Part 2:");
    let input = std::fs::read_to_string("./input.txt").context("Error reading input file.")?;

    let monkeys = MonkeyGroup::from_str(&input)?;
    let result = monkeys.solve_human_value()?;

    display_result(&result);
    Ok(())
}

#[derive(Debug, Clone)]
enum MonkeyValue {
    Number(i64),
    Operation(String, Operation, String),
}

impl MonkeyValue {
    pub fn from_str_with_key(s: &str) -> Result<(String, Self), anyhow::Error> {
        let (id, value) = s.split_once(':').context("Id and value not separated by :")?;
        let mut split = value.trim().split_whitespace();

        let first_segment = split.next().context("No first segment")?;
        if first_segment.chars().all(|c| c.is_digit(10)) {
            let num = first_segment.parse::<i64>()?;
            return Ok((id.to_string(), Self::Number(num)));
        }

        let operation = match split.next().context("No operation found")? {
            "+" => Operation::Add,
            "-" => Operation::Sub,
            "*" => Operation::Mul,
            "/" => Operation::Div,
            op => return Err(anyhow::anyhow!("Op not recognized: {op}")),
        };

        let third_segment = split.next().context("No third segment")?;
        Ok((id.to_string(), Self::Operation(first_segment.to_string(), operation, third_segment.to_string())))
    }
}

#[derive(Debug, Clone, Copy)]
enum Operation {
    Add,
    Sub,
    Mul,
    Div,
}

impl Operation {
    pub fn apply(&self, lhs: i64, rhs: i64) -> i64 {
        match self {
            Operation::Add => lhs + rhs,
            Operation::Sub => lhs - rhs,
            Operation::Mul => lhs * rhs,
            Operation::Div => lhs / rhs,
        }
    }
}

#[derive(Debug)]
enum MonkeyResult<'a> {
    Known(i64),
    Unknown(&'a str),
}

impl<'a> MonkeyResult<'a> {
    pub fn from_result(id: &'a str, result: Option<i64>) -> Self {
        match result {
            Some(n) => Known(n),
            None => Unknown(id),
        }
    }
}

struct MonkeyGroup {
    monkeys: HashMap<String, MonkeyValue>,
}

impl MonkeyGroup {
    pub fn eval(&self, monkey_id: &str) -> Result<i64, anyhow::Error> {
        let monkey = self.monkeys
            .get(monkey_id)
            .with_context(|| format!("No monkey with id '{monkey_id}' found"))?;

        Ok(match monkey {
            MonkeyValue::Number(n) => *n,
            MonkeyValue::Operation(lhs, op, rhs) => {
                let left = self.eval(lhs)?;
                let right = self.eval(rhs)?;
                op.apply(left, right)
            },
        })
    }

    pub fn eval_non_human(&self, monkey_id: &str) -> Result<Option<i64>, anyhow::Error> {
        if monkey_id == HUMAN {
            return Ok(None);
        }

        let monkey = self.monkeys
            .get(monkey_id)
            .with_context(|| format!("No monkey with id '{monkey_id}' found"))?;

        Ok(match monkey {
            MonkeyValue::Number(n) => Some(*n),
            MonkeyValue::Operation(lhs, op, rhs) => {
                let left = match self.eval_non_human(lhs)? {
                    Some(n) => n,
                    None => return Ok(None),
                };
                let right = match self.eval_non_human(rhs)? {
                    Some(n) => n,
                    None => return Ok(None),
                };
                Some(op.apply(left, right))
            },
        })
    }

    pub fn solve_human_value(&self) -> Result<i64, anyhow::Error> {
        let root = self.monkeys
            .get(ROOT)
            .with_context(|| format!("No root monkey found"))?;
        let results = match root {
            MonkeyValue::Operation(lhs, _, rhs) =>
                (
                    MonkeyResult::from_result(lhs, self.eval_non_human(lhs)?),
                    MonkeyResult::from_result(rhs, self.eval_non_human(rhs)?),
                ),
            _ => unreachable!(),
        };

        match results {
            (Known(expected), Unknown(x)) |
            (Unknown(x), Known(expected)) => self.solve_for_result(x, expected),
            _ => unreachable!(),
        }
    }

    fn solve_for_result(&self, monkey_id: &str, expected: i64) -> Result<i64, anyhow::Error> {
        if monkey_id == HUMAN {
            return Ok(expected);
        }

        let monkey = self.monkeys
            .get(monkey_id)
            .with_context(|| format!("No root monkey found"))?;
        let operation = match monkey {
            MonkeyValue::Operation(lhs, op, rhs) =>
                (
                    MonkeyResult::from_result(lhs, self.eval_non_human(lhs)?),
                    op,
                    MonkeyResult::from_result(rhs, self.eval_non_human(rhs)?),
                ),
            _ => unreachable!("Unreacheable for {monkey_id}: {:?}", monkey),
        };
        
        let (x, x_expected) = match operation {
            (Known(n), Operation::Add, Unknown(x)) |
            (Unknown(x), Operation::Add, Known(n)) => (x, expected - n),
            (Known(n), Operation::Mul, Unknown(x)) |
            (Unknown(x), Operation::Mul, Known(n)) => (x, expected / n),
            // n - x = expected => x = n - expected
            (Known(n), Operation::Sub, Unknown(x)) => (x, n - expected),
            // x - n = expected => x = expected + n
            (Unknown(x), Operation::Sub, Known(n)) => (x, expected + n),
            // n / x = expected => n = expected * x => x = n / expected
            (Known(n), Operation::Div, Unknown(x)) => (x, n / expected),
            // x / n = expected => x = expected * n
            (Unknown(x), Operation::Div, Known(n)) => (x, expected * n),
            _ => unreachable!("Unreacheable pattern: {:?}", operation),
        };

        self.solve_for_result(x, x_expected)
    }
}

impl FromStr for MonkeyGroup {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let monkeys = s.lines()
            .map(MonkeyValue::from_str_with_key)
            .collect::<Result<HashMap<_, _>, _>>()?;
        Ok(Self {
            monkeys,
        })
    }
}
