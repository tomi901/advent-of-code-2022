use std::{cmp::Reverse, num::ParseIntError, str::FromStr};
use xmas::display_result;

#[derive(Debug, Clone, Copy)]
enum Value {
    Literal(u64),
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
    pub fn eval(&self, old: u64) -> u64 {
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
    pub fn apply(&self, old: u64) -> u64 {
        match self {
            Operation::Add(value) => old + value.eval(old),
            Operation::Multiply(value) => old * value.eval(old),
        }
    }
}

#[derive(Debug, Clone)]
struct Monkey {
    items: Vec<u64>,
    operation: Operation,
    divisible_test: u64,
    on_true_pass_to: usize,
    on_false_pass_to: usize,
    inspection_count: u32,
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
            .map(str::parse::<u64>)
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
            .parse::<u64>()
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
            inspection_count: 0,
        })
    }
}

#[derive(Debug, Clone)]
struct MonkeyGroup {
    rounds: u32,
    monkeys: Vec<Monkey>,
}

impl FromStr for MonkeyGroup {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let monkeys = {
            let mut lines = s.lines();
            let mut monkeys = vec![];
            while let Some(monkey) = Monkey::from_lines(&mut lines) {
                monkeys.push(monkey);
            }
            monkeys
        };
        Ok(Self { monkeys, rounds: 0 })
    }
}

impl MonkeyGroup {
    pub fn play_round(&mut self) {
        const WORRY_DECAY: u64 = 3;
        self.rounds += 1;
        for i in 0..(self.monkeys.len()) {
            let throws = {
                let monkey = &mut self.monkeys[i];
                let throws = monkey.items
                    .iter()
                    .cloned()
                    .map(|item| monkey.operation.apply(item) / WORRY_DECAY)
                    .map(|item| {
                        let to = if item % monkey.divisible_test == 0 {
                            monkey.on_true_pass_to
                        } else {
                            monkey.on_false_pass_to
                        };
                        Throw { item, to }
                    })
                    .collect::<Vec<_>>();
                monkey.inspection_count += monkey.items.len() as u32;
                monkey.items.clear();
                throws
            };
            for throw in throws {
                let target = &mut self.monkeys[throw.to];
                target.items.push(throw.item);
            }
        }
    }

    pub fn monkey_business_level(&self) -> u32 {
        // We can optimize this, but let's skip it for now
        let mut levels = self.monkeys.iter()
            .map(|m| m.inspection_count)
            .collect::<Vec<_>>();
        levels.sort_unstable_by_key(|&i| Reverse(i));
        println!("Scores: {:?}", levels);
        levels[0] * levels[1]
    }
}

struct Throw {
    item: u64,
    to: usize,
}

fn main() {
    part_1();
    part_2();
}

fn part_1() {
    let input = std::fs::read_to_string("./input.txt").expect("Error reading input file.");
    let mut monkeys = MonkeyGroup::from_str(&input).unwrap();
    for _ in 0..20 {
        monkeys.play_round();
    }
    // println!("{:#?}", monkeys);

    let result = monkeys.monkey_business_level();
    display_result(&result);
}

fn part_2() {
    
}
