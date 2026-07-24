//! Structured application errors and user-facing diagnostics.

use std::error::Error;
use std::fmt;
use std::io;
use std::path::PathBuf;

use crate::engine::EngineError;

/// The documented command syntax.
pub const USAGE: &str = "usage: reed [-n] [-e script]... [script] [file...]";

/// A failure at Reed's application boundary.
#[derive(Debug)]
pub enum ReedError {
    /// The command-line shape does not match the version 0.1 contract.
    Usage,
    /// A script argument could not be represented as Unicode.
    ScriptIsNotUnicode,
    /// The script could not be parsed or a pattern could not be compiled.
    InvalidScript(String),
    /// An input file could not be opened.
    OpenFile {
        /// Path that could not be opened.
        path: PathBuf,
        /// Operating-system error returned while opening.
        source: io::Error,
    },
    /// An input source could not be read as UTF-8 text.
    ReadInput {
        /// Optional path for file inputs; `None` for standard input.
        path: Option<PathBuf>,
        /// I/O or UTF-8 validation error returned while reading.
        source: io::Error,
    },
    /// Transformed text could not be written to stdout.
    WriteOutput(io::Error),
}

impl ReedError {
    pub(crate) fn from_engine_error(path: Option<PathBuf>, error: EngineError) -> Self {
        match error {
            EngineError::Read(source) => Self::ReadInput { path, source },
            EngineError::Write(source) => Self::WriteOutput(source),
        }
    }
}

impl fmt::Display for ReedError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage => formatter.write_str(USAGE),
            Self::ScriptIsNotUnicode => formatter.write_str("script is not valid Unicode"),
            Self::InvalidScript(message) => write!(formatter, "invalid script: {message}"),
            Self::OpenFile { path, source } => {
                write!(formatter, "cannot open '{}': {source}", path.display())
            }
            Self::ReadInput {
                path: Some(path),
                source,
            } => write!(
                formatter,
                "cannot read '{}' as UTF-8 text: {source}",
                path.display()
            ),
            Self::ReadInput { path: None, source } => {
                write!(
                    formatter,
                    "cannot read standard input as UTF-8 text: {source}"
                )
            }
            Self::WriteOutput(source) => write!(formatter, "cannot write output: {source}"),
        }
    }
}

impl Error for ReedError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::OpenFile { source, .. }
            | Self::ReadInput { source, .. }
            | Self::WriteOutput(source) => Some(source),
            Self::Usage | Self::ScriptIsNotUnicode | Self::InvalidScript(_) => None,
        }
    }
}
