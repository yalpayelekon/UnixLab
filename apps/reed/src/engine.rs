//! Line-cycle engine that applies a compiled Reed script.

use std::fmt;
use std::io::{self, BufRead, Write};

use crate::address::{LineState, RangeState};
use crate::script::{CommandKind, Script};

/// Counts produced by a completed transform.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct TransformSummary {
    /// Number of logical input lines examined.
    pub lines_processed: u64,
    /// Number of lines written to the output.
    pub lines_written: u64,
}

/// A streaming failure classified by input or output direction.
#[derive(Debug)]
pub enum EngineError {
    /// Reading or UTF-8 decoding failed.
    Read(io::Error),
    /// Writing transformed text failed.
    Write(io::Error),
}

impl fmt::Display for EngineError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read(source) => write!(formatter, "input read failed: {source}"),
            Self::Write(source) => write!(formatter, "output write failed: {source}"),
        }
    }
}

impl std::error::Error for EngineError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Read(source) | Self::Write(source) => Some(source),
        }
    }
}

/// Mutable execution state shared across one or more input sources.
pub struct Engine<'a> {
    script: &'a Script,
    quiet: bool,
    range_states: Vec<RangeState>,
    summary: TransformSummary,
}

impl<'a> Engine<'a> {
    /// Creates engine state for `script`.
    #[must_use]
    pub fn new(script: &'a Script, quiet: bool) -> Self {
        Self {
            script,
            quiet,
            range_states: script
                .commands()
                .iter()
                .map(|_| RangeState::new())
                .collect(),
            summary: TransformSummary {
                lines_processed: 0,
                lines_written: 0,
            },
        }
    }

    /// Returns the accumulated summary.
    #[must_use]
    pub const fn summary(&self) -> TransformSummary {
        self.summary
    }

    /// Streams lines from `reader` without last-line semantics.
    ///
    /// # Errors
    ///
    /// Returns [`EngineError::Read`] or [`EngineError::Write`] on I/O failure.
    pub fn transform_streaming<R: BufRead, W: Write>(
        &mut self,
        mut reader: R,
        writer: &mut W,
    ) -> Result<(), EngineError> {
        let mut line = String::new();
        loop {
            line.clear();
            let bytes_read = reader.read_line(&mut line).map_err(EngineError::Read)?;
            if bytes_read == 0 {
                break;
            }
            self.summary.lines_processed += 1;
            trim_line_ending(&mut line);
            let state = LineState {
                line_no: self.summary.lines_processed,
                is_last: false,
            };
            apply_cycle(
                self.script,
                &mut self.range_states,
                &mut line,
                state,
                self.quiet,
                writer,
                &mut self.summary,
            )?;
        }
        Ok(())
    }

    /// Buffers every line from `reader`, then applies the script with accurate
    /// `$` semantics for this batch.
    ///
    /// Line numbers continue from any previously processed input.
    ///
    /// # Errors
    ///
    /// Returns [`EngineError::Read`] or [`EngineError::Write`] on I/O failure.
    pub fn transform_buffered<R: BufRead, W: Write>(
        &mut self,
        mut reader: R,
        writer: &mut W,
    ) -> Result<(), EngineError> {
        let lines = read_all_lines(&mut reader)?;
        let total = lines.len() as u64;
        let base = self.summary.lines_processed;

        for (offset, original) in lines.into_iter().enumerate() {
            let line_no = base + offset as u64 + 1;
            self.summary.lines_processed = line_no;
            let mut line = original;
            let state = LineState {
                line_no,
                is_last: line_no - base == total,
            };
            apply_cycle(
                self.script,
                &mut self.range_states,
                &mut line,
                state,
                self.quiet,
                writer,
                &mut self.summary,
            )?;
        }
        Ok(())
    }
}

/// Applies `script` to UTF-8 lines from `reader`, writing results to `writer`.
///
/// When the script uses `$`, all lines are buffered so last-line addresses are
/// exact. Otherwise the engine streams one line at a time.
///
/// # Errors
///
/// Returns [`EngineError::Read`] for I/O and UTF-8 input failures, or
/// [`EngineError::Write`] when output fails.
pub fn transform_reader<R: BufRead, W: Write>(
    script: &Script,
    reader: R,
    writer: &mut W,
    quiet: bool,
) -> Result<TransformSummary, EngineError> {
    let mut engine = Engine::new(script, quiet);
    if script.needs_last_line() {
        engine.transform_buffered(reader, writer)?;
    } else {
        engine.transform_streaming(reader, writer)?;
    }
    Ok(engine.summary())
}

fn apply_cycle<W: Write>(
    script: &Script,
    range_states: &mut [RangeState],
    pattern_space: &mut String,
    state: LineState,
    quiet: bool,
    writer: &mut W,
    summary: &mut TransformSummary,
) -> Result<(), EngineError> {
    let mut suppress_auto_print = false;

    for (command, range_state) in script.commands().iter().zip(range_states.iter_mut()) {
        if !range_state.matches(&command.address, pattern_space, state) {
            continue;
        }

        match &command.kind {
            CommandKind::Print => {
                write_line(writer, pattern_space)?;
                summary.lines_written += 1;
            }
            CommandKind::Delete => {
                suppress_auto_print = true;
                break;
            }
            CommandKind::Substitute {
                pattern,
                replacement,
                flags,
            } => {
                let replaced = if flags.global {
                    pattern.replace_all(pattern_space, replacement.as_str())
                } else {
                    pattern.replace(pattern_space, replacement.as_str())
                };
                let changed = replaced.as_ref() != pattern_space.as_str();
                if changed {
                    *pattern_space = replaced.into_owned();
                }
                if flags.print && changed {
                    write_line(writer, pattern_space)?;
                    summary.lines_written += 1;
                }
            }
        }
    }

    if !quiet && !suppress_auto_print {
        write_line(writer, pattern_space)?;
        summary.lines_written += 1;
    }

    Ok(())
}

fn read_all_lines<R: BufRead>(reader: &mut R) -> Result<Vec<String>, EngineError> {
    let mut lines = Vec::new();
    let mut line = String::new();
    loop {
        line.clear();
        let bytes_read = reader.read_line(&mut line).map_err(EngineError::Read)?;
        if bytes_read == 0 {
            break;
        }
        trim_line_ending(&mut line);
        lines.push(std::mem::take(&mut line));
    }
    Ok(lines)
}

fn write_line<W: Write>(writer: &mut W, line: &str) -> Result<(), EngineError> {
    writeln!(writer, "{line}").map_err(EngineError::Write)
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
fn transform_string(
    script_source: &str,
    input: &str,
    quiet: bool,
) -> Result<(String, TransformSummary), EngineError> {
    let script = Script::compile(&[script_source.to_owned()]).expect("test script");
    let mut output = Vec::new();
    let summary = transform_reader(
        &script,
        std::io::Cursor::new(input.as_bytes()),
        &mut output,
        quiet,
    )?;
    let text = String::from_utf8(output).expect("utf-8 output");
    Ok((text, summary))
}

#[cfg(test)]
mod tests {
    use std::io::{self, Cursor, Write};

    use super::{EngineError, transform_reader, transform_string};
    use crate::script::Script;

    #[test]
    fn substitutes_first_and_global() {
        let (output, _) = transform_string("s/a/A/", "a a\n", false).expect("ok");
        assert_eq!(output, "A a\n");

        let (output, _) = transform_string("s/a/A/g", "a a\n", false).expect("ok");
        assert_eq!(output, "A A\n");
    }

    #[test]
    fn quiet_mode_with_print_flag() {
        let (output, _) = transform_string("s/a/A/p", "a\nb\n", true).expect("ok");
        assert_eq!(output, "A\n");
    }

    #[test]
    fn deletes_matching_lines() {
        let (output, _) = transform_string("/drop/d", "keep\ndrop\nkeep\n", false).expect("ok");
        assert_eq!(output, "keep\nkeep\n");
    }

    #[test]
    fn applies_line_range() {
        let (output, _) = transform_string("2,3d", "one\ntwo\nthree\nfour\n", false).expect("ok");
        assert_eq!(output, "one\nfour\n");
    }

    #[test]
    fn last_line_address_buffers_input() {
        let script = Script::compile(&[String::from(r"$s/$/!/")]).expect("compile");
        assert!(script.needs_last_line());
        let mut output = Vec::new();
        transform_reader(
            &script,
            Cursor::new(b"one\ntwo\nthree\n"),
            &mut output,
            false,
        )
        .expect("ok");
        assert_eq!(output, b"one\ntwo\nthree!\n");
    }

    #[test]
    fn normalizes_crlf() {
        let (output, summary) = transform_string("s/a/A/", "a\r\nb\n", false).expect("ok");
        assert_eq!(summary.lines_processed, 2);
        assert_eq!(output, "A\nb\n");
    }

    #[test]
    fn rejects_invalid_utf8() {
        let script = Script::compile(&[String::from("s/a/b/")]).expect("compile");
        let error = transform_reader(&script, Cursor::new([0xff, b'\n']), &mut Vec::new(), false)
            .expect_err("invalid utf-8");
        assert!(matches!(error, EngineError::Read(_)));
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

        let script = Script::compile(&[String::from("p")]).expect("compile");
        let error = transform_reader(&script, Cursor::new(b"line\n"), &mut FailingWriter, true)
            .expect_err("write failure");
        assert!(matches!(error, EngineError::Write(_)));
    }
}
