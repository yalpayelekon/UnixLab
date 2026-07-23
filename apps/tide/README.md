# Tide

Tide is a small, cross-platform HTTP client from UnixLab, inspired by curl.

Version 0.1 performs one GET request:

```text
tide <url>
```

The response body is streamed to stdout as raw bytes. HTTP and HTTPS are
supported, redirects are followed up to ten times, and requests have a
30-second total timeout.

## Example

```sh
tide https://example.com/
```

HTTP 4xx and 5xx bodies are written normally and do not by themselves cause a
non-zero exit status.

## Limitations

Tide 0.1 does not support other methods, uploads, custom headers,
authentication, proxy configuration, output files, or curl's complete option
and protocol surface.

See [the 0.1 design document](../../docs/design/tide-0.1.md) for its contract,
architecture, exit statuses, and testing strategy.

