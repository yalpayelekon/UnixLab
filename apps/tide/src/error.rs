//! Structured transfer failures and curl-inspired exit statuses.

use std::error::Error;
use std::fmt;
use std::io;

/// The documented command syntax.
pub const USAGE: &str = "usage: tide <url>";

/// A failure at Tide's application boundary.
#[derive(Debug)]
pub enum TideError {
    /// The command-line shape does not match the version 0.1 contract.
    Usage,
    /// The host supplied a URL that cannot be represented as Unicode.
    UrlIsNotUnicode,
    /// The URL is missing a supported HTTP or HTTPS scheme.
    InvalidUrl,
    /// Request setup, DNS, connection, redirect, timeout, or TLS failed.
    Request(ureq::Error),
    /// Reading the response body failed.
    ReadBody(io::Error),
    /// Writing response bytes to stdout failed.
    WriteOutput(io::Error),
}

impl TideError {
    /// Returns the documented curl-inspired process exit status.
    #[must_use]
    pub fn exit_code(&self) -> u8 {
        match self {
            Self::Usage => 2,
            Self::UrlIsNotUnicode | Self::InvalidUrl | Self::Request(ureq::Error::BadUri(_)) => 3,
            Self::Request(ureq::Error::HostNotFound) => 6,
            Self::Request(ureq::Error::Timeout(_)) => 28,
            Self::Request(
                ureq::Error::Tls(_) | ureq::Error::Rustls(_) | ureq::Error::TlsRequired,
            ) => 35,
            Self::Request(ureq::Error::TooManyRedirects) => 47,
            Self::Request(_) => 7,
            Self::ReadBody(_) => 18,
            Self::WriteOutput(_) => 23,
        }
    }
}

impl fmt::Display for TideError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage => formatter.write_str(USAGE),
            Self::UrlIsNotUnicode => formatter.write_str("URL is not valid Unicode"),
            Self::InvalidUrl => formatter.write_str("URL must use http:// or https://"),
            Self::Request(source) => write!(formatter, "request failed: {source}"),
            Self::ReadBody(source) => write!(formatter, "response body failed: {source}"),
            Self::WriteOutput(source) => write!(formatter, "cannot write output: {source}"),
        }
    }
}

impl Error for TideError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Request(source) => Some(source),
            Self::ReadBody(source) | Self::WriteOutput(source) => Some(source),
            Self::Usage | Self::UrlIsNotUnicode | Self::InvalidUrl => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io;

    use super::TideError;

    #[test]
    fn maps_local_failures_to_documented_exit_codes() {
        assert_eq!(TideError::Usage.exit_code(), 2);
        assert_eq!(TideError::InvalidUrl.exit_code(), 3);
        assert_eq!(
            TideError::ReadBody(io::Error::other("read")).exit_code(),
            18
        );
        assert_eq!(
            TideError::WriteOutput(io::Error::other("write")).exit_code(),
            23
        );
    }

    #[test]
    fn maps_request_failures_to_curl_inspired_exit_codes() {
        assert_eq!(TideError::Request(ureq::Error::HostNotFound).exit_code(), 6);
        assert_eq!(
            TideError::Request(ureq::Error::ConnectionFailed).exit_code(),
            7
        );
        assert_eq!(
            TideError::Request(ureq::Error::Tls("handshake")).exit_code(),
            35
        );
        assert_eq!(
            TideError::Request(ureq::Error::TooManyRedirects).exit_code(),
            47
        );
    }
}
