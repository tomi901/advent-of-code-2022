use std::str::FromStr;

use anyhow::{self, Context};
use day_25::SNAFU;
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

    let snafus = input.lines().map(SNAFU::from_str).collect::<Result<Vec<_>, _>>()?;
    let result = snafus.into_iter().sum::<SNAFU>().to_string();

    display_result(&result);
    Ok(())
}

fn part_2() -> anyhow::Result<()> {
    println!("Part 2:");

    Ok(())
}
