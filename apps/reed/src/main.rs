//! Process entry point for Reed.

use std::io;
use std::process::ExitCode;

fn main() -> ExitCode {
    match reed::cli::parse_args(std::env::args_os().skip(1)) {
        Ok(config) => run(&config),
        Err(error) => {
            eprintln!("reed: {error}");
            ExitCode::from(1)
        }
    }
}

fn run(config: &reed::Config) -> ExitCode {
    let stdout = io::stdout();
    let mut output = stdout.lock();

    match reed::run(config, &mut output) {
        Ok(_) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("reed: {error}");
            ExitCode::from(1)
        }
    }
}
