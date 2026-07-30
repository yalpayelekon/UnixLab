//! Application library for the `quartz` command.
//!
//! The library keeps process concerns out of the query engine so parsing,
//! evaluation, and failure behavior can be tested without spawning a process.

pub mod cli;
pub mod error;
pub mod filter;
pub mod query;

use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;

use serde_json::Value;

pub use cli::Config;
pub use error::QuartzError;
pub use filter::Filter;
pub use query::QuerySummary;

/// Runs Quartz with a validated configuration.
///
/// # Errors
///
/// Returns an error when the filter is invalid, an input cannot be opened or
/// parsed as JSON, or output cannot be written.
pub fn run<W: Write>(config: &Config, writer: &mut W) -> Result<QuerySummary, QuartzError> {
    let filter = Filter::parse(config.filter()).map_err(QuartzError::InvalidFilter)?;
    let mut documents = 0;

    if config.files().is_empty() {
        let stdin = io::stdin();
        query_source(&filter, config.compact(), stdin.lock(), None, writer)?;
        documents += 1;
    } else {
        for path in config.files() {
            let file = open_file(path)?;
            query_source(
                &filter,
                config.compact(),
                file,
                Some(path.as_path()),
                writer,
            )?;
            documents += 1;
        }
    }

    Ok(QuerySummary { documents })
}

fn open_file(path: &Path) -> Result<File, QuartzError> {
    File::open(path).map_err(|source| QuartzError::OpenFile {
        path: path.to_path_buf(),
        source,
    })
}

fn query_source<R, W>(
    filter: &Filter,
    compact: bool,
    mut reader: R,
    path: Option<&Path>,
    writer: &mut W,
) -> Result<(), QuartzError>
where
    R: Read,
    W: Write,
{
    let mut bytes = Vec::new();
    reader
        .read_to_end(&mut bytes)
        .map_err(|source| QuartzError::ReadInput {
            path: path.map(Path::to_path_buf),
            source,
        })?;

    let value: Value =
        serde_json::from_slice(&bytes).map_err(|error| QuartzError::InvalidJson {
            path: path.map(Path::to_path_buf),
            message: error.to_string(),
        })?;

    let selected = query::apply(filter, &value);
    query::write_value(&selected, compact, writer).map_err(QuartzError::WriteOutput)
}
