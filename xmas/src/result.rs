use std::fmt::Display;

pub fn display_result<T: Display>(result: &T) {
    println!("Result:");
    println!("{}", result);
}
