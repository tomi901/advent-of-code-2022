use std::{collections::HashSet, fmt::Debug, num::ParseIntError, str::FromStr};

use anyhow::{self, Context};
use xmas::display_result;
use bidirectional_map::Bimap;

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
    // let mut mixer = Mixer::new(&numbers);
    // mixer.mix();

    // let mixed_numbers: Vec<i64> = mixer.iter().cloned().collect();
    // let result = [1000, 2000, 3000]
    //     .into_iter()
    //     .map(|i| mixed_numbers[wrapping_index(i, numbers.len())])
    //     .sum::<i64>();

    // display_result(&result);
    Ok(())
}

fn part_2() -> anyhow::Result<()> {
    println!("Part 2:");

    Ok(())
}

fn parse_numbers(content: &str) -> Result<Vec<i64>, ParseIntError> {
    content.lines()
        .map(i64::from_str)
        .collect::<Result<Vec<_>, _>>()
}

#[derive(Clone)]
struct Mixer<'a> {
    arr: &'a [i64],
    move_to: Box<[usize]>,
}

impl<'a> Debug for Mixer<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(self.iter())
            .finish()
    }
}

impl<'a> Mixer<'a> {
    pub fn new(arr: &'a [i64]) -> Self {
        let move_to = (0..arr.len()).collect();
        Self {
            arr,
            move_to,
        }
    }

    pub fn mix(&mut self) {
        // println!("Mix indices started as: {:?}", self.indexes);
        for i in 0..self.move_to.len() {
            self.mix_element(i);
        }
    }

    fn mix_element(&mut self, i: usize) {
        let shift = self.arr[i];
        let cur_i = self.move_to[i];
        let move_to_index = wrapping_index(cur_i as i64 + shift, self.move_to.len());

        self.move_element(i, move_to_index);
        println!("({}) Shifted {} located at {} and placed at {}, list is\n  {:?} and indexes {:?}",
            i, shift, cur_i, move_to_index, self, self.move_to);
    }

    fn move_element(&mut self, index_at: usize, to: usize) {
        let val = format!("{:?}", self.move_to);
        let from = self.move_to[index_at];
        println!("Changing index {} cur value {} to be {}", index_at, from, to);
        match from.cmp(&to) {
            std::cmp::Ordering::Less => {
                println!("Shifting {:?} to the left", from + 1..=to);
                for index in self.move_to.iter_mut()
                    .filter(|i| (from + 1..=to).contains(*i))
                {
                    *index -= 1;
                }
            },
            std::cmp::Ordering::Greater => {
                println!("Shifting {:?} to the right", to..from);
                for index in self.move_to.iter_mut()
                    .filter(|i| (to..from).contains(*i))
                {
                    *index += 1;
                }
            },
            std::cmp::Ordering::Equal => {},
        }
        self.move_to[index_at] = to;
        println!(" Index changes:\n {}\n {:?}", val, self.move_to);
    }

    pub fn iter(&self) -> impl Iterator<Item = &i64> + '_ {
        let mut inverted_move_to = self.move_to.iter().enumerate().collect::<Vec<_>>();
        inverted_move_to.sort_by_key(|(_, to)| *to);
        inverted_move_to.into_iter().map(|(i, _)| &self.arr[i])
    }
}

fn wrapping_index(i: i64, len: usize) -> usize {
    let new_index = i % len as i64;
    if new_index > 0 {
        new_index as usize
    } else {
        (new_index + len as i64 - 1) as usize
    }
}

#[cfg(test)]
mod tests {
    use crate::Mixer;

    #[test]
    fn mixes_correctly_with_example_data() {
        let initial_arrangement = [1, 2, -3, 3, -2, 0, 4];
        let arrangements = [
            [2, 1, -3, 3, -2, 0, 4],
            [1, -3, 2, 3, -2, 0, 4],
            [1, 2, 3, -2, -3, 0, 4],
            [1, 2, -2, -3, 0, 3, 4],
            [1, 2, -3, 0, 3, 4, -2],
            [1, 2, -3, 0, 3, 4, -2],
            [1, 2, -3, 4, 0, 3, -2],
        ];
        let mut mixer = Mixer::new(&initial_arrangement);

        for (i, arrangement) in arrangements.iter().enumerate() {
            mixer.mix_element(i);
            let nums = mixer.iter().cloned().collect::<Vec<i64>>();
            println!("({}) Comparing {:?} and {:?}", i, nums, arrangement);
            assert_eq!(&nums, arrangement);
        }
    }
}
