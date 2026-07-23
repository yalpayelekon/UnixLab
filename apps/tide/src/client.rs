//! HTTP policy and streaming response transfer.

use std::io::{Read, Write};
use std::time::Duration;

use crate::error::TideError;

const GLOBAL_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_REDIRECTS: u32 = 10;
const TRANSFER_BUFFER_SIZE: usize = 16 * 1024;
const USER_AGENT: &str = concat!("tide/", env!("CARGO_PKG_VERSION"));

/// Metadata from a completed response transfer.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct TransferSummary {
    /// Final HTTP response status after redirects.
    pub status: u16,
    /// Number of response-body bytes written.
    pub bytes_transferred: u64,
}

/// Sends one GET request and streams the response body into `writer`.
///
/// # Errors
///
/// Returns an error for unsupported URLs, request failures, response-body read
/// failures, or output write failures.
pub fn get<W: Write>(url: &str, writer: &mut W) -> Result<TransferSummary, TideError> {
    validate_url(url)?;

    let config = ureq::Agent::config_builder()
        .timeout_global(Some(GLOBAL_TIMEOUT))
        .max_redirects(MAX_REDIRECTS)
        .http_status_as_error(false)
        .build();
    let agent: ureq::Agent = config.into();
    let response = agent
        .get(url)
        .header("User-Agent", USER_AGENT)
        .call()
        .map_err(TideError::Request)?;
    let status = response.status().as_u16();
    let (_, body) = response.into_parts();
    let mut reader = body.into_reader();
    let bytes_transferred = transfer(&mut reader, writer)?;

    Ok(TransferSummary {
        status,
        bytes_transferred,
    })
}

fn validate_url(url: &str) -> Result<(), TideError> {
    let Some((scheme, remainder)) = url.split_once("://") else {
        return Err(TideError::InvalidUrl);
    };

    if (!scheme.eq_ignore_ascii_case("http") && !scheme.eq_ignore_ascii_case("https"))
        || remainder.is_empty()
    {
        return Err(TideError::InvalidUrl);
    }

    Ok(())
}

fn transfer<R: Read, W: Write>(reader: &mut R, writer: &mut W) -> Result<u64, TideError> {
    let mut buffer = [0_u8; TRANSFER_BUFFER_SIZE];
    let mut transferred = 0_u64;

    loop {
        let bytes_read = reader.read(&mut buffer).map_err(TideError::ReadBody)?;
        if bytes_read == 0 {
            return Ok(transferred);
        }

        writer
            .write_all(&buffer[..bytes_read])
            .map_err(TideError::WriteOutput)?;
        let bytes_read =
            u64::try_from(bytes_read).expect("transfer buffer length always fits into u64");
        transferred += bytes_read;
    }
}

#[cfg(test)]
mod tests {
    use std::io::{self, Cursor, Read, Write};

    use super::{TRANSFER_BUFFER_SIZE, transfer, validate_url};
    use crate::error::TideError;

    #[test]
    fn accepts_http_and_https_urls_case_insensitively() {
        assert!(validate_url("http://example.com").is_ok());
        assert!(validate_url("HTTPS://example.com/path").is_ok());
    }

    #[test]
    fn rejects_missing_and_unsupported_schemes() {
        assert!(matches!(
            validate_url("example.com"),
            Err(TideError::InvalidUrl)
        ));
        assert!(matches!(
            validate_url("ftp://example.com"),
            Err(TideError::InvalidUrl)
        ));
        assert!(matches!(
            validate_url("https://"),
            Err(TideError::InvalidUrl)
        ));
    }

    #[test]
    fn transfers_binary_data_exactly() {
        let input = [0_u8, 1, 2, 0xff, b'\n'];
        let mut reader = Cursor::new(input);
        let mut output = Vec::new();

        let count = transfer(&mut reader, &mut output).expect("transfer should succeed");

        assert_eq!(count, 5);
        assert_eq!(output, input);
    }

    #[test]
    fn streams_bodies_larger_than_the_buffer() {
        let input = vec![0x5a; TRANSFER_BUFFER_SIZE * 3 + 7];
        let mut reader = Cursor::new(&input);
        let mut output = Vec::new();

        let count = transfer(&mut reader, &mut output).expect("transfer should succeed");

        assert_eq!(
            count,
            u64::try_from(input.len()).expect("length should fit")
        );
        assert_eq!(output, input);
    }

    #[test]
    fn classifies_body_read_failures() {
        struct FailingReader;

        impl Read for FailingReader {
            fn read(&mut self, _buffer: &mut [u8]) -> io::Result<usize> {
                Err(io::Error::new(io::ErrorKind::UnexpectedEof, "truncated"))
            }
        }

        let error = transfer(&mut FailingReader, &mut Vec::new())
            .expect_err("read failure should propagate");

        assert!(matches!(error, TideError::ReadBody(_)));
    }

    #[test]
    fn classifies_output_write_failures() {
        struct FailingWriter;

        impl Write for FailingWriter {
            fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
                Err(io::Error::new(io::ErrorKind::BrokenPipe, "closed"))
            }

            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }

        let error = transfer(&mut Cursor::new(b"response"), &mut FailingWriter)
            .expect_err("write failure should propagate");

        assert!(matches!(error, TideError::WriteOutput(_)));
    }
}
