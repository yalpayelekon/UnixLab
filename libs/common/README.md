# unixlab-common

Domain-neutral primitives that are genuinely shared across multiple UnixLab
applications.

This crate is the bottom of the dependency graph. It must remain small,
platform-independent, and free of application policy. Code belongs here only
after at least two consumers need the same stable concept; it is not a
miscellaneous utilities crate.

No public primitives are defined during the foundation phase.

