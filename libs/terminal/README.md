# unixlab-terminal

Cross-platform terminal capability detection and terminal-specific rendering
policy shared by UnixLab applications.

This crate will handle concerns such as TTY detection, color capability, and
terminal dimensions when applications need them. It must preserve redirected
I/O semantics and keep platform-specific code behind narrow boundaries.

No terminal abstraction is introduced during the foundation phase.

