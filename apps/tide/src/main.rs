//! Process entry point for Tide.

use std::io::{self, Write};
use std::process::ExitCode;

fn main() -> ExitCode {
    match tide::cli::parse_args(std::env::args_os().skip(1)) {
        Ok(config) => run(&config),
        Err(error) => report(&error),
    }
}

fn run(config: &tide::Config) -> ExitCode {
    let stdout = io::stdout();
    let mut output = stdout.lock();

    match tide::run(config, &mut output) {
        Ok(_) => ExitCode::SUCCESS,
        Err(error) => report(&error),
    }
}

fn report(error: &tide::TideError) -> ExitCode {
    let _ = writeln!(io::stderr().lock(), "tide: {error}");
    ExitCode::from(error.exit_code())
}
