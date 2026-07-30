//! Parsing for Resolver's fixed version 0.1 command-line contract.

use std::ffi::OsString;

use crate::error::ResolverError;

/// Validated command-line input for one lookup.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Config {
    name: String,
}

impl Config {
    /// Creates a lookup configuration from a Unicode name.
    #[must_use]
    pub fn new(name: String) -> Self {
        Self { name }
    }

    /// Returns the hostname or IP literal to resolve.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Parses arguments after the executable name.
///
/// # Errors
///
/// Returns a usage error unless there is exactly one argument, or a name
/// encoding error when the argument cannot be represented as Unicode.
pub fn parse_args<I>(args: I) -> Result<Config, ResolverError>
where
    I: IntoIterator<Item = OsString>,
{
    let mut args = args.into_iter();
    let name = args.next().ok_or(ResolverError::Usage)?;

    if args.next().is_some() {
        return Err(ResolverError::Usage);
    }

    let name = name
        .into_string()
        .map_err(|_| ResolverError::NameIsNotUnicode)?;
    Ok(Config::new(name))
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use super::parse_args;
    use crate::error::ResolverError;

    #[test]
    fn parses_one_name() {
        let config = parse_args([OsString::from("example.com")]).expect("one name should parse");

        assert_eq!(config.name(), "example.com");
    }

    #[test]
    fn rejects_missing_name() {
        let error = parse_args::<[OsString; 0]>([]).expect_err("a missing name should fail");

        assert!(matches!(error, ResolverError::Usage));
    }

    #[test]
    fn rejects_extra_arguments() {
        let error = parse_args([OsString::from("example.com"), OsString::from("extra")])
            .expect_err("an extra argument should fail");

        assert!(matches!(error, ResolverError::Usage));
    }
}
