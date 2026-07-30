# Resolver 0.1 design

- Status: Implemented
- Application: `resolver`
- Inspiration: dig / host
- Compatibility target: host-inspired A/AAAA address listing via the
  operating-system resolver

## Scope

Resolver 0.1 accepts exactly:

```text
resolver <name>
```

It resolves one hostname or IP literal with the OS resolver and prints each
unique IP address on its own line. IPv4 and IPv6 addresses are both included
when the system returns them.

The following are deliberately out of scope:

- Record-type selection (`A`, `AAAA`, `MX`, `TXT`, `NS`, and others)
- dig-style question/answer/authority/additional sections
- Custom nameserver (`@server`) selection
- Reverse (PTR) lookups
- DNSSEC validation
- User-selectable timeouts and resolver flags
- JSON or other structured output formats
- `/etc/hosts` behavior beyond what the operating system already applies

Each requires a separate proposal.

## Command contract

The name must be representable as Unicode. Resolution uses
`std::net::ToSocketAddrs` on `"<name>:0"`. Addresses are printed with their
standard Display form, one per line, in first-seen order from the OS, with
duplicates removed.

Diagnostics use `resolver: <message>` on stderr. Addresses are written only to
stdout.

Memory use is negligible: only the small set of returned socket addresses is
held.

## Exit statuses

| Status | Meaning |
| --- | --- |
| 0 | One or more addresses printed |
| 1 | Name could not be resolved or produced no addresses |
| 2 | Usage, non-Unicode name, or stdout write failure |

## Architecture

`main.rs` owns OS arguments, stdout/stderr, and process exit status. The
library is divided into:

- `cli`: validates the one-positional-argument interface.
- `lookup`: performs system resolution, deduplicates addresses, and writes
  lines.
- `error`: preserves structured causes and maps failures to documented exit
  statuses.
- `lib.rs`: exposes configuration, summary, and orchestration.

## Design decisions

### Use the operating-system resolver

Version 0.1 only needs address records. `ToSocketAddrs` is portable, requires
no external crates, and matches how most applications resolve hosts. A
dedicated DNS library (for example Hickory) is deferred until typed queries or
custom nameservers are in scope.

### Keep blocking lookups

One synchronous lookup is the entire workload. An async runtime would add
dependencies and complexity without improving the 0.1 interface.

### Accept platform resolver differences

Order, dual-stack policy, search domains, and hosts-file behavior vary by OS.
Resolver documents that results follow the platform resolver rather than
emulating a single dig implementation.

### Keep domain errors local

Resolver defines usage, lookup, and write failures locally. Existing apps do
not yet share a concrete API that justifies moving types into `unixlab-cli` or
`unixlab-errors`.

### Use stable, dependency-free benchmarks

The benchmark target uses `std::hint::black_box` and `Instant` under Cargo's
stable custom harness, measuring repeated `localhost` lookups without adding a
benchmarking framework.

## Testing strategy

Unit tests cover argument validation, address deduplication/formatting with
injected socket addresses, empty results, and exit-status mapping.

Integration tests execute the real binary against `localhost`, an IP literal,
an RFC 6761 `.invalid` name, and usage errors. They verify stdout/stderr
separation and exit statuses without requiring a custom DNS server.

CI runs the suite on Linux, macOS, and Windows. The benchmark is compiled by
Clippy's all-targets pass and can be run with:

```sh
cargo bench -p resolver
```

## Future changes

Record types, custom nameservers, reverse lookups, and dig-compatible
presentation must remain separate proposals. Introducing a DNS library should
wait until those features need protocol control beyond `getaddrinfo`.
