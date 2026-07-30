//! Stable benchmark for Resolver's localhost lookup path.

use std::hint::black_box;
use std::io::sink;
use std::time::Instant;

use resolver::Config;

const ITERATIONS: usize = 200;

fn main() {
    let config = Config::new(String::from("localhost"));
    let started = Instant::now();

    for _ in 0..ITERATIONS {
        let summary = resolver::run(&config, &mut sink()).expect("localhost lookup should succeed");
        black_box(summary);
    }

    println!(
        "resolved localhost for {ITERATIONS} iterations in {:?}",
        started.elapsed()
    );
}
