//! Black-box tests for Tide's command-line and HTTP behavior.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::process::{Command, Output};
use std::thread::{self, JoinHandle};

struct LocalServer {
    address: SocketAddr,
    handle: JoinHandle<()>,
}

impl LocalServer {
    fn respond(status: &str, headers: &[(&str, &str)], body: &[u8]) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("local test server should bind");
        let address = listener
            .local_addr()
            .expect("local test server should have an address");
        let status = status.to_owned();
        let headers = headers
            .iter()
            .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
            .collect::<Vec<_>>();
        let body = body.to_vec();
        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("client should connect");
            read_request(&mut stream);

            write!(
                stream,
                "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n",
                body.len()
            )
            .expect("response headers should write");
            for (name, value) in headers {
                write!(stream, "{name}: {value}\r\n").expect("response header should write");
            }
            stream
                .write_all(b"\r\n")
                .expect("header terminator should write");
            stream.write_all(&body).expect("response body should write");
        });

        Self { address, handle }
    }

    fn url(&self, path: &str) -> String {
        format!("http://{}{path}", self.address)
    }

    fn finish(self) {
        self.handle.join().expect("test server should finish");
    }
}

fn read_request(stream: &mut TcpStream) {
    let mut request = Vec::new();
    let mut buffer = [0_u8; 512];

    while !request.ends_with(b"\r\n\r\n") {
        let count = stream.read(&mut buffer).expect("request should read");
        assert!(count > 0, "request ended before headers completed");
        request.extend_from_slice(&buffer[..count]);
    }
}

fn run(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tide"))
        .args(arguments)
        .output()
        .expect("tide process should run")
}

#[test]
fn streams_successful_response_body() {
    let server = LocalServer::respond("200 OK", &[("Content-Type", "text/plain")], b"hello tide\n");

    let output = run(&[&server.url("/hello")]);
    server.finish();

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, b"hello tide\n");
    assert!(output.stderr.is_empty());
}

#[test]
fn preserves_binary_response_bytes() {
    let expected = [0_u8, 1, 2, 0xff, b'\r', b'\n'];
    let server = LocalServer::respond(
        "200 OK",
        &[("Content-Type", "application/octet-stream")],
        &expected,
    );

    let output = run(&[&server.url("/binary")]);
    server.finish();

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, expected);
}

#[test]
fn writes_http_error_body_and_returns_zero() {
    let server = LocalServer::respond("404 Not Found", &[], b"missing\n");

    let output = run(&[&server.url("/missing")]);
    server.finish();

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, b"missing\n");
    assert!(output.stderr.is_empty());
}

#[test]
fn follows_redirects() {
    let destination = LocalServer::respond("200 OK", &[], b"redirected\n");
    let destination_url = destination.url("/final");
    let redirect =
        LocalServer::respond("302 Found", &[("Location", destination_url.as_str())], b"");

    let output = run(&[&redirect.url("/start")]);
    redirect.finish();
    destination.finish();

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, b"redirected\n");
    assert!(output.stderr.is_empty());
}

#[test]
fn rejects_unsupported_url_scheme() {
    let output = run(&["ftp://example.com/file"]);

    assert_eq!(output.status.code(), Some(3));
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr should be UTF-8"),
        "tide: URL must use http:// or https://\n"
    );
}

#[test]
fn reports_connection_failure() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("temporary listener should bind");
    let address = listener
        .local_addr()
        .expect("temporary listener should have an address");
    drop(listener);
    let url = format!("http://{address}/");

    let output = run(&[&url]);

    assert_eq!(output.status.code(), Some(7));
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8(output.stderr)
            .expect("stderr should be UTF-8")
            .starts_with("tide: request failed:")
    );
}

#[test]
fn rejects_invalid_usage() {
    let output = run(&[]);

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr should be UTF-8"),
        "tide: usage: tide <url>\n"
    );
}
