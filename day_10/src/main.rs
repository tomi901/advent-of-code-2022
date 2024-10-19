use std::str::FromStr;
use xmas::display_result;

enum Instruction {
    Noop,
    AddX(i64),
}

impl FromStr for Instruction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split_whitespace();
        let cmd_name = split.next();
        Ok(match cmd_name {
            Some("addx") => {
                let amount = split
                    .next()
                    .expect("No addx ammount")
                    .parse::<i64>()
                    .unwrap();
                Self::AddX(amount)
            },
            Some("noop") => Self::Noop,
            Some(cmd) => panic!("Unrecognized command: {}", cmd),
            None => panic!("Empty command"),
        })
    }
}

impl Instruction {
    pub fn delay(&self) -> u8 {
        match self {
            Instruction::Noop => 1,
            Instruction::AddX(_) => 2,
        }
    }
}

struct CPUExecution<I: Iterator<Item = Instruction>> {
    instructions: I,
    cur_instruction: Instruction,
    delay: u8,
    state: CPUState,
}

impl<I: Iterator<Item = Instruction>> CPUExecution<I> {
    pub fn new(instructions: I) -> Self {
        Self { instructions, cur_instruction: Instruction::Noop, delay: 0, state: CPUState::new() }
    }
}

impl<I: Iterator<Item = Instruction>> Iterator for CPUExecution<I> {
    type Item = CPUState;

    fn next(&mut self) -> Option<Self::Item> {
        if self.delay == 0 {
            self.state.execute(&self.cur_instruction);

            // This will ignore the last instruction unless we add some "finished" flag
            self.cur_instruction = match self.instructions.next() {
                Some(i) => i,
                None => return None,
            };

            self.delay = self.cur_instruction.delay();
        }

        self.delay -= 1;
        self.state.cycle += 1;

        Some(self.state.clone())
    }
}

#[derive(Debug, Clone)]
struct CPUState {
    pub reg_x: i64,
    pub cycle: u64,
}

impl CPUState {
    pub fn new() -> Self {
        Self { reg_x: 1, cycle: 0 }
    }

    pub fn signal_strength(&self) -> i64 {
        // println!("Signal: cycle {} * reg_x {} = {}", self.cycle, self.reg_x, self.cycle as i64 * self.reg_x);
        self.cycle as i64 * self.reg_x
    }

    pub fn execute(&mut self, instruction: &Instruction) {
        match instruction {
            Instruction::Noop => {},
            Instruction::AddX(amount) => self.reg_x += amount,
        }
    }
}

fn main() {
    part_1();
    part_2();
}

fn part_1() {
    let input = std::fs::read_to_string("./input.txt").expect("Error reading input file.");
    
    let instructions = input.lines().map(Instruction::from_str).collect::<Result<Vec<_>, _>>().unwrap();
    let result = CPUExecution::new(instructions.into_iter())
        .filter(|s| s.cycle >= 20 && (s.cycle - 20) % 40 == 0)
        .map(|s| s.signal_strength())
        .sum::<i64>();

    display_result(&result);
}

fn part_2() {
    let input = std::fs::read_to_string("./input.txt").expect("Error reading input file.");
    let instructions = input.lines().map(Instruction::from_str).collect::<Result<Vec<_>, _>>().unwrap();
    
    const WIDTH: u64 = 40;
    const HEIGHT: u64 = 6;
    const SPRITE_WIDTH: u64 = 1;
    let mut render = String::with_capacity((WIDTH * HEIGHT + HEIGHT) as usize);

    let mut x_pos: u64 = 0;
    for state in CPUExecution::new(instructions.into_iter()) {
        let draw = state.reg_x.abs_diff(x_pos as i64) <= SPRITE_WIDTH;
        render.push(if draw { '#' } else { '.' });

        x_pos += 1;
        if x_pos >= WIDTH {
            x_pos = 0;
            render.push('\n');
        }
    }

    display_result(&render);
}
