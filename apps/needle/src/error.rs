//! Structured application errors and user-facing diagnostics.

use std::error::Error;
use std::fmt;
use std::io;
use std::path::PathBuf;

use crate::search::SearchError;

/// The documented command syntax.
pub const USAGE: &str = "usage: needle <pattern> <file>";

/// A failure at Needle's application boundary.
#[derive(Debug)]
pub enum NeedleError {
    /// The command-line shape does not match the version 0.1 contract.
    Usage,
    /// The host supplied a pattern that cannot be represented as Unicode.
    PatternIsNotUnicode,
    /// The regular expression is invalid.
    InvalidPattern(regex::Error),
    /// The input file could not be opened.
    OpenFile {
        /// Path that could not be opened.
        path: PathBuf,
        /// Operating-system error returned while opening.
        source: io::Error,
    },
    /// The input file could not be read as UTF-8 text.
    ReadFile {
        /// Path that could not be read.
        path: PathBuf,
        /// I/O or UTF-8 validation error returned while reading.
        source: io::Error,
    },
    /// A matching line could not be written to stdout.
    WriteOutput(io::Error),
}

impl NeedleError {
    pub(crate) fn from_search_error(path: PathBuf, error: SearchError) -> Self {
        match error {
            SearchError::Read(source) => Self::ReadFile { path, source },
            SearchError::Write(source) => Self::WriteOutput(source),
        }
    }
}

impl fmt::Display for NeedleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage => formatter.write_str(USAGE),
            Self::PatternIsNotUnicode => formatter.write_str("pattern is not valid Unicode"),
            Self::InvalidPattern(source) => write!(formatter, "invalid pattern: {source}"),
            Self::OpenFile { path, source } => {
                write!(formatter, "cannot open '{}': {source}", path.display())
            }
            Self::ReadFile { path, source } => {
                write!(
                    formatter,
                    "cannot read '{}' as UTF-8 text: {source}",
                    path.display()
                )
            }
            Self::WriteOutput(source) => write!(formatter, "cannot write output: {source}"),
        }
    }
}

impl Error for NeedleError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidPattern(source) => Some(source),
            Self::OpenFile { source, .. }
            | Self::ReadFile { source, .. }
            | Self::WriteOutput(source) => Some(source),
            Self::Usage | Self::PatternIsNotUnicode => None,
        }
    }
}
