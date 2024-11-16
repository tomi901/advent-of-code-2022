use std::{collections::HashMap, str::FromStr};

use anyhow::{self, Context};
use xmas::display_result;

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
    let result = monkeys.eval("root")?;

    display_result(&result);
    Ok(())
}

fn part_2() -> anyhow::Result<()> {
    println!("Part 2:");

    Ok(())
}

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

enum Operation {
    Add,
    Sub,
    Mul,
    Div,
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
                match op {
                    Operation::Add => left + right,
                    Operation::Sub => left - right,
                    Operation::Mul => left * right,
                    Operation::Div => left / right,
                }
            },
        })
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
