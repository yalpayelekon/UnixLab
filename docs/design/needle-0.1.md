# Needle 0.1 design

- Status: Implemented
- Application: `needle`
- Compatibility target: grep-style matching and exit status for one UTF-8 file

## Scope

Needle 0.1 accepts exactly:

```text
needle <pattern> <file>
```

`pattern` is a Rust `regex` regular expression. Needle reads `file` as UTF-8,
prints every matching line, and highlights non-empty matches when stdout is an
interactive terminal.

The following are deliberately out of scope:

- Recursive directory traversal
- Multiple files
- Standard-input search
- Fixed-string mode
- Context lines and line numbers
- User-selectable color policy
- Compressed files and memory mapping
- Exact GNU grep syntax or regex dialect compatibility

These features require separate proposals rather than hidden expansion of the
0.1 interface.

## Command contract

The pattern must be valid Unicode supplied by the host command-line API. The
file path remains an `OsString`/`PathBuf`, so non-Unicode paths are supported
on platforms that allow them.

Output is one matching logical line followed by `\n`. CRLF and LF inputs
produce the same logical output. Color uses ANSI SGR bold red only when stdout
is a terminal; redirected output remains plain text. Zero-width matches select
their line but are not highlighted because they contain no visible text.

Exit statuses match grep's core contract:

| Status | Meaning |
| --- | --- |
| 0 | At least one line matched |
| 1 | No lines matched |
| 2 | Usage, pattern, file, decoding, read, or output error |

Diagnostics use `needle: <message>` on stderr. Requested data is written only
to stdout.

## Architecture

`main.rs` is the process boundary. It gathers OS arguments, detects whether
stdout is a terminal, delegates to the library, prints a diagnostic on failure,
and maps the outcome to an exit status.

The library has five focused modules:

- `cli`: validates the fixed positional interface without introducing a CLI
  framework.
- `error`: preserves structured causes and produces user-facing diagnostics.
- `matcher`: compiles and owns the regular expression.
- `output`: renders matching lines with optional match highlighting.
- `search`: streams UTF-8 lines, coordinates matching and rendering, and
  returns a testable summary.

`lib.rs` exposes `run` as application orchestration and re-exports the small
set of types needed by tests and benchmarks.

## Design decisions

### Use the regex crate

Implementing a correct, denial-of-service-resistant regular-expression engine
is outside Needle's purpose. The `regex` crate provides a mature Unicode-aware
engine with linear-time matching guarantees. It is the only runtime
dependency.

### Parse the 0.1 CLI directly

The interface contains two positional values and no options. Standard-library
argument parsing is shorter and clearer than a CLI framework at this stage.
The decision can be revisited when real option parsing is proposed.

### Stream with `BufRead`

Needle processes one line at a time, keeping memory proportional to the longest
line instead of the file size. The generic reader/writer boundary makes search
behavior independently testable.

### Require UTF-8 explicitly

`BufRead::read_line` validates UTF-8 while streaming. Invalid data is an
explicit error rather than lossy replacement, matching the stated 0.1 input
contract.

### Color only interactive output

Automatic terminal detection prevents ANSI control sequences from corrupting
pipes and redirected files. Rendering receives an explicit color policy so it
can be tested without a pseudo-terminal.

### Keep domain errors local

Needle defines variants for usage, pattern, open, read, and write failures.
There is not yet a demonstrated cross-application contract for the shared
`errors` crate, so moving this type there would be premature.

### Use stable, dependency-free benchmarks

The benchmark target uses `std::hint::black_box` and `Instant` under Cargo's
stable custom benchmark harness. It measures the complete in-memory
search/render path without adding a benchmarking framework to production
dependencies.

## Testing strategy

Unit tests cover argument validation, valid and invalid regexes, Unicode
matching, zero-width behavior, colored rendering, line-ending normalization,
search summaries, invalid UTF-8, and I/O failure propagation.

Integration tests execute the real binary and verify matching, no-match,
invalid-pattern, missing-file, invalid-UTF-8, and usage outcomes, including
stdout/stderr separation and exit statuses.

CI runs the suite on Linux, macOS, and Windows. The benchmark is compiled by
Clippy's all-targets pass and can be run with:

```sh
cargo bench -p needle
```

## Future changes

Recursive search must remain a separate layer over single-input search. It
will require decisions about symlinks, ignore files, filesystem errors,
parallelism, deterministic output, and binary detection. Those concerns must
not leak into the 0.1 line matcher.

