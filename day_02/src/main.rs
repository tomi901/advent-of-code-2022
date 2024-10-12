
// Note: Could've simplified this a lot by making these shapes ints instead
// And we calculate wins if (0 > 1 > 2 > 0...)
#[derive(Debug, Clone, PartialEq, Eq)]
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
        if self == other {
            RoundResult::Tie
        } else if &self.wins_against() == other {
            RoundResult::Win
        } else {
            RoundResult::Lose
        }
    }

    pub fn score_against(&self, other: &Self) -> u64 {
        self.base_score() + self.result_against(&other).score()
    }

    pub fn wins_against(&self) -> Self {
        match self {
            Self::Rock => Self::Scissors,
            Self::Paper => Self::Rock,
            Self::Scissors => Self::Paper,
        }
    }

    pub fn loses_against(&self) -> Self {
        match self {
            Self::Rock => Self::Paper,
            Self::Paper => Self::Scissors,
            Self::Scissors => Self::Rock,
        }
    }

    pub fn get_opponent_shape(&self, expected_result: &RoundResult) -> Self {
        match expected_result {
            RoundResult::Lose => self.loses_against(),
            RoundResult::Tie => self.clone(),
            RoundResult::Win => self.wins_against(),
        }
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
            Self::Lose => 0,
            Self::Tie => 3,
            Self::Win => 6,
        }
    }

    pub fn inverted(&self) -> Self {
        match self {
            Self::Lose => Self::Win,
            Self::Tie => Self::Tie,
            Self::Win => Self::Lose,
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "A" | "X" => Self::Lose,
            "B" | "Y" => Self::Tie,
            "C" | "Z" => Self::Win,
            _ => todo!("Not implemented: {}", s),
        }
    }
}

fn main() {
    part_2();
}

fn part_1() {
    let input = std::fs::read_to_string("./input.txt").expect("Error reading input file.");

    let mut score = 0;
    for line in input.lines() {
        let (opponent, player) = line
            .split_once(" ")
            .map(|(l, r)| (Shape::from_str(l), Shape::from_str(r)))
            .unwrap();

        // println!("{:?} against {:?} = {} + {}", player, opponent, player.base_score(), player.result_against(&opponent).score());

        score += player.score_against(&opponent);
    }

    println!("Result:");
    println!("{}", score);
}

fn part_2() {
    let input = std::fs::read_to_string("./input.txt").expect("Error reading input file.");

    let mut score = 0;
    for line in input.lines() {
        let (opponent, expected_result) = line
            .split_once(" ")
            .map(|(l, r)| (Shape::from_str(l), RoundResult::from_str(r)))
            .unwrap();

        let player = opponent.get_opponent_shape(&expected_result.inverted());
        score += player.base_score() + expected_result.score();
    }

    println!("Result:");
    println!("{}", score);
}
