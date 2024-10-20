use std::{num::ParseIntError, str::FromStr};
use xmas::display_result;

#[derive(Debug, Clone, Copy)]
enum Value {
    Literal(i32),
    Old,
}

impl FromStr for Value {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "old" => Ok(Self::Old),
            _ => s.parse().map(Self::Literal),
        }
    }
}

impl Value {
    pub fn eval(&self, old: i32) -> i32 {
        match self {
            Value::Literal(i) => *i,
            Value::Old => old,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Operation {
    Add(Value),
    Multiply(Value),
}

impl FromStr for Operation {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.bytes().next() {
            Some(b'+') => Self::Add(Value::from_str(s[1..].trim()).unwrap()),
            Some(b'*') => Self::Multiply(Value::from_str(s[1..].trim()).unwrap()),
            Some(c) => panic!("Unknown operator: {:?}", char::from_u32(c as u32)),
            None => panic!("Empty string"),
        })
    }
}

impl Operation {
    pub fn apply(&self, old: i32) -> i32 {
        match self {
            Operation::Add(value) => old + value.eval(old),
            Operation::Multiply(value) => old * value.eval(old),
        }
    }
}

#[derive(Debug, Clone)]
struct Monkey {
    items: Vec<i32>,
    operation: Operation,
    divisible_test: i32,
    on_true_pass_to: usize,
    on_false_pass_to: usize,
}

impl Monkey {
    pub fn from_lines<'a>(lines: &mut impl Iterator<Item = &'a str>) -> Option<Self> {
        let header = match lines.skip_while(|l| l.is_empty()).next() {
            Some(s) => s,
            None => return None,
        };
        assert!(header.starts_with("Monkey"));

        let starting_items_line = lines.next().unwrap().trim_start();
        const STARTING_ITEMS: &str = "Starting items:";
        assert!(starting_items_line.starts_with(STARTING_ITEMS));
        let items = starting_items_line
            .trim_start_matches(STARTING_ITEMS)
            .split(',')
            .map(str::trim)
            .map(str::parse::<i32>)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        let operation_line = lines.next().unwrap().trim_start();
        const OPERATION: &str = "Operation: new = old";
        assert!(operation_line.starts_with(OPERATION));
        let operation =
            Operation::from_str(operation_line.trim_start_matches(OPERATION).trim()).unwrap();

        let divisible_test = lines.next()
            .and_then(|l| l.split_whitespace().last())
            .unwrap()
            .parse::<i32>()
            .unwrap();

        let on_true_pass_to = lines.next()
            .and_then(|l| l.split_whitespace().last())
            .unwrap()
            .parse::<usize>()
            .unwrap();

        let on_false_pass_to = lines.next()
            .and_then(|l| l.split_whitespace().last())
            .unwrap()
            .parse::<usize>()
            .unwrap();

        Some(Self {
            items,
            operation,
            divisible_test,
            on_true_pass_to,
            on_false_pass_to,
        })
    }
}

fn main() {
    part_1();
    part_2();
}

fn part_1() {
    let input = std::fs::read_to_string("./input.txt").expect("Error reading input file.");
    let monkeys = {
        let mut lines = input.lines();
        let mut monkeys = vec![];
        while let Some(monkey) = Monkey::from_lines(&mut lines) {
            monkeys.push(monkey);
        }
        monkeys
    };

    // println!("{:#?}", monkeys);

    // display_result(&result);
}

fn part_2() {
    
}
