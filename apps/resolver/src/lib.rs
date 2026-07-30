//! Application library for the `resolver` command.
//!
//! The library keeps process concerns out of the lookup path so parsing,
//! resolution, and failure behavior can be tested without spawning a process.

pub mod cli;
pub mod error;
pub mod lookup;

use std::io::Write;

pub use cli::Config;
pub use error::ResolverError;
pub use lookup::LookupSummary;

/// Runs Resolver with a validated configuration.
///
/// # Errors
///
/// Returns an error when the name cannot be resolved, no addresses are found,
/// or output cannot be written.
pub fn run<W: Write>(config: &Config, writer: &mut W) -> Result<LookupSummary, ResolverError> {
    lookup::lookup_name(config.name(), writer)
}
