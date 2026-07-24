//! Application library for the `reed` command.
//!
//! The library keeps process concerns out of the transform engine so parsing,
//! editing, and failure behavior can be tested without spawning a process.

pub mod address;
pub mod cli;
pub mod engine;
pub mod error;
pub mod script;

use std::fs::File;
use std::io::{self, BufReader, Write};
use std::path::Path;

pub use cli::Config;
pub use engine::TransformSummary;
pub use error::ReedError;
pub use script::Script;

use engine::Engine;

/// Runs Reed with a validated configuration.
///
/// # Errors
///
/// Returns an error when the script is invalid, an input cannot be opened or
/// read as UTF-8, or output cannot be written.
pub fn run<W: Write>(config: &Config, writer: &mut W) -> Result<TransformSummary, ReedError> {
    let script = Script::compile(config.scripts()).map_err(ReedError::InvalidScript)?;
    let mut engine = Engine::new(&script, config.quiet());

    if config.files().is_empty() {
        let stdin = io::stdin();
        transform_source(&mut engine, &script, stdin.lock(), None, writer)?;
    } else if script.needs_last_line() {
        let mut combined = Vec::new();
        for path in config.files() {
            let mut file = open_file(path)?;
            io::copy(&mut file, &mut combined).map_err(|source| ReedError::ReadInput {
                path: Some(path.clone()),
                source,
            })?;
        }
        transform_source(
            &mut engine,
            &script,
            io::Cursor::new(combined),
            None,
            writer,
        )?;
    } else {
        for path in config.files() {
            let file = open_file(path)?;
            transform_source(
                &mut engine,
                &script,
                BufReader::new(file),
                Some(path.as_path()),
                writer,
            )?;
        }
    }

    Ok(engine.summary())
}

fn open_file(path: &Path) -> Result<File, ReedError> {
    File::open(path).map_err(|source| ReedError::OpenFile {
        path: path.to_path_buf(),
        source,
    })
}

fn transform_source<R, W>(
    engine: &mut Engine<'_>,
    script: &Script,
    reader: R,
    path: Option<&Path>,
    writer: &mut W,
) -> Result<(), ReedError>
where
    R: io::BufRead,
    W: Write,
{
    let result = if script.needs_last_line() {
        engine.transform_buffered(reader, writer)
    } else {
        engine.transform_streaming(reader, writer)
    };
    result.map_err(|error| ReedError::from_engine_error(path.map(Path::to_path_buf), error))
}
