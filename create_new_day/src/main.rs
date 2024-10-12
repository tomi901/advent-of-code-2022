use std::{fs, io::Cursor, process::Command};
use clap::Parser;
use color_print::cprintln;
use tokio;

const YEAR: u64 = 2022;

#[derive(Parser, Debug)]
struct Args {
    day_number: usize,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let crate_name = format!("day_{:02}", args.day_number);
    println!("🎁 Creating crate {crate_name}...");

    Command::new("cargo")
        .args(["workspaces", "create"])
        .args(["--bin", &crate_name])
        .args(["--name", &crate_name])
        .args(["--edition", "2021"])
        .output()
        .expect("Failed to create cargo library.");

    println!("📝 Preparing main.rs...");
    fs::write(format!("{crate_name}/src/main.rs"), TEMPLATE_FILE)
        .expect("Error writing template main.rs.");

    println!("📋 Downloading input...");
    let client = reqwest::Client::new();
    let session = std::env::var("AOC_SESSION").expect("Invalid AOC_SESSION env variable.");
    let result = client.get(format!("https://adventofcode.com/{}/day/{}/input", YEAR, args.day_number))
        .header("Cookie", format!("session={}", session))
        .send()
        .await
        .expect("Client error downloading input!")
        .error_for_status()
        .expect("Server error downloading input!");

    let mut file = std::fs::File::create(format!("{crate_name}/input.txt")).unwrap();
    let mut content = Cursor::new(result.bytes().await.unwrap());

    std::io::copy(&mut content, &mut file).unwrap();

    cprintln!("🎄 <green>Done!</> Don't let Santa down and don't forget to run:");
    cprintln!("   <yellow>cd {crate_name}</>");
}

const TEMPLATE_FILE: &str = r#"
fn main() {
    let input = std::fs::read_to_string("./input.txt").expect("Error reading input file.");
    for line in input.lines() {
        // Process lines
    }

    println!("Result:");
}
"#;
