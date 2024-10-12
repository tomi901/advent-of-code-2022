use std::cmp::Reverse;


fn main() {
    let input = std::fs::read_to_string("./input.txt").expect("Error reading input file.");

    // If we had yield to easily create iterators, this would've been so much easier
    let mut total_calories = 0;
    let mut calories_sums = Vec::new();
    for line in input.lines() {
        if line.is_empty() {
            calories_sums.push(total_calories);
            total_calories = 0;
            continue;
        }

        let calories: i64 = line.parse().unwrap();
        total_calories += calories;
    }
    calories_sums.push(total_calories);
    calories_sums.sort_by_key(|&c| Reverse(c));

    let result: i64 = calories_sums.iter().take(3).sum();
    println!("Result:");
    println!("{result}");
}
