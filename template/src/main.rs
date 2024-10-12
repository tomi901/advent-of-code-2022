use std::fmt::Display;


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

fn display_result<T: Display>(result: &T) {
    println!("Result:");
    println!("{}", result);
}
