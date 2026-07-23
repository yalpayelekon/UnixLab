# Needle

Needle is a modern, cross-platform grep-like utility from UnixLab.

Version 0.1 searches one UTF-8 text file with a regular expression:

```text
needle <pattern> <file>
```

Matching lines are printed to stdout. Matching text is highlighted when stdout
is an interactive terminal; redirected output is plain.

## Example

```sh
needle "error|warning" application.log
```

## Exit status

- `0`: one or more lines matched
- `1`: no lines matched
- `2`: usage, pattern, file, decoding, read, or output error

## Limitations

Needle 0.1 accepts one file. It does not yet support recursive search, multiple
files, stdin, context lines, line numbers, or grep's complete option and regex
compatibility surface.

See [the 0.1 design document](../../docs/design/needle-0.1.md) for architecture,
decisions, and the testing strategy.

