//! Structured application errors and documented exit statuses.

use std::error::Error;
use std::fmt;
use std::io;

/// The documented command syntax.
pub const USAGE: &str = "usage: resolver <name>";

/// A failure at Resolver's application boundary.
#[derive(Debug)]
pub enum ResolverError {
    /// The command-line shape does not match the version 0.1 contract.
    Usage,
    /// The host supplied a name that cannot be represented as Unicode.
    NameIsNotUnicode,
    /// The operating-system resolver could not resolve the name.
    Lookup(io::Error),
    /// Resolution succeeded but produced no addresses.
    NoAddresses,
    /// Addresses could not be written to stdout.
    WriteOutput(io::Error),
}

impl ResolverError {
    /// Returns the documented process exit status.
    #[must_use]
    pub const fn exit_code(&self) -> u8 {
        match self {
            Self::Lookup(_) | Self::NoAddresses => 1,
            Self::Usage | Self::NameIsNotUnicode | Self::WriteOutput(_) => 2,
        }
    }
}

impl fmt::Display for ResolverError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage => formatter.write_str(USAGE),
            Self::NameIsNotUnicode => formatter.write_str("name is not valid Unicode"),
            Self::Lookup(source) => write!(formatter, "cannot resolve name: {source}"),
            Self::NoAddresses => formatter.write_str("no addresses found"),
            Self::WriteOutput(source) => write!(formatter, "cannot write output: {source}"),
        }
    }
}

impl Error for ResolverError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Lookup(source) | Self::WriteOutput(source) => Some(source),
            Self::Usage | Self::NameIsNotUnicode | Self::NoAddresses => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io;

    use super::ResolverError;

    #[test]
    fn maps_failures_to_documented_exit_codes() {
        assert_eq!(ResolverError::Usage.exit_code(), 2);
        assert_eq!(ResolverError::NameIsNotUnicode.exit_code(), 2);
        assert_eq!(
            ResolverError::WriteOutput(io::Error::other("write")).exit_code(),
            2
        );
        assert_eq!(
            ResolverError::Lookup(io::Error::other("lookup")).exit_code(),
            1
        );
        assert_eq!(ResolverError::NoAddresses.exit_code(), 1);
    }
}
