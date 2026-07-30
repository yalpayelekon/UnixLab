# Quartz

Quartz is a modern, cross-platform jq-inspired JSON query tool from UnixLab.

Version 0.1 evaluates a path filter against one JSON document per input:

```text
quartz [-c] [filter] [file...]
```

The filter defaults to `.`. A positional argument that begins with `.` is the
filter; otherwise all positionals are files and standard input is used when
none are given. `-c` selects compact output; the default is pretty-printed.

## Example

```sh
quartz .user.name users.json
quartz -c .items[0] payload.json
```

## Exit status

- `0`: query succeeded (including paths that resolve to `null`)
- `1`: usage, filter, JSON, file, or output error

## Limitations

Quartz 0.1 supports identity, identifier object keys, and non-negative array
indexes only. It does not yet support pipes, iterators, operators, functions,
quoted keys, NDJSON streams, or YAML.

See [the 0.1 design document](../../docs/design/quartz-0.1.md) for architecture,
decisions, and the testing strategy.
