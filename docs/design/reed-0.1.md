# Reed 0.1 design

- Status: Implemented
- Application: `reed`
- Inspiration: sed
- Compatibility target: sed-inspired stream editing with addresses and
  `s` / `p` / `d`, using the Rust `regex` dialect

## Scope

Reed 0.1 accepts:

```text
reed [-n] [-e script]... [script] [file...]
```

- If any `-e` is present, there is no positional script; remaining arguments
  are input files.
- If no `-e` is present, the first non-option argument is the script; remaining
  arguments are input files.
- Zero files means read standard input as UTF-8 lines.
- Multiple files are processed in order as one continuous line stream: line
  numbers keep increasing across files.

The script language supports:

| Feature | Behavior |
| --- | --- |
| Addresses | empty (all lines), `N`, `$`, `/regex/`, `addr1,addr2` |
| `s` | Substitute with any single-character delimiter; flags `g` and `p` |
| `p` | Print the pattern space |
| `d` | Delete the pattern space, skip remaining commands, suppress auto-print |
| Separators | `;` and newlines between commands |

The following are deliberately out of scope:

- Hold space (`h` / `H` / `g` / `G` / `x`)
- Branches and labels (`b` / `t` / `:`)
- `y`, `i`, `a`, `c`, `n`, `N`, `w`, `=`
- In-place editing (`-i`)
- Script files (`-f`)
- Binary or NUL-delimited input
- GNU or BSD sed BRE/ERE dialect compatibility
- Exact GNU sed address edge-case fidelity

Each requires a separate proposal.

## Command contract

Scripts and regular-expression patterns must be valid Unicode supplied by the
host command-line API. File paths remain `OsString` / `PathBuf`, so
non-Unicode paths are supported on platforms that allow them.

Reed reads UTF-8 text. Invalid UTF-8 is an explicit error. CRLF and LF inputs
produce the same logical lines; output lines are terminated with `\n`.

The default cycle applies matching commands to the pattern space, then
auto-prints unless `-n` is set. The substitute `p` flag prints when a
replacement occurs, which is useful with `-n`.

Replacement text uses the Rust `regex` crate's replacement syntax (`$0`, `$1`,
`${name}`, and `$$` for a literal `$`).

Diagnostics use `reed: <message>` on stderr. Transformed text is written only
to stdout.

## Exit statuses

| Status | Meaning |
| --- | --- |
| 0 | Script applied successfully |
| 1 | Usage, script parse/compile, open, read/decode, or write failure |

## Architecture

`main.rs` is the process boundary. It gathers OS arguments, opens stdin or
files, locks stdout, delegates to the library, prints a diagnostic on failure,
and maps the outcome to an exit status.

The library has focused modules:

- `cli`: validates `-n`, repeated `-e`, the positional script, and file list
  without a CLI framework.
- `error`: preserves structured causes and produces user-facing diagnostics.
- `script`: parses scripts into compiled commands.
- `address`: compiles and evaluates addresses against line state.
- `engine`: runs the line cycle over a reader/writer boundary.
- `lib.rs`: exposes configuration, summary, and orchestration.

## Design decisions

### Use the regex crate

Implementing a correct regular-expression engine is outside Reed's purpose.
The `regex` crate matches Needle's dependency choice and provides linear-time
matching with Unicode support. Reed documents that it is not a GNU BRE/ERE
clone.

### Parse the 0.1 CLI and scripts directly

The option surface is small (`-n`, `-e`) and the script language is a fixed
subset. Hand-written parsers keep dependencies minimal and diagnostics local.
A CLI framework can be reconsidered when the option surface grows.

### Stream unless `$` is required

When the compiled script does not use `$`, Reed streams one line at a time so
memory stays proportional to the longest line. When `$` appears, Reed buffers
all input lines for that invocation so last-line semantics are correct, and
documents that `$` disables streaming.

### Keep domain errors local

Reed defines usage, script, open, read, and write failures locally. Needle and
Tide do not yet share a concrete API that justifies moving types into
`unixlab-cli` or `unixlab-errors`.

### Use stable, dependency-free benchmarks

The benchmark target uses `std::hint::black_box` and `Instant` under Cargo's
stable custom harness, measuring the in-memory substitute path without adding a
benchmarking framework.

## Testing strategy

Unit tests cover argument validation, script and address parsing, substitute
flags, delete/print cycles, quiet mode, `$` buffering, line-ending
normalization, invalid UTF-8, and I/O failure propagation.

Integration tests execute the real binary and verify substitute, `-n` with
`p`, addresses, ranges, multiple `-e`, stdin, multi-file order, invalid
scripts, missing files, invalid UTF-8, usage outcomes, stdout/stderr
separation, and exit statuses.

CI runs the suite on Linux, macOS, and Windows. The benchmark is compiled by
Clippy's all-targets pass and can be run with:

```sh
cargo bench -p reed
```

## Future changes

Hold space, branching, in-place editing, and script files must remain separate
proposals. Broader sed compatibility should not expand the 0.1 parser
implicitly. Streaming policy for `$` may later use peeking or length
precomputation when a documented workload requires it.
