$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

cargo fmt --all --check
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

cargo clippy --workspace --all-targets --all-features -- -D warnings
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

cargo test --workspace --all-features
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$env:RUSTDOCFLAGS = '-D warnings'
cargo doc --workspace --all-features --no-deps
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

