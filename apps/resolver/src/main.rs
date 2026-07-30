//! Process entry point for Resolver.

use std::io::{self, Write};
use std::process::ExitCode;

fn main() -> ExitCode {
    match resolver::cli::parse_args(std::env::args_os().skip(1)) {
        Ok(config) => run(&config),
        Err(error) => report(&error),
    }
}

fn run(config: &resolver::Config) -> ExitCode {
    let stdout = io::stdout();
    let mut output = stdout.lock();

    match resolver::run(config, &mut output) {
        Ok(_) => ExitCode::SUCCESS,
        Err(error) => report(&error),
    }
}

fn report(error: &resolver::ResolverError) -> ExitCode {
    let _ = writeln!(io::stderr().lock(), "resolver: {error}");
    ExitCode::from(error.exit_code())
}
