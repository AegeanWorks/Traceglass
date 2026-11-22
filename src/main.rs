use std::process::ExitCode;
use traceglass::{parse_trace, Comparison};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [command] if command == "inventory" => {
            println!("cli|inventory|inventory\ncli|demo|demo --scripted");
            ExitCode::SUCCESS
        }
        [command, scripted] if command == "demo" && scripted == "--scripted" => demo(0),
        [command, scripted, flag, value]
            if command == "demo" && scripted == "--scripted" && flag == "--seed" =>
        {
            match value.parse::<u64>() {
                Ok(seed) => demo(seed),
                Err(_) => usage(),
            }
        }
        _ => usage(),
    }
}

fn demo(seed: u64) -> ExitCode {
    let left = parse_trace("start\tready\nfinish\tok\n").expect("valid demo trace");
    let right = parse_trace("start\tready\nlog\textra\nfinish\tok\n").expect("valid demo trace");
    let comparison = Comparison::try_aligned(left, right, 100).expect("bounded demo alignment");
    println!(
        "SEED: {seed}\nWORLD-DELTA: aligned_rows=2->{}\nOK: comparison complete",
        comparison.len()
    );
    ExitCode::SUCCESS
}

fn usage() -> ExitCode {
    eprintln!("Usage: traceglass inventory | demo --scripted [--seed N]");
    ExitCode::from(2)
}
