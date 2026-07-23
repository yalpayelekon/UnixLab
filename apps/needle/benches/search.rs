//! Stable benchmark for Needle's complete in-memory search path.

use std::hint::black_box;
use std::io::{Cursor, sink};
use std::time::Instant;

use needle::matcher::Matcher;
use needle::search::search_reader;

const ITERATIONS: usize = 200;

fn main() {
    let input = benchmark_input();
    let matcher =
        Matcher::new(r"error|warning|needle-\d+").expect("benchmark pattern should compile");
    let started = Instant::now();

    for _ in 0..ITERATIONS {
        let reader = Cursor::new(black_box(input.as_bytes()));
        let mut output = sink();
        let summary = search_reader(&matcher, reader, &mut output, false)
            .expect("in-memory search should succeed");
        black_box(summary);
    }

    println!(
        "searched {} bytes for {ITERATIONS} iterations in {:?}",
        input.len(),
        started.elapsed()
    );
}

fn benchmark_input() -> String {
    let mut input = String::with_capacity(256 * 1024);
    for index in 0..4_000 {
        use std::fmt::Write as _;
        writeln!(
            input,
            "record {index}: status={} message=needle-{index}",
            if index % 20 == 0 { "warning" } else { "ok" }
        )
        .expect("writing to a String should succeed");
    }
    input
}
