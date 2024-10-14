use std::{fmt::Display, str::FromStr};
use cli_clipboard::{ClipboardContext, ClipboardProvider};

type Crate = char;
type CrateStack = Vec<Crate>;

#[derive(Debug, Default)]
struct CrateCollection(Vec<CrateStack>);

impl CrateCollection {
    pub fn from_lines<'a>(lines: &mut impl Iterator<Item = &'a str>) -> Self {
        let mut collection = Self::default();
        for line in lines.into_iter() {
            if line.starts_with(" 1 ") {
                break;
            }

            let mut index = 0;
            let mut chars = line.chars();
            loop {
                match chars.next() {
                    Some('[') => {},
                    Some(' ') => {
                        chars.nth(2);
                        index += 1;
                        continue;
                    },
                    None => break,
                    c => panic!("Found: {:?}", c),
                }
                let _crate = chars.next().unwrap();
                assert_eq!(chars.next(), Some(']'));

                collection.add_crate(_crate, index);

                if chars.next() == None {
                    break;
                }

                index += 1;
            }
        }

        for stack in collection.0.iter_mut() {
            stack.reverse();
        }

        lines.next();
        collection
    }

    fn add_crate(&mut self, _crate: Crate, stack_index: usize) {
        self.ensure_stacks_amount(stack_index + 1);
        self.0[stack_index].push(_crate);
    }
    
    fn ensure_stacks_amount(&mut self, amount: usize) {
        for _ in self.0.len()..amount {
            self.0.push(CrateStack::new());
        }
    }

    pub fn execute_instruction(&mut self, instruction: &Instruction) {
        for _ in 0..instruction.quantity {
            self.move_crate(instruction.from, instruction.to);
        }
    }
    
    fn move_crate(&mut self, from: usize, to: usize) {
        let _crate = self.0[from].pop().unwrap();
        self.0[to].push(_crate);
    }

    pub fn top_crates(&self) -> String {
        self.0.iter()
            .map(|stack| stack.last().unwrap_or(&' '))
            .collect()
    }
    
    pub fn execute_instruction_9001(&mut self, instruction: &Instruction) {
        self.execute_instruction(instruction);
        let mut stack = &mut self.0[instruction.to];
        let len = stack.len();
        stack[(len - instruction.quantity)..len].reverse();
    }
}

struct Instruction {
    quantity: usize,
    from: usize,
    to: usize,
}

impl FromStr for Instruction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split_whitespace();
        split.next().unwrap();
        let quantity = split.next().unwrap().parse().unwrap();
        split.next().unwrap();
        let from = split.next().unwrap().parse::<usize>().unwrap() - 1;
        split.next().unwrap();
        let to = split.next().unwrap().parse::<usize>().unwrap() - 1;
        Ok(Self { quantity, from, to })
    }
}

fn main() {
    // part_1();
    part_2();
}

fn part_1() {
    let input = std::fs::read_to_string("./input.txt").expect("Error reading input file.");
    
    let mut lines = input.lines();
    let mut collection = CrateCollection::from_lines(&mut lines);

    for instruction in lines.map(Instruction::from_str) {
        let _instruction = instruction.unwrap();
        collection.execute_instruction(&_instruction);
    }

    let result = collection.top_crates();
    display_result(&result);
}

fn part_2() {
    let input = std::fs::read_to_string("./input.txt").expect("Error reading input file.");
    
    let mut lines = input.lines();
    let mut collection = CrateCollection::from_lines(&mut lines);

    for instruction in lines.map(Instruction::from_str) {
        let _instruction = instruction.unwrap();
        collection.execute_instruction_9001(&_instruction);
    }

    let result = collection.top_crates();
    display_result(&result);
}

// TODO: Move this to common library crate
fn display_result<T: Display>(result: &T) {
    println!("Result:");
    let str_result = format!("{}", result);
    println!("{}", &str_result);

    let mut clipboard = ClipboardContext::new().unwrap();
    clipboard.set_contents(str_result.clone()).unwrap();
    println!("Copied result to clipboard!");
}
