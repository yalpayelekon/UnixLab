# unixlab-fs

Portable filesystem boundaries and carefully documented platform adaptations
shared by UnixLab tools.

Likely responsibilities include file metadata normalization, testable
filesystem seams, and explicit handling of paths that are not valid Unicode.
This crate does not wrap `std::fs` wholesale and does not impose application
traversal policy.

No filesystem abstraction is introduced during the foundation phase.

