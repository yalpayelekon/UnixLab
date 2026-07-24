//! Stable benchmark for Reed's complete in-memory substitute path.

use std::hint::black_box;
use std::io::{Cursor, sink};
use std::time::Instant;

use reed::engine::transform_reader;
use reed::script::Script;

const ITERATIONS: usize = 200;

fn main() {
    let input = benchmark_input();
    let script = Script::compile(&[String::from(r"s/status=\w+/status=ok/g")])
        .expect("benchmark script should compile");
    let started = Instant::now();

    for _ in 0..ITERATIONS {
        let reader = Cursor::new(black_box(input.as_bytes()));
        let mut output = sink();
        let summary = transform_reader(&script, reader, &mut output, false)
            .expect("in-memory transform should succeed");
        black_box(summary);
    }

    println!(
        "transformed {} bytes for {ITERATIONS} iterations in {:?}",
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
            "record {index}: status={} message=reed-{index}",
            if index % 20 == 0 { "warning" } else { "ok" }
        )
        .expect("writing to a String should succeed");
    }
    input
}
