use num::integer::lcm;
use std::{cmp::Reverse, num::ParseIntError, ops::Mul, str::FromStr};
use xmas::display_result;

type Item = u64;

#[derive(Debug, Clone, Copy)]
enum Value {
    Literal(Item),
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
    pub fn eval(&self, old: Item) -> Item {
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
    pub fn apply(&self, old: Item) -> Item {
        match self {
            Operation::Add(value) => old + value.eval(old),
            Operation::Multiply(value) => old * value.eval(old),
        }
    }
}

#[derive(Debug, Clone)]
struct Monkey {
    items: Vec<Item>,
    operation: Operation,
    divisible_test: Item,
    on_true_pass_to: usize,
    on_false_pass_to: usize,
    inspection_count: u64,
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
            .map(str::parse::<Item>)
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
            .parse::<Item>()
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

    pub fn calculate_throw_with_decay(&self, item: Item, worry_decay: Item) -> Throw {
        let new_value = self.operation.apply(item) / worry_decay;
        Throw { item: new_value, to: self.get_target(new_value % self.divisible_test == 0) }
    }

    pub fn get_target(&self, condition: bool) -> usize {
        if condition { self.on_true_pass_to } else { self.on_false_pass_to }
    }
}

#[derive(Debug, Clone)]
struct MonkeyGroup {
    monkeys: Vec<Monkey>,
    rounds: u32,
    // Needed to limit the max number and avoid overflows
    // Altough all numbers are primes, so we could've just used a simple mul instead of num::lcm
    lcm: Item,
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
        Ok(MonkeyGroup::new(monkeys))
    }
}

impl MonkeyGroup {
    pub fn new(monkeys: Vec<Monkey>) -> Self {
        let lcm = monkeys.iter().map(|m| m.divisible_test).reduce(lcm).unwrap();
        Self { monkeys, rounds: 0, lcm }
    }

    pub fn play_round(&mut self, worry_decay: Item) {
        self.rounds += 1;
        for i in 0..(self.monkeys.len()) {
            let throws = {
                let monkey = &mut self.monkeys[i];
                let throws = monkey.items
                    .iter()
                    .cloned()
                    .map(|item| monkey.calculate_throw_with_decay(item, worry_decay))
                    .map(|throw| Throw { item: throw.item % self.lcm, ..throw })
                    .collect::<Vec<_>>();
                monkey.inspection_count += monkey.items.len() as u64;
                monkey.items.clear();
                throws
            };
            for throw in throws {
                let target = &mut self.monkeys[throw.to];
                target.items.push(throw.item);
            }
        }
    }

    pub fn monkey_business_level(&self) -> u64 {
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
    item: Item,
    to: usize,
}

fn main() {
    // part_1();
    part_2();
}

fn part_1() {
    let input = std::fs::read_to_string("./input.txt").expect("Error reading input file.");
    let mut monkeys = MonkeyGroup::from_str(&input).unwrap();
    for _ in 0..20 {
        monkeys.play_round(3);
    }
    // println!("{:#?}", monkeys);

    let result = monkeys.monkey_business_level();
    display_result(&result);
}

fn part_2() {
    let input = std::fs::read_to_string("./input.txt").expect("Error reading input file.");
    let mut monkeys = MonkeyGroup::from_str(&input).unwrap();
    println!("LCM: {}", monkeys.lcm);
    println!("Mul: {}", monkeys.monkeys.iter().map(|m| m.divisible_test).reduce(Mul::mul).unwrap());
    for _ in 0..10_000 {
        monkeys.play_round(1);
    }
    // println!("{:#?}", monkeys);

    let result = monkeys.monkey_business_level();
    display_result(&result);
}
