//! Rendering of matching lines and optional terminal color.

use std::io::{self, Write};

use crate::matcher::Matcher;

const MATCH_START: &[u8] = b"\x1b[1;31m";
const MATCH_END: &[u8] = b"\x1b[0m";

/// Writes one matching line, highlighting visible matches when requested.
///
/// A newline is always emitted, producing consistent output for LF, CRLF, and
/// unterminated final input lines.
///
/// # Errors
///
/// Returns the first error reported by the output writer.
pub fn write_matching_line<W: Write>(
    writer: &mut W,
    line: &str,
    matcher: &Matcher,
    color: bool,
) -> io::Result<()> {
    if !color {
        writeln!(writer, "{line}")?;
        return Ok(());
    }

    let mut previous_end = 0;
    for matched in matcher.matches(line) {
        if matched.is_empty() {
            continue;
        }

        writer.write_all(&line.as_bytes()[previous_end..matched.start()])?;
        writer.write_all(MATCH_START)?;
        writer.write_all(matched.as_str().as_bytes())?;
        writer.write_all(MATCH_END)?;
        previous_end = matched.end();
    }

    writer.write_all(&line.as_bytes()[previous_end..])?;
    writer.write_all(b"\n")
}

#[cfg(test)]
mod tests {
    use super::write_matching_line;
    use crate::matcher::Matcher;

    #[test]
    fn writes_plain_matching_line() {
        let matcher = Matcher::new("needle").expect("pattern should compile");
        let mut output = Vec::new();

        write_matching_line(&mut output, "find a needle", &matcher, false)
            .expect("rendering should succeed");

        assert_eq!(output, b"find a needle\n");
    }

    #[test]
    fn highlights_each_non_empty_match() {
        let matcher = Matcher::new("red").expect("pattern should compile");
        let mut output = Vec::new();

        write_matching_line(&mut output, "red and red", &matcher, true)
            .expect("rendering should succeed");

        assert_eq!(output, b"\x1b[1;31mred\x1b[0m and \x1b[1;31mred\x1b[0m\n");
    }

    #[test]
    fn zero_width_match_does_not_change_visible_output() {
        let matcher = Matcher::new("^").expect("pattern should compile");
        let mut output = Vec::new();

        write_matching_line(&mut output, "needle", &matcher, true)
            .expect("rendering should succeed");

        assert_eq!(output, b"needle\n");
    }
}
