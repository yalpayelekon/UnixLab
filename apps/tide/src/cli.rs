//! Parsing for Tide's fixed version 0.1 command-line contract.

use std::ffi::OsString;

use crate::error::TideError;

/// Validated command-line input for one GET request.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Config {
    url: String,
}

impl Config {
    /// Creates a request configuration from a Unicode URL.
    #[must_use]
    pub fn new(url: String) -> Self {
        Self { url }
    }

    /// Returns the requested URL.
    #[must_use]
    pub fn url(&self) -> &str {
        &self.url
    }
}

/// Parses arguments after the executable name.
///
/// # Errors
///
/// Returns a usage error unless there is exactly one argument, or a URL
/// encoding error when the host argument cannot be represented as Unicode.
pub fn parse_args<I>(args: I) -> Result<Config, TideError>
where
    I: IntoIterator<Item = OsString>,
{
    let mut args = args.into_iter();
    let url = args.next().ok_or(TideError::Usage)?;

    if args.next().is_some() {
        return Err(TideError::Usage);
    }

    let url = url.into_string().map_err(|_| TideError::UrlIsNotUnicode)?;
    Ok(Config::new(url))
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use super::parse_args;
    use crate::error::TideError;

    #[test]
    fn parses_one_url() {
        let config =
            parse_args([OsString::from("https://example.com/path")]).expect("one URL should parse");

        assert_eq!(config.url(), "https://example.com/path");
    }

    #[test]
    fn rejects_missing_url() {
        let error = parse_args([]).expect_err("a missing URL should fail");

        assert!(matches!(error, TideError::Usage));
    }

    #[test]
    fn rejects_extra_arguments() {
        let error = parse_args([
            OsString::from("https://example.com"),
            OsString::from("extra"),
        ])
        .expect_err("an extra argument should fail");

        assert!(matches!(error, TideError::Usage));
    }
}
