# Engineering guidelines

## Core principles

1. Correctness before compatibility breadth; compatibility before convenience.
2. Make resource behavior visible. Stream data when inputs may be unbounded.
3. Prefer standard-library facilities and small explicit types.
4. Add abstractions after repeated evidence, not in anticipation.
5. Keep platform differences explicit and tested.
6. Treat command output, diagnostics, and logs as separate interfaces.

## Rust

- Use stable Rust and edition 2024. The minimum supported Rust version (MSRV)
  is 1.85 until changed by an architecture decision.
- Format all Rust with the repository rustfmt configuration.
- Workspace code must pass Clippy with warnings denied in CI.
- Unsafe Rust is forbidden by default. Enabling it requires a narrowly scoped
  lint exception, documented invariants, targeted tests, and maintainer review.
- Prefer borrowing and iterators when they improve clarity; do not optimize
  away readable ownership without measurements.
- Use typed domain errors. Preserve sources and attach context at boundaries.
- Avoid panics for expected input, filesystem, network, terminal, and process
  failures. Assertions remain appropriate for violated internal invariants.
- Public items require rustdoc. Include examples for behavior that is not
  obvious from the type signature.

## Dependencies

Every external dependency must solve a concrete need better than a small,
maintainable local implementation. Pull requests adding one must discuss:

- Why the standard library is insufficient.
- Maintenance health, license, and security posture.
- Transitive dependency and build-time cost.
- Platform support and feature flags.
- Whether it is part of the public API.

Disable default features when they add unused capability. Do not centralize a
dependency in the workspace until multiple crates need coordinated versions.

## Testing

Use the smallest test that proves behavior:

- Unit tests for algorithms and edge cases.
- Integration tests for crate boundaries.
- Black-box tests for CLI output, exit status, and stream behavior.
- Platform-specific tests for intentional OS differences.
- Property or fuzz testing for parsers and hostile input when those components
  are introduced.
- Benchmarks only for important performance claims or regressions.

Tests must be deterministic, independent of network access by default, and
safe to run concurrently. Temporary resources must be isolated and cleaned up.
Bug fixes include a regression test.

## Command-line behavior

- stdout is for requested data; stderr is for diagnostics.
- Redirected output must not contain unsolicited styling.
- Exit statuses form a documented compatibility contract.
- Accept non-Unicode OS strings and paths where the platform permits them.
- Avoid reading an entire input when a streaming implementation is practical.
- Broken pipes should terminate quietly when downstream consumers close.

## Documentation

Each application has a README covering purpose, compatibility scope, examples,
platform notes, and limitations. Public APIs have rustdoc. Significant design
choices are captured in design documents or ADRs, including rejected
alternatives.

Documentation must describe current behavior, not intended behavior disguised
as complete. Examples are tested when practical.

## Commits and reviews

Commits follow Conventional Commits. Pull requests are focused, explain why,
and include tests and documentation. Review checks correctness, portability,
failure behavior, dependency cost, security, and maintainability.

## Completion criteria

A change is complete when:

- Behavior and compatibility decisions are documented.
- Relevant tests pass on Linux, macOS, and Windows.
- Formatting, Clippy, tests, and documentation checks pass.
- Errors and resource limits have been considered.
- No unrelated abstraction or dependency was added.

