
#[derive(Debug)]
enum Shape {
    Rock,
    Paper,
    Scissors,
}

impl Shape {
    pub fn base_score(&self) -> u64 {
        match self {
            Self::Rock => 1,
            Self::Paper => 2,
            Self::Scissors => 3,
        }
    }

    pub fn result_against(&self, other: &Self) -> RoundResult {
        match self {
            Self::Rock => match other {
                Self::Rock => RoundResult::Tie,
                Self::Paper => RoundResult::Lose,
                Self::Scissors => RoundResult::Win,
            },
            Self::Paper => match other {
                Self::Rock => RoundResult::Win,
                Self::Paper => RoundResult::Tie,
                Self::Scissors => RoundResult::Lose,
            },
            Self::Scissors => match other {
                Self::Rock => RoundResult::Lose,
                Self::Paper => RoundResult::Win,
                Self::Scissors => RoundResult::Tie,
            },
        }
    }

    pub fn score_against(&self, other: &Self) -> u64 {
        self.base_score() + self.result_against(&other).score()
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "A" | "X" => Self::Rock,
            "B" | "Y" => Self::Paper,
            "C" | "Z" => Self::Scissors,
            _ => todo!("Not implemented: {}", s),
        }
    }
}

#[derive(Debug)]
enum RoundResult {
    Lose,
    Tie,
    Win,
}

impl RoundResult {
    pub fn score(&self) -> u64 {
        match self {
            RoundResult::Lose => 0,
            RoundResult::Tie => 3,
            RoundResult::Win => 6,
        }
    }
}

fn main() {
    let input = std::fs::read_to_string("./input.txt").expect("Error reading input file.");

    let mut score = 0;
    for line in input.lines() {
        let (opponent, player) = line
            .split_once(" ")
            .map(|(l, r)| (Shape::from_str(l), Shape::from_str(r)))
            .unwrap();

        println!("{:?} against {:?} = {} + {}", player, opponent, player.base_score(), player.result_against(&opponent).score());

        score += player.score_against(&opponent);
    }

    println!("Result:");
    println!("{}", score);
}
