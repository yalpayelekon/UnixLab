# Quartz 0.1 design

- Status: Implemented
- Application: `quartz`
- Inspiration: jq
- Compatibility target: jq-inspired JSON path query with compact or
  pretty output, using `serde_json` for parsing and encoding

## Scope

Quartz 0.1 accepts:

```text
quartz [-c] [filter] [file...]
```

- `-c` selects compact JSON output; the default is pretty-printed with a
  two-space indent.
- A positional argument that begins with `.` is the filter. Any earlier
  non-option positionals are not accepted; remaining arguments are input
  files.
- If no filter argument is present, the filter defaults to `.`.
- Zero files means read standard input as one UTF-8 JSON value.
- One or more files are each read as one JSON value; results are written in
  argument order.

The filter language supports:

| Feature | Behavior |
| --- | --- |
| `.` | Identity: the whole input value |
| `.key` | Object field access; keys are identifiers `[A-Za-z_][A-Za-z0-9_]*` |
| `.key.nested` | Nested object field access |
| `.key[N]` / `.[N]` | Non-negative array index access |
| Missing path | JSON `null` (jq-like), still exit status 0 |

The following are deliberately out of scope:

- Pipes (`|`), iterators (`.[]`), operators, and functions
- Quoted or non-identifier object keys
- Negative array indices
- NDJSON / multi-value JSON streams within one input
- YAML, in-place editing, and colored output

Each requires a separate proposal.

## Command contract

Filters must be valid Unicode supplied by the host command-line API. File
paths remain `OsString` / `PathBuf`, so non-Unicode paths are supported on
platforms that allow them. A path that begins with `.` is treated as a
filter; open such files with an explicit relative form such as `./.hidden`.

Quartz reads each input as one UTF-8 JSON document. Invalid UTF-8 or invalid
JSON is an explicit error. Memory use is proportional to the size of each
input document because the value is loaded fully before querying.

Diagnostics use `quartz: <message>` on stderr. Encoded JSON is written only
to stdout. Each result is followed by a trailing newline.

## Exit statuses

| Status | Meaning |
| --- | --- |
| 0 | Query applied successfully (including paths that resolve to `null`) |
| 1 | Usage, filter parse, JSON parse, open, read/decode, or write failure |

## Architecture

`main.rs` is the process boundary. It gathers OS arguments, opens stdin or
files, locks stdout, delegates to the library, prints a diagnostic on failure,
and maps the outcome to an exit status.

The library has focused modules:

- `cli`: validates `-c`, the optional filter, and the file list without a CLI
  framework.
- `error`: preserves structured causes and produces user-facing diagnostics.
- `filter`: parses path expressions into a compiled segment list.
- `query`: applies a filter to a `serde_json::Value` and encodes output.
- `lib.rs`: exposes configuration, summary, and orchestration.

## Design decisions

### Use serde_json

Implementing a correct JSON parser and encoder is outside Quartz's purpose.
`serde_json` is widely maintained, portable, and keeps the dependency budget
to one crate for the hard problem—the same role `regex` plays for Needle and
Reed.

### Parse the 0.1 CLI and filters directly

The option surface is a single flag and a small path language. Hand-written
parsers keep dependencies minimal and diagnostics local. A CLI framework can
be reconsidered when the option surface grows.

### Treat leading-dot positionals as filters

Defaulting the filter to `.` makes `quartz file.json` useful. Distinguishing
filters by a leading `.` avoids requiring the filter always, while remaining
predictable for jq-like expressions.

### Keep domain errors local

Quartz defines usage, filter, JSON, open, read, and write failures locally.
Existing apps do not yet share a concrete API that justifies moving types into
`unixlab-cli` or `unixlab-errors`.

### Use stable, dependency-free benchmarks

The benchmark target uses `std::hint::black_box` and `Instant` under Cargo's
stable custom harness, measuring the in-memory path-query path without adding
a benchmarking framework.

## Testing strategy

Unit tests cover argument validation, filter parsing, path evaluation on
objects/arrays/missing keys, pretty versus compact encoding, invalid UTF-8,
and I/O failure propagation.

Integration tests execute the real binary and verify identity, nested paths,
array indices, `-c`, stdin, multi-file order, invalid JSON, invalid filters,
missing files, usage outcomes, stdout/stderr separation, and exit statuses.

CI runs the suite on Linux, macOS, and Windows. The benchmark is compiled by
Clippy's all-targets pass and can be run with:

```sh
cargo bench -p quartz
```

## Future changes

Iterators, pipes, functions, quoted keys, and NDJSON streaming must remain
separate proposals. Broader jq compatibility should not expand the 0.1 filter
language implicitly. Shared JSON helpers belong in `libs/` only after another
consumer needs the same contract.
