# unixlab-logging

Consistent initialization and policy for developer-facing diagnostics across
UnixLab applications.

Logging is distinct from normal command output and user-facing errors. This
crate will own filtering, destinations, and machine-readable diagnostic modes
when concrete requirements exist. It must not require applications to depend
on a particular global logger in their core logic.

No logging backend is selected during the foundation phase.

