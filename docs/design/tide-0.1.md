# Tide 0.1 design

- Status: Implemented
- Application: `tide`
- Inspiration: curl

## Scope

Tide 0.1 accepts exactly:

```text
tide <url>
```

It sends one HTTP GET request, follows at most ten redirects, and streams the
response body to stdout. HTTP and HTTPS are supported on Linux, macOS, and
Windows.

The following are deliberately out of scope:

- Request methods other than GET
- Request bodies and file uploads
- Custom request headers
- Printing response headers
- Authentication, cookies, and proxy configuration
- Saving directly to a named file
- Parallel or resumed transfers
- User-selectable timeout and redirect policy
- curl's complete option and protocol surface

Each requires an explicit interface and compatibility proposal.

## Command contract

The URL must be representable as Unicode and use HTTP or HTTPS. The response
body is copied as raw bytes without newline or text conversion. This preserves
text, JSON, images, archives, and other binary payloads exactly.

Tide follows up to ten redirects and applies a 30-second total request timeout.
HTTP 4xx and 5xx responses are successful transfers: their bodies are written
and the process exits zero, matching curl's behavior when `--fail` is absent.

Diagnostics use `tide: <message>` on stderr. Response bytes are written only to
stdout.

## Exit statuses

Tide uses the corresponding curl exit status where the 0.1 failure maps
unambiguously:

| Status | Meaning |
| --- | --- |
| 0 | Response body transferred successfully |
| 2 | Invalid command usage |
| 3 | Malformed or unsupported URL |
| 6 | Host name could not be resolved |
| 7 | Connection or other request failure |
| 18 | Response body ended or failed unexpectedly |
| 23 | stdout write failed |
| 28 | Request timed out |
| 35 | TLS connection failed |
| 47 | Redirect limit exceeded |

This is not yet a promise of full curl exit-code compatibility.

## Architecture

`main.rs` owns OS arguments, stdout/stderr, and process exit status. The
library is divided into:

- `cli`: validates the one-positional-argument interface.
- `client`: configures HTTP policy and streams response bytes.
- `error`: preserves request, response-read, and output-write failures and
  maps them to documented exit statuses.
- `lib.rs`: exposes stable configuration, transfer summary, and orchestration.

The response-copy loop classifies read and write failures separately; using
`std::io::copy` would lose the information needed for exit statuses 18 and 23.

## Design decisions

### Use blocking I/O

Version 0.1 performs one request. An async runtime would increase compile time,
dependency count, and control-flow complexity without improving concurrency.
The blocking boundary can later be placed behind parallel orchestration if a
documented feature requires it.

### Use ureq with rustls only

HTTP parsing, TLS, certificate validation, redirects, and hostile network input
should use maintained implementations. `ureq` is a small blocking client,
supports Rust 1.85, and forbids unsafe code in its own crate. Only its `rustls`
feature is enabled; gzip, JSON, cookies, charset conversion, multipart, native
TLS, and proxy features are omitted.

Rustls with Web PKI roots gives consistent certificate verification across all
three supported operating systems. Platform-native certificate stores are a
possible future policy but need explicit portability and enterprise-environment
requirements.

### Stream raw bytes

The response is never accumulated in memory. Memory usage remains bounded by a
16 KiB transfer buffer, and the body is not interpreted as UTF-8. stdout is
locked once at the process boundary.

### Treat HTTP error statuses as responses

An HTTP status is not a transport failure. Returning 404 or 500 content by
default matches curl without `--fail` and keeps status-policy options open for
a later version.

### Set bounded network policy

Ten redirects prevents redirect loops. A 30-second global timeout prevents a
default invocation from hanging indefinitely. Both values are public command
behavior and require a documented change.

### Avoid shared libraries for unproven concepts

Tide keeps its error and CLI policy local. Needle and Tide do not yet share a
concrete API that justifies moving code into `unixlab-cli` or
`unixlab-errors`.

## Testing strategy

Unit tests cover CLI validation, error-to-exit mapping, byte-exact transfer,
large streaming bodies, and separate read/write failure classification.

Integration tests start a local TCP HTTP server and execute the real binary.
They cover successful text and binary responses, 404 bodies, redirects,
malformed URLs, unreachable endpoints, usage, stdout/stderr separation, and
exit statuses. Tests require no external network.

The benchmark runs complete requests against an in-process local HTTP server,
including HTTP parsing and streaming, without depending on internet latency:

```sh
cargo bench -p tide
```

## Future changes

Options for methods, headers, request bodies, output files, progress, proxies,
certificate stores, compression, and parallelism must extend the process
boundary without coupling protocol transport to presentation. HTTP/2 or async
I/O should be adopted only after benchmarks demonstrate a relevant workload.

