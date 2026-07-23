//! Application library for the `needle` command.
//!
//! The library keeps process concerns out of the search engine so matching,
//! rendering, and failure behavior can be tested without spawning a process.

pub mod cli;
pub mod error;
pub mod matcher;
pub mod output;
pub mod search;

use std::fs::File;
use std::io::{BufReader, Write};

pub use cli::Config;
pub use error::NeedleError;
pub use search::SearchSummary;

use matcher::Matcher;

/// Runs Needle with a validated configuration.
///
/// The caller chooses whether matched text is colored because terminal
/// detection is a process-boundary concern.
///
/// # Errors
///
/// Returns an error when the pattern is invalid, the file cannot be opened or
/// read as UTF-8, or output cannot be written.
pub fn run<W: Write>(
    config: &Config,
    writer: &mut W,
    color: bool,
) -> Result<SearchSummary, NeedleError> {
    let matcher = Matcher::new(config.pattern()).map_err(NeedleError::InvalidPattern)?;
    let file = File::open(config.file()).map_err(|source| NeedleError::OpenFile {
        path: config.file().to_path_buf(),
        source,
    })?;
    let reader = BufReader::new(file);

    search::search_reader(&matcher, reader, writer, color)
        .map_err(|error| NeedleError::from_search_error(config.file().to_path_buf(), error))
}
