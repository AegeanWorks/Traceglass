use std::io::{self, BufRead, Read, Write};
use std::path::Path;
use std::process::ExitCode;
use traceglass::input::{read_input, InputFormat};
use traceglass::ui::{parse_action, render, summarize, summarize_json};
use traceglass::{parse_trace, Comparison, Navigation};

fn main() -> ExitCode {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    match args.as_slice() {
        [command] if command == "inventory" => {
            println!("cli|inventory|inventory\ncli|demo|demo --scripted\ncli|view|view LEFT RIGHT\ncli|check|check LEFT RIGHT");
            ExitCode::SUCCESS
        }
        [command, scripted] if command == "demo" && scripted == "--scripted" => demo(0),
        [command, scripted, flag, value]
            if command == "demo" && scripted == "--scripted" && flag == "--seed" =>
        {
            match value.to_str().and_then(|text| text.parse::<u64>().ok()) {
                Some(seed) => demo(seed),
                None => usage(),
            }
        }
        [command, left, right] if command == "view" => {
            match view(Path::new(left), Path::new(right), InputFormat::Structured) {
                Ok(()) => ExitCode::SUCCESS,
                Err(error) => {
                    eprintln!("Error: {error}");
                    ExitCode::FAILURE
                }
            }
        }
        [command, flag, left, right] if command == "view" && flag == "--lines" => {
            match view(Path::new(left), Path::new(right), InputFormat::Lines) {
                Ok(()) => ExitCode::SUCCESS,
                Err(error) => {
                    eprintln!("Error: {error}");
                    ExitCode::FAILURE
                }
            }
        }
        [command, left, right] if command == "check" => run_check(
            Path::new(left),
            Path::new(right),
            false,
            InputFormat::Structured,
        ),
        [command, flag, left, right] if command == "check" && flag == "--json" => run_check(
            Path::new(left),
            Path::new(right),
            true,
            InputFormat::Structured,
        ),
        [command, flag, left, right] if command == "check" && flag == "--lines" => {
            run_check(Path::new(left), Path::new(right), false, InputFormat::Lines)
        }
        [command, a, b, left, right]
            if command == "check"
                && ((a == "--json" && b == "--lines") || (a == "--lines" && b == "--json")) =>
        {
            run_check(Path::new(left), Path::new(right), true, InputFormat::Lines)
        }
        _ => usage(),
    }
}

fn run_check(left: &Path, right: &Path, json: bool, format: InputFormat) -> ExitCode {
    match check(left, right, json, format) {
        Ok(false) => ExitCode::SUCCESS,
        Ok(true) => ExitCode::from(1),
        Err(error) => {
            eprintln!("Error: {error}");
            ExitCode::from(2)
        }
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
    eprintln!("Usage: traceglass view [--lines] LEFT RIGHT | check [--json] [--lines] LEFT RIGHT | inventory | demo --scripted [--seed N]");
    ExitCode::from(2)
}

fn load_comparison(
    left: &Path,
    right: &Path,
    allow_stdin: bool,
    format: InputFormat,
) -> Result<Comparison, String> {
    const MAX_BYTES: usize = 4 * 1024 * 1024;
    const MAX_CELLS: usize = 1_000_000;
    let is_stdin = |path: &Path| allow_stdin && path.as_os_str() == "-";
    if is_stdin(left) && is_stdin(right) {
        return Err("standard input can be used for only one trace".to_string());
    }
    let load = |path: &Path| {
        if is_stdin(path) {
            return read_input(io::stdin().lock(), MAX_BYTES, format)
                .map_err(|error| format!("standard input: {error}"));
        }
        let file =
            std::fs::File::open(path).map_err(|error| format!("cannot open {path:?}: {error}"))?;
        read_input(file, MAX_BYTES, format).map_err(|error| format!("{path:?}: {error}"))
    };
    Comparison::try_aligned(load(left)?, load(right)?, MAX_CELLS)
        .map_err(|_| "traces exceed the alignment limit of 1,000,000 cells".to_string())
}

fn check(left: &Path, right: &Path, json: bool, format: InputFormat) -> Result<bool, String> {
    let comparison = load_comparison(left, right, true, format)?;
    let (different, summary) = if json {
        summarize_json(&comparison)
    } else {
        summarize(&comparison)
    };
    io::stdout()
        .lock()
        .write_all(summary.as_bytes())
        .map_err(|error| format!("console output failed: {error}"))?;
    Ok(different)
}

fn view(left: &Path, right: &Path, format: InputFormat) -> Result<(), String> {
    let comparison = load_comparison(left, right, false, format)?;
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
