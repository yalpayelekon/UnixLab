# Applications

Each child directory will be an independently usable, testable, and
releasable UnixLab command-line application.

An application should contain a thin binary entry point and keep its domain
logic in a library target within the same package. Shared code moves to
`libs/` only after multiple applications demonstrate the same requirement.

Adding an application requires an accepted design document covering its scope,
compatibility target, portability constraints, testing strategy, and initial
dependency budget.

