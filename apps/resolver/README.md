# Resolver

Resolver is a modern, cross-platform dig/host-inspired DNS lookup tool from
UnixLab.

Version 0.1 resolves one hostname or IP literal with the operating-system
resolver:

```text
resolver <name>
```

Unique IP addresses are printed to stdout, one per line.

## Example

```sh
resolver example.com
resolver localhost
```

## Exit status

- `0`: one or more addresses printed
- `1`: name could not be resolved or produced no addresses
- `2`: usage, non-Unicode name, or output error

## Limitations

Resolver 0.1 lists A/AAAA-style addresses from the system resolver only. It
does not yet support record-type selection, custom nameservers, reverse
lookups, DNSSEC, or dig-compatible answer formatting.

See [the 0.1 design document](../../docs/design/resolver-0.1.md) for
architecture, decisions, and the testing strategy.
