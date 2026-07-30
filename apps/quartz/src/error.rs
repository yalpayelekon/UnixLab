//! Structured application errors and user-facing diagnostics.

use std::error::Error;
use std::fmt;
use std::io;
use std::path::PathBuf;

/// The documented command syntax.
pub const USAGE: &str = "usage: quartz [-c] [filter] [file...]";

/// A failure at Quartz's application boundary.
#[derive(Debug)]
pub enum QuartzError {
    /// The command-line shape does not match the version 0.1 contract.
    Usage,
    /// A filter argument could not be represented as Unicode.
    FilterIsNotUnicode,
    /// The filter could not be parsed.
    InvalidFilter(String),
    /// An input file could not be opened.
    OpenFile {
        /// Path that could not be opened.
        path: PathBuf,
        /// Operating-system error returned while opening.
        source: io::Error,
    },
    /// An input source could not be read or decoded as UTF-8 JSON.
    ReadInput {
        /// Optional path for file inputs; `None` for standard input.
        path: Option<PathBuf>,
        /// I/O or decode error returned while reading.
        source: io::Error,
    },
    /// Input bytes were not valid JSON.
    InvalidJson {
        /// Optional path for file inputs; `None` for standard input.
        path: Option<PathBuf>,
        /// Parser message from `serde_json`.
        message: String,
    },
    /// Encoded JSON could not be written to stdout.
    WriteOutput(io::Error),
}

impl fmt::Display for QuartzError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage => formatter.write_str(USAGE),
            Self::FilterIsNotUnicode => formatter.write_str("filter is not valid Unicode"),
            Self::InvalidFilter(message) => write!(formatter, "invalid filter: {message}"),
            Self::OpenFile { path, source } => {
                write!(formatter, "cannot open '{}': {source}", path.display())
            }
            Self::ReadInput {
                path: Some(path),
                source,
            } => write!(formatter, "cannot read '{}': {source}", path.display()),
            Self::ReadInput { path: None, source } => {
                write!(formatter, "cannot read standard input: {source}")
            }
            Self::InvalidJson {
                path: Some(path),
                message,
            } => write!(
                formatter,
                "cannot parse '{}' as JSON: {message}",
                path.display()
            ),
            Self::InvalidJson {
                path: None,
                message,
            } => write!(formatter, "cannot parse standard input as JSON: {message}"),
            Self::WriteOutput(source) => write!(formatter, "cannot write output: {source}"),
        }
    }
}

impl Error for QuartzError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::OpenFile { source, .. }
            | Self::ReadInput { source, .. }
            | Self::WriteOutput(source) => Some(source),
            Self::Usage
            | Self::FilterIsNotUnicode
            | Self::InvalidFilter(_)
            | Self::InvalidJson { .. } => None,
        }
    }
}
