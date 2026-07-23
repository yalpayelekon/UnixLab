//! Streaming search orchestration over generic readers and writers.

use std::fmt;
use std::io::{self, BufRead, Write};

use crate::matcher::Matcher;
use crate::output::write_matching_line;

/// Counts produced by a completed search.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct SearchSummary {
    /// Number of logical input lines examined.
    pub lines_searched: u64,
    /// Number of matching lines written.
    pub matching_lines: u64,
}

impl SearchSummary {
    /// Reports whether at least one line matched.
    #[must_use]
    pub const fn found_match(self) -> bool {
        self.matching_lines > 0
    }
}

/// A streaming failure classified by input or output direction.
#[derive(Debug)]
pub enum SearchError {
    /// Reading or UTF-8 decoding failed.
    Read(io::Error),
    /// Writing a matching line failed.
    Write(io::Error),
}

impl fmt::Display for SearchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read(source) => write!(formatter, "input read failed: {source}"),
            Self::Write(source) => write!(formatter, "output write failed: {source}"),
        }
    }
}

impl std::error::Error for SearchError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Read(source) | Self::Write(source) => Some(source),
        }
    }
}

/// Searches UTF-8 lines from `reader` and writes matching lines to `writer`.
///
/// Memory use is proportional to the longest line. Both LF and CRLF endings
/// are normalized to a single LF in output.
///
/// # Errors
///
/// Returns [`SearchError::Read`] for I/O and UTF-8 input failures, or
/// [`SearchError::Write`] when output fails.
pub fn search_reader<R: BufRead, W: Write>(
    matcher: &Matcher,
    mut reader: R,
    writer: &mut W,
    color: bool,
) -> Result<SearchSummary, SearchError> {
    let mut summary = SearchSummary {
        lines_searched: 0,
        matching_lines: 0,
    };
    let mut line = String::new();

    loop {
        line.clear();
        let bytes_read = reader.read_line(&mut line).map_err(SearchError::Read)?;
        if bytes_read == 0 {
            break;
        }

        summary.lines_searched += 1;
        trim_line_ending(&mut line);

        if matcher.is_match(&line) {
            write_matching_line(writer, &line, matcher, color).map_err(SearchError::Write)?;
            summary.matching_lines += 1;
        }
    }

    Ok(summary)
}

fn trim_line_ending(line: &mut String) {
    if line.ends_with('\n') {
        line.pop();
        if line.ends_with('\r') {
            line.pop();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::{self, Cursor, Write};

    use super::{SearchError, search_reader};
    use crate::matcher::Matcher;

    #[test]
    fn searches_lf_crlf_and_unterminated_lines() {
        let matcher = Matcher::new("match").expect("pattern should compile");
        let input = Cursor::new(b"first\r\nmatch one\nlast match");
        let mut output = Vec::new();

        let summary =
            search_reader(&matcher, input, &mut output, false).expect("search should succeed");

        assert_eq!(summary.lines_searched, 3);
        assert_eq!(summary.matching_lines, 2);
        assert!(summary.found_match());
        assert_eq!(output, b"match one\nlast match\n");
    }

    #[test]
    fn reports_no_match() {
        let matcher = Matcher::new("absent").expect("pattern should compile");
        let input = Cursor::new(b"first\nsecond\n");
        let mut output = Vec::new();

        let summary =
            search_reader(&matcher, input, &mut output, false).expect("search should succeed");

        assert_eq!(summary.lines_searched, 2);
        assert_eq!(summary.matching_lines, 0);
        assert!(!summary.found_match());
        assert!(output.is_empty());
    }

    #[test]
    fn rejects_invalid_utf8() {
        let matcher = Matcher::new(".").expect("pattern should compile");
        let input = Cursor::new([0xff, b'\n']);
        let mut output = Vec::new();

        let error = search_reader(&matcher, input, &mut output, false)
            .expect_err("invalid UTF-8 should fail");

        assert!(matches!(error, SearchError::Read(_)));
    }

    #[test]
    fn reports_output_failure() {
        struct FailingWriter;

        impl Write for FailingWriter {
            fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
                Err(io::Error::new(io::ErrorKind::BrokenPipe, "closed"))
            }

            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }

        let matcher = Matcher::new("match").expect("pattern should compile");
        let input = Cursor::new(b"match\n");

        let error = search_reader(&matcher, input, &mut FailingWriter, false)
            .expect_err("output failure should propagate");

        assert!(matches!(error, SearchError::Write(_)));
    }
}
