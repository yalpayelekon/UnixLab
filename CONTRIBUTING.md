# Contributing to UnixLab

Thank you for helping build UnixLab.

## Before writing code

1. Search existing issues and discussions.
2. Open an issue for user-visible behavior or a design proposal for substantial
   architecture.
3. Agree on scope before implementing a new application or shared abstraction.
4. Update user and architecture documentation before or alongside code.

## Development workflow

1. Install the stable Rust toolchain. The repository toolchain file installs
   rustfmt and Clippy.
2. Create a focused branch from the default branch.
3. Add tests for behavior and regressions.
4. Run `scripts/check.sh` or `scripts/check.ps1`.
5. Open a pull request using the repository template.

## Commits

Use [Conventional Commits](https://www.conventionalcommits.org/):

```text
type(optional-scope): concise imperative summary
```

Common types are `feat`, `fix`, `docs`, `refactor`, `test`, `build`, `ci`,
`chore`, and `perf`. Use the application or library name as the scope when it
adds clarity, for example `feat(terminal): detect redirected output`.

Keep commits reviewable and avoid mixing unrelated changes. Breaking changes
must use `!` or a `BREAKING CHANGE:` footer.

## Pull requests

- Explain the user or engineering problem.
- Describe the chosen design and rejected alternatives when relevant.
- Add or update documentation and tests.
- Keep platform-specific behavior behind a narrow, documented boundary.
- Do not add a dependency without explaining its maintenance and portability
  cost.

See [docs/engineering-guidelines.md](docs/engineering-guidelines.md) for the
full quality standard.

