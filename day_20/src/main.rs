use std::{num::ParseIntError, str::FromStr};

use anyhow::{self, Context};
use xmas::display_result;
use mixer::ShiftMixer;

mod mixer;

fn main() -> anyhow::Result<()> {
    part_1()?;
    println!();
    part_2()?;
    Ok(())
}

fn part_1() -> anyhow::Result<()> {
    println!("Part 1:");
    let input = std::fs::read_to_string("./input.txt").context("Error reading input file.")?;

    let numbers = parse_numbers(&input)?;
    let mut mixer = ShiftMixer::new(&numbers);
    mixer.mix()?;

    let mixed_numbers: Vec<i64> = mixer.iter().collect();
    let nums_file = mixed_numbers.iter().map(|i| format!("{i}\n")).collect::<String>();

    std::fs::write("./temp_output.txt", nums_file)?;
    println!("Written results to ./temp_output.txt");

    // println!("Mixed numbers: {:?}", mixed_numbers);
    let zero_i = mixed_numbers.iter().position(|n| *n == 0).unwrap();
    let result = [1000, 2000, 3000]
        .into_iter()
        .map(|i| {
            let num = mixed_numbers[(zero_i + i) % mixed_numbers.len()];
            // println!("{num}");
            num
        })
        .sum::<i64>();

    display_result(&result);
    Ok(())
}

fn part_2() -> anyhow::Result<()> {
    println!("Part 2:");
    let input = std::fs::read_to_string("./input.txt").context("Error reading input file.")?;

    let numbers = parse_numbers(&input)?;
    let mut mixer = ShiftMixer::new_with_key(&numbers, 811589153);
    mixer.mix_many(10)?;

    let mixed_numbers: Vec<i64> = mixer.iter().collect();
    let nums_file = mixed_numbers.iter().map(|i| format!("{i}\n")).collect::<String>();

    std::fs::write("./temp_output.txt", nums_file)?;
    println!("Written results to ./temp_output.txt");

    // println!("Mixed numbers: {:?}", mixed_numbers);
    let zero_i = mixed_numbers.iter().position(|n| *n == 0).unwrap();
    let result = [1000, 2000, 3000]
        .into_iter()
        .map(|i| {
            let num = mixed_numbers[(zero_i + i) % mixed_numbers.len()];
            // println!("{num}");
            num
        })
        .sum::<i64>();

    display_result(&result);
    Ok(())
}

fn parse_numbers(content: &str) -> Result<Vec<i64>, ParseIntError> {
    content.lines()
        .map(i64::from_str)
        .collect::<Result<Vec<_>, _>>()
}
