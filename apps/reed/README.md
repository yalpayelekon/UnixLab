# Reed

Reed is a modern, cross-platform sed-inspired stream editor from UnixLab.

Version 0.1 applies a small script language to UTF-8 text from files or
standard input:

```text
reed [-n] [-e script]... [script] [file...]
```

Supported commands are `s` (substitute), `p` (print), and `d` (delete), with
optional addresses (`N`, `$`, `/regex/`, and `addr1,addr2`). Regular expressions
use the Rust `regex` dialect, not GNU sed BRE/ERE.

## Examples

```sh
reed 's/foo/bar/' notes.txt
reed -n 's/error/ERROR/p' app.log
reed '/^#/d' config.txt
reed -n '2,4p' lines.txt
reed -e 's/a/A/' -e 's/b/B/' input.txt
reed 's/old/new/g' < stream.txt
```

## Exit status

- `0`: script applied successfully
- `1`: usage, script, file, decoding, read, or output error

## Limitations

Reed 0.1 does not implement hold space, branches, in-place editing, script
files, or GNU/BSD sed dialect compatibility. Using `$` buffers the full input
for that invocation so last-line addresses are exact.

See [the 0.1 design document](../../docs/design/reed-0.1.md) for architecture,
decisions, and the testing strategy.
