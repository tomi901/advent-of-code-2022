use std::process::Command;
use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    day_number: usize,
}

fn main() {
    let args = Args::parse();

    let crate_name = format!("day_{:02}", args.day_number);
    println!("Creating crate {crate_name}...");

    Command::new("cargo")
        .args(["workspaces", "create"])
        .args(["--bin", &crate_name])
        .args(["--name", &crate_name])
        .args(["--edition", "2021"])
        .output()
        .expect("Failed to create cargo library");
}
