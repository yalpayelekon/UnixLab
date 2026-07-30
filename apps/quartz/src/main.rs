//! Process entry point for Quartz.

use std::io;
use std::process::ExitCode;

fn main() -> ExitCode {
    match quartz::cli::parse_args(std::env::args_os().skip(1)) {
        Ok(config) => run(&config),
        Err(error) => {
            eprintln!("quartz: {error}");
            ExitCode::from(1)
        }
    }
}

fn run(config: &quartz::Config) -> ExitCode {
    let stdout = io::stdout();
    let mut output = stdout.lock();

    match quartz::run(config, &mut output) {
        Ok(_) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("quartz: {error}");
            ExitCode::from(1)
        }
    }
}
