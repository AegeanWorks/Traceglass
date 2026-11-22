use std::io::{self, BufRead, Read, Write};
use std::process::ExitCode;
use traceglass::input::read_trace;
use traceglass::ui::{parse_action, render};
use traceglass::{parse_trace, Comparison, Navigation};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [command] if command == "inventory" => {
            println!("cli|inventory|inventory\ncli|demo|demo --scripted\ncli|view|view LEFT RIGHT");
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
        [command, left, right] if command == "view" => match view(left, right) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("Error: {error}");
                ExitCode::FAILURE
            }
        },
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
    eprintln!("Usage: traceglass view LEFT RIGHT | inventory | demo --scripted [--seed N]");
    ExitCode::from(2)
}

fn view(left: &str, right: &str) -> Result<(), String> {
    const MAX_BYTES: usize = 4 * 1024 * 1024;
    const MAX_CELLS: usize = 1_000_000;
    let load = |path: &str| {
        let file =
            std::fs::File::open(path).map_err(|error| format!("cannot open {path:?}: {error}"))?;
        read_trace(file, MAX_BYTES).map_err(|error| format!("{path:?}: {error}"))
    };
    let comparison = Comparison::try_aligned(load(left)?, load(right)?, MAX_CELLS)
        .map_err(|_| "traces exceed the alignment limit of 1,000,000 cells".to_string())?;
    let mut state = Navigation::new(&comparison);
    let stdin = io::stdin();
    let mut input = stdin.lock();
    let stdout = io::stdout();
    let mut output = stdout.lock();
    while !state.is_finished() {
        write!(output, "{}> ", render(&comparison, state))
            .and_then(|_| output.flush())
            .map_err(|error| format!("console output failed: {error}"))?;
        let mut line = Vec::new();
        let count = (&mut input)
            .take(257)
            .read_until(b'\n', &mut line)
            .map_err(|error| format!("console input failed: {error}"))?;
        if count == 0 {
            break;
        }
        if line.len() > 256 {
            return Err("command exceeds 256 bytes".into());
        }
        let command = std::str::from_utf8(&line).map_err(|_| "command is not UTF-8".to_string())?;
        if let Some(action) = parse_action(command) {
            state = state.apply(action, &comparison);
        } else {
            writeln!(output, "Unknown command. Use p, n, P, N, or q.")
                .map_err(|error| format!("console output failed: {error}"))?;
        }
    }
    Ok(())
}
