//! Stable benchmark for Quartz's in-memory path query.

use std::hint::black_box;
use std::io::sink;
use std::time::Instant;

use quartz::filter::Filter;
use quartz::query::{apply, write_value};
use serde_json::json;

const ITERATIONS: usize = 5_000;

fn main() {
    let value = benchmark_value();
    let filter = Filter::parse(".items[3].name").expect("benchmark filter should parse");
    let started = Instant::now();

    for _ in 0..ITERATIONS {
        let selected = apply(&filter, black_box(&value));
        let mut output = sink();
        write_value(black_box(&selected), true, &mut output)
            .expect("in-memory encode should succeed");
        black_box(selected);
    }

    println!(
        "queried benchmark document for {ITERATIONS} iterations in {:?}",
        started.elapsed()
    );
}

fn benchmark_value() -> serde_json::Value {
    let items: Vec<_> = (0..64)
        .map(|index| {
            json!({
                "id": index,
                "name": format!("item-{index}"),
                "active": index % 2 == 0,
            })
        })
        .collect();
    json!({ "items": items })
}
