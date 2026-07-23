//! Parsing for Needle's fixed version 0.1 command-line contract.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use crate::error::NeedleError;

/// Validated inputs for one Needle search.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Config {
    pattern: String,
    file: PathBuf,
}

impl Config {
    /// Creates a configuration from an already validated pattern and file path.
    #[must_use]
    pub fn new(pattern: String, file: PathBuf) -> Self {
        Self { pattern, file }
    }

    /// Returns the regular-expression source.
    #[must_use]
    pub fn pattern(&self) -> &str {
        &self.pattern
    }

    /// Returns the input file path.
    #[must_use]
    pub fn file(&self) -> &Path {
        &self.file
    }
}

/// Parses arguments after the executable name.
///
/// # Errors
///
/// Returns a usage error unless there are exactly two arguments, or when the
/// pattern cannot be represented as Unicode.
pub fn parse_args<I>(args: I) -> Result<Config, NeedleError>
where
    I: IntoIterator<Item = OsString>,
{
    let mut args = args.into_iter();
    let pattern = args.next().ok_or(NeedleError::Usage)?;
    let file = args.next().ok_or(NeedleError::Usage)?;

    if args.next().is_some() {
        return Err(NeedleError::Usage);
    }

    let pattern = pattern
        .into_string()
        .map_err(|_| NeedleError::PatternIsNotUnicode)?;

    Ok(Config::new(pattern, PathBuf::from(file)))
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::path::Path;

    use super::parse_args;
    use crate::error::NeedleError;

    #[test]
    fn parses_pattern_and_file() {
        let config = parse_args([OsString::from("error"), OsString::from("app.log")])
            .expect("valid arguments should parse");

        assert_eq!(config.pattern(), "error");
        assert_eq!(config.file(), Path::new("app.log"));
    }

    #[test]
    fn rejects_missing_arguments() {
        let error = parse_args([OsString::from("error")])
            .expect_err("a missing file should be a usage error");

        assert!(matches!(error, NeedleError::Usage));
    }

    #[test]
    fn rejects_extra_arguments() {
        let error = parse_args([
            OsString::from("error"),
            OsString::from("app.log"),
            OsString::from("extra"),
        ])
        .expect_err("an extra argument should be a usage error");

        assert!(matches!(error, NeedleError::Usage));
    }
}
