use std::str::FromStr;
use anyhow::{self, Context};
use day_24::BlizzardMap;
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

    let map = BlizzardMap::from_str(&input)?;
    // map.display_at(0);
    // map.display_at(1);
    // map.display_at(2);
    // map.display_at(3);
    // map.display_at(map.blizzard_loop_len());

    let result = map.navigate().context("No path found")?;

    display_result(&result);
    Ok(())
}

fn part_2() -> anyhow::Result<()> {
    println!("Part 2:");
    let input = std::fs::read_to_string("./input.txt").context("Error reading input file.")?;

    let map = BlizzardMap::from_str(&input)?;
    let result = map.navigate_back_and_forth().context("No path found")?;

    display_result(&result);
    Ok(())
}
