# unixlab-errors

Shared contracts for turning typed domain errors into consistent,
user-oriented diagnostics and process exit behavior.

Libraries should retain precise error types and sources. Applications own
final rendering and exit codes. This crate must not become a universal error
enum or erase recoverable error information.

No error abstraction is introduced until an application demonstrates the
required contract.

