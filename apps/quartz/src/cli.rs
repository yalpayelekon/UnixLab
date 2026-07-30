//! Parsing for Quartz's version 0.1 command-line contract.

use std::ffi::OsString;
use std::path::PathBuf;

use crate::error::QuartzError;

/// Validated inputs for one Quartz invocation.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Config {
    compact: bool,
    filter: String,
    files: Vec<PathBuf>,
}

impl Config {
    /// Creates a configuration from validated options and inputs.
    #[must_use]
    pub fn new(compact: bool, filter: String, files: Vec<PathBuf>) -> Self {
        Self {
            compact,
            filter,
            files,
        }
    }

    /// Returns whether compact JSON encoding is requested (`-c`).
    #[must_use]
    pub const fn compact(&self) -> bool {
        self.compact
    }

    /// Returns the filter expression.
    #[must_use]
    pub fn filter(&self) -> &str {
        &self.filter
    }

    /// Returns input file paths. An empty slice means standard input.
    #[must_use]
    pub fn files(&self) -> &[PathBuf] {
        &self.files
    }
}

/// Parses arguments after the executable name.
///
/// # Errors
///
/// Returns a usage error when options are malformed, or when a leading-dot
/// filter cannot be represented as Unicode.
pub fn parse_args<I>(args: I) -> Result<Config, QuartzError>
where
    I: IntoIterator<Item = OsString>,
{
    let mut compact = false;
    let mut positional = Vec::new();
    let mut args = args.into_iter();

    while let Some(argument) = args.next() {
        let Some(text) = argument.to_str() else {
            positional.push(argument);
            positional.extend(args.by_ref());
            break;
        };

        match text {
            "-c" => compact = true,
            option if option.starts_with('-') && option != "-" => {
                return Err(QuartzError::Usage);
            }
            _ => {
                positional.push(argument);
                positional.extend(args.by_ref());
                break;
            }
        }
    }

    let mut positional = positional.into_iter();
    let filter;
    let mut files = Vec::new();

    match positional.next() {
        Some(first) => {
            let is_filter = first.to_str().is_some_and(|text| text.starts_with('.'));
            if is_filter {
                filter = first
                    .into_string()
                    .map_err(|_| QuartzError::FilterIsNotUnicode)?;
            } else {
                filter = String::from(".");
                files.push(PathBuf::from(first));
            }
        }
        None => filter = String::from("."),
    }

    files.extend(positional.map(PathBuf::from));

    Ok(Config::new(compact, filter, files))
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::path::Path;

    use super::parse_args;
    use crate::error::QuartzError;

    #[test]
    fn defaults_to_identity_on_stdin() {
        let config = parse_args::<[OsString; 0]>([]).expect("empty args should parse");

        assert!(!config.compact());
        assert_eq!(config.filter(), ".");
        assert!(config.files().is_empty());
    }

    #[test]
    fn treats_file_without_filter_as_identity() {
        let config = parse_args([OsString::from("data.json")]).expect("file-only should parse");

        assert_eq!(config.filter(), ".");
        assert_eq!(config.files(), [Path::new("data.json")]);
    }

    #[test]
    fn parses_filter_and_file() {
        let config = parse_args([OsString::from(".name"), OsString::from("data.json")])
            .expect("filter and file should parse");

        assert_eq!(config.filter(), ".name");
        assert_eq!(config.files(), [Path::new("data.json")]);
    }

    #[test]
    fn parses_compact_flag() {
        let config = parse_args([
            OsString::from("-c"),
            OsString::from("."),
            OsString::from("a.json"),
            OsString::from("b.json"),
        ])
        .expect("compact mode should parse");

        assert!(config.compact());
        assert_eq!(config.filter(), ".");
        assert_eq!(config.files(), [Path::new("a.json"), Path::new("b.json")]);
    }

    #[test]
    fn treats_dash_as_positional_path() {
        let config = parse_args([OsString::from("-")])
            .expect("dash should be a file path with default filter");

        assert_eq!(config.filter(), ".");
        assert_eq!(config.files(), [Path::new("-")]);
    }

    #[test]
    fn rejects_unknown_option() {
        let error = parse_args([OsString::from("-x")]).expect_err("unknown option");
        assert!(matches!(error, QuartzError::Usage));
    }
}
