use std::ffi::OsString;
use std::io::{self, BufRead, Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use traceglass::input::{read_input, InputFormat};
use traceglass::ui::{parse_action, render, summarize, summarize_json, summarize_json_all};
use traceglass::{parse_trace, Comparison, Navigation};

/// A parsed command. Operands stay native paths until the shell opens them (ADR-004).
enum Command {
    Help,
    Inventory,
    Demo { seed: Option<u64> },
    View { left: PathBuf, right: PathBuf, format: InputFormat },
    Check {
        left: PathBuf,
        right: PathBuf,
        json: bool,
        all: bool,
        format: InputFormat,
    },
}

fn is(arg: &OsString, text: &str) -> bool {
    arg.to_str() == Some(text)
}

/// Parse flags in any order, then require exactly two native path operands.
fn parse_args(args: &[OsString]) -> Result<Command, ()> {
    let Some((first, rest)) = args.split_first() else {
        return Err(());
    };
    if is(first, "help") || is(first, "--help") || is(first, "-h") {
        return if rest.is_empty() { Ok(Command::Help) } else { Err(()) };
    }
    if is(first, "inventory") {
        return if rest.is_empty() { Ok(Command::Inventory) } else { Err(()) };
    }
    if is(first, "demo") {
        if rest.len() == 1 && is(&rest[0], "--scripted") {
            return Ok(Command::Demo { seed: None });
        }
        if rest.len() == 3 && is(&rest[0], "--scripted") && is(&rest[1], "--seed") {
            let seed = rest[2].to_str().and_then(|s| s.parse().ok()).ok_or(())?;
            return Ok(Command::Demo { seed: Some(seed) });
        }
        return Err(());
    }
    let view = is(first, "view");
    if !view && !is(first, "check") {
        return Err(());
    }
    let (mut json, mut all, mut lines) = (false, false, false);
    let mut operands = Vec::new();
    for arg in rest {
        if is(arg, "--json") {
            json = true;
        } else if is(arg, "--lines") {
            lines = true;
        } else if is(arg, "--all-differences") {
            all = true;
        } else {
            operands.push(arg);
        }
    }
    if operands.len() != 2 {
        return Err(());
    }
    let format = if lines {
        InputFormat::Lines
    } else {
        InputFormat::Structured
    };
    let left = PathBuf::from(operands[0]);
    let right = PathBuf::from(operands[1]);
    if view {
        return (!json && !all)
            .then_some(Command::View { left, right, format })
            .ok_or(());
    }
    if !all || json {
        Ok(Command::Check { left, right, json, all, format })
    } else {
        Err(())
    }
}

const HELP: &str = "\
Usage: traceglass view [--lines] LEFT RIGHT | check [--json] [--all-differences] [--lines] LEFT RIGHT | inventory | demo --scripted [--seed N] | help\n\
Flags may appear in any order. --lines treats each UTF-8 line as an event; --json emits version 1; --all-differences requires --json and emits version 2 with every differing row; - reads one trace from stdin in check.\n\
Exit codes: check 0 equal, 1 different, 2 usage/input/output error.\n";

fn usage() -> ExitCode {
    eprint!("{HELP}");
    ExitCode::from(2)
}

fn main() -> ExitCode {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    match parse_args(&args) {
        Ok(Command::Help) => {
            print!("{HELP}");
            ExitCode::SUCCESS
        }
        Ok(Command::Inventory) => {
            println!("cli|inventory|inventory\ncli|demo|demo --scripted\ncli|view|view LEFT RIGHT\ncli|check|check LEFT RIGHT");
            ExitCode::SUCCESS
        }
        Ok(Command::Demo { seed }) => demo(seed.unwrap_or(0)),
        Ok(Command::View { left, right, format }) => {
            match view(&left, &right, format) {
                Ok(()) => ExitCode::SUCCESS,
                Err(error) => {
                    eprintln!("Error: {error}");
                    ExitCode::FAILURE
                }
            }
        }
        Ok(Command::Check { left, right, json, all, format }) => {
            run_check(&left, &right, json, all, format)
        }
        Err(()) => usage(),
    }
}

fn run_check(
    left: &Path,
    right: &Path,
    json: bool,
    all: bool,
    format: InputFormat,
) -> ExitCode {
    match check(left, right, json, all, format) {
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

fn check(
    left: &Path,
    right: &Path,
    json: bool,
    all: bool,
    format: InputFormat,
) -> Result<bool, String> {
    let comparison = load_comparison(left, right, true, format)?;
    let (different, summary) = if all {
        summarize_json_all(&comparison)
    } else if json {
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
            .take(258)
            .read_until(b'\n', &mut line)
            .map_err(|error| format!("console input failed: {error}"))?;
        if count == 0 {
            break;
        }
        let command_len = line.len() - usize::from(line.last() == Some(&b'\n'));
        if command_len > 256 {
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
