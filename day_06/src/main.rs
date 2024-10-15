use std::{collections::HashSet, fmt::Display};
use cli_clipboard::{ClipboardContext, ClipboardProvider};

fn main() {
    // part_1();
    part_2();
}

fn part_1() {
    let input = std::fs::read_to_string("./input.txt").expect("Error reading input file.");
    
    let result: usize = input.lines()
        .flat_map(|s| find_signal_start_index(s, 4))
        .sum();

    display_result(&result);
}

fn part_2() {
    let input = std::fs::read_to_string("./input.txt").expect("Error reading input file.");
    
    let result: usize = input.lines()
        .flat_map(|s| find_signal_start_index(s, 14))
        .sum();

    display_result(&result);
}

fn find_signal_start_index(s: &str, length_required: usize) -> Option<usize> {
    (length_required..s.len())
        .map(|i| (i, &s[(i - length_required)..i]))
        .filter(|&(_, bytes)| {
            let mut lookup = HashSet::new();
            for b in bytes.bytes() {
                if lookup.contains(&b) {
                    return false;
                }
                lookup.insert(b);
            }
            return true;
        })
        .find_map(|(i, _)| Some(i))
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
