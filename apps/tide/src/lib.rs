//! Application library for the `tide` command.
//!
//! The library separates HTTP transfer policy from the process boundary so
//! behavior can be tested without depending on global stdout or stderr.

pub mod cli;
pub mod client;
pub mod error;

use std::io::Write;

pub use cli::Config;
pub use client::TransferSummary;
pub use error::TideError;

/// Executes one HTTP GET and streams its response body to `writer`.
///
/// # Errors
///
/// Returns an error when the URL is unsupported, the request fails, the
/// response body cannot be read, or output cannot be written.
pub fn run<W: Write>(config: &Config, writer: &mut W) -> Result<TransferSummary, TideError> {
    client::get(config.url(), writer)
}
