//! Process entry point for Needle.

use std::io::{self, IsTerminal};
use std::process::ExitCode;

fn main() -> ExitCode {
    match needle::cli::parse_args(std::env::args_os().skip(1)) {
        Ok(config) => run(&config),
        Err(error) => {
            eprintln!("needle: {error}");
            ExitCode::from(2)
        }
    }
}

fn run(config: &needle::Config) -> ExitCode {
    let stdout = io::stdout();
    let color = stdout.is_terminal();
    let mut output = stdout.lock();

    match needle::run(config, &mut output, color) {
        Ok(summary) if summary.found_match() => ExitCode::SUCCESS,
        Ok(_) => ExitCode::from(1),
        Err(error) => {
            eprintln!("needle: {error}");
            ExitCode::from(2)
        }
    }
}
