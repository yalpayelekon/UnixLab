//! Stable end-to-end benchmark using an in-process HTTP server.

use std::hint::black_box;
use std::io::{Read, Write, sink};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Instant;

use tide::Config;

const ITERATIONS: usize = 100;
const BODY_SIZE: usize = 256 * 1024;

fn main() {
    let body = vec![b'x'; BODY_SIZE];
    let listener = TcpListener::bind("127.0.0.1:0").expect("benchmark server should bind");
    let address = listener
        .local_addr()
        .expect("benchmark server should have an address");
    let server = thread::spawn(move || serve(&listener, &body));
    let config = Config::new(format!("http://{address}/benchmark"));
    let started = Instant::now();

    for _ in 0..ITERATIONS {
        let summary =
            tide::run(&config, &mut sink()).expect("local benchmark request should succeed");
        black_box(summary);
    }

    server.join().expect("benchmark server should finish");
    println!(
        "transferred {BODY_SIZE} bytes for {ITERATIONS} requests in {:?}",
        started.elapsed()
    );
}

fn serve(listener: &TcpListener, body: &[u8]) {
    for stream in listener.incoming().take(ITERATIONS) {
        let mut stream = stream.expect("benchmark client should connect");
        read_request(&mut stream);
        write!(
            stream,
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        )
        .expect("benchmark response headers should write");
        stream
            .write_all(body)
            .expect("benchmark response body should write");
    }
}

fn read_request(stream: &mut TcpStream) {
    let mut request = Vec::new();
    let mut buffer = [0_u8; 512];

    while !request.ends_with(b"\r\n\r\n") {
        let count = stream
            .read(&mut buffer)
            .expect("benchmark request should read");
        assert!(count > 0, "request ended before headers completed");
        request.extend_from_slice(&buffer[..count]);
    }
}
