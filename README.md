# UnixLab

UnixLab is a collection of production-quality reimplementations of classic
Unix tools written in Rust. It is a systems-programming laboratory, not a
collection of coding challenges: each tool is expected to be documented,
tested, portable, maintainable, and useful on its own.

The project is in its early development phase. Needle, Tide, and Reed are the
first implemented applications.

## Planned tools

| Project | Inspiration | Status |
| --- | --- | --- |
| [`needle`](apps/needle) | `grep` | 0.1 |
| [`tide`](apps/tide) | `curl` | 0.1 |
| [`reed`](apps/reed) | `sed` | 0.1 |
| `hush` | Unix shells | Planned |
| `quartz` | JSON parsers | Planned |
| `branch` | Git clients | Planned |
| `resolver` | DNS tools | Planned |

Names and scope remain subject to an architecture proposal before
implementation.

## Repository layout

- `apps/` contains independently releasable command-line applications.
- `libs/` contains narrowly scoped libraries shared by applications.
- `docs/` contains architecture decisions and project-wide documentation.
- `examples/` contains runnable integration examples as APIs emerge.
- `scripts/` contains portable developer automation.
- `.github/` contains CI and community configuration.

Read [the architecture guide](docs/architecture.md) and
[engineering manifesto](ENGINEERING.md), followed by the detailed
[engineering guidelines](docs/engineering-guidelines.md), before proposing a
new tool.

## Development

UnixLab uses the stable Rust toolchain, Rust 2024, Clippy, and rustfmt.

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo doc --workspace --all-features --no-deps
```

On Unix-like systems, `./scripts/check.sh` runs the complete local check. On
Windows, use `./scripts/check.ps1`.

## Contributing

Contributions are welcome. Start with [CONTRIBUTING.md](CONTRIBUTING.md), the
[code of conduct](CODE_OF_CONDUCT.md), and an issue or design proposal for
substantial changes. Commits follow the Conventional Commits specification.

## License

UnixLab is licensed under the [MIT License](LICENSE).
