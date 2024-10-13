use std::fmt::Display;
use cli_clipboard::{ClipboardContext, ClipboardProvider};

fn main() {
    part_1();
    // part_2();
}

fn part_1() {
    let input = std::fs::read_to_string("./input.txt").expect("Error reading input file.");
    
    for line in input.lines() {
        // Process lines
    }

    // display_result(&result);
}

fn part_2() {

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
