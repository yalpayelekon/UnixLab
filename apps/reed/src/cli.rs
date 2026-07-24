//! Parsing for Reed's version 0.1 command-line contract.

use std::ffi::OsString;
use std::path::PathBuf;

use crate::error::ReedError;

/// Validated inputs for one Reed invocation.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Config {
    quiet: bool,
    scripts: Vec<String>,
    files: Vec<PathBuf>,
}

impl Config {
    /// Creates a configuration from validated options and scripts.
    #[must_use]
    pub fn new(quiet: bool, scripts: Vec<String>, files: Vec<PathBuf>) -> Self {
        Self {
            quiet,
            scripts,
            files,
        }
    }

    /// Returns whether automatic printing is suppressed (`-n`).
    #[must_use]
    pub const fn quiet(&self) -> bool {
        self.quiet
    }

    /// Returns the script fragments in order.
    #[must_use]
    pub fn scripts(&self) -> &[String] {
        &self.scripts
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
/// Returns a usage error when options are malformed or no script is provided,
/// or when a script cannot be represented as Unicode.
pub fn parse_args<I>(args: I) -> Result<Config, ReedError>
where
    I: IntoIterator<Item = OsString>,
{
    let mut quiet = false;
    let mut scripts = Vec::new();
    let mut used_e = false;
    let mut positional = Vec::new();
    let mut args = args.into_iter();

    while let Some(argument) = args.next() {
        let Some(text) = argument.to_str() else {
            if used_e {
                positional.push(argument);
                continue;
            }
            return Err(ReedError::ScriptIsNotUnicode);
        };

        match text {
            "-n" => quiet = true,
            "-e" => {
                used_e = true;
                let script = args.next().ok_or(ReedError::Usage)?;
                let script = script
                    .into_string()
                    .map_err(|_| ReedError::ScriptIsNotUnicode)?;
                if script.is_empty() {
                    return Err(ReedError::Usage);
                }
                scripts.push(script);
            }
            option if option.starts_with('-') && option != "-" => {
                return Err(ReedError::Usage);
            }
            _ => {
                positional.push(argument);
                positional.extend(args.by_ref());
                break;
            }
        }
    }

    if used_e {
        if scripts.is_empty() {
            return Err(ReedError::Usage);
        }
        let files = positional.into_iter().map(PathBuf::from).collect();
        return Ok(Config::new(quiet, scripts, files));
    }

    let mut positional = positional.into_iter();
    let script = positional.next().ok_or(ReedError::Usage)?;
    let script = script
        .into_string()
        .map_err(|_| ReedError::ScriptIsNotUnicode)?;
    if script.is_empty() {
        return Err(ReedError::Usage);
    }
    scripts.push(script);
    let files = positional.map(PathBuf::from).collect();

    Ok(Config::new(quiet, scripts, files))
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::path::Path;

    use super::parse_args;
    use crate::error::ReedError;

    #[test]
    fn parses_positional_script_and_file() {
        let config = parse_args([OsString::from("s/a/b/"), OsString::from("input.txt")])
            .expect("valid arguments should parse");

        assert!(!config.quiet());
        assert_eq!(config.scripts(), ["s/a/b/"]);
        assert_eq!(config.files(), [Path::new("input.txt")]);
    }

    #[test]
    fn parses_quiet_and_multiple_e_scripts() {
        let config = parse_args([
            OsString::from("-n"),
            OsString::from("-e"),
            OsString::from("s/a/b/"),
            OsString::from("-e"),
            OsString::from("p"),
            OsString::from("one.txt"),
            OsString::from("two.txt"),
        ])
        .expect("valid arguments should parse");

        assert!(config.quiet());
        assert_eq!(config.scripts(), ["s/a/b/", "p"]);
        assert_eq!(config.files(), [Path::new("one.txt"), Path::new("two.txt")]);
    }

    #[test]
    fn treats_dash_as_positional_path() {
        let config = parse_args([OsString::from("s/a/b/"), OsString::from("-")])
            .expect("dash should be a file path");

        assert_eq!(config.files(), [Path::new("-")]);
    }

    #[test]
    fn rejects_missing_script() {
        let error = parse_args::<[OsString; 0]>([]).expect_err("missing script should fail");
        assert!(matches!(error, ReedError::Usage));
    }

    #[test]
    fn rejects_e_without_script_value() {
        let error = parse_args([OsString::from("-e")]).expect_err("-e needs a value");
        assert!(matches!(error, ReedError::Usage));
    }

    #[test]
    fn rejects_unknown_option() {
        let error =
            parse_args([OsString::from("-x"), OsString::from("s/a/b/")]).expect_err("unknown");
        assert!(matches!(error, ReedError::Usage));
    }
}
