# unixlab-cli

Shared command-line conventions and small integration helpers for UnixLab
applications.

This crate may eventually standardize help/version behavior, global diagnostic
options, and conversion from application outcomes to exit status. Parsing and
domain-specific commands remain in each application. It must not become a
framework that hides control flow.

No CLI parser or public abstraction is selected during the foundation phase.

