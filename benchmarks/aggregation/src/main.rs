//! Trade aggregation benchmark.
//!
//! Measures the cost of aggregating N trades into net positions using
//! a HashMap, comparing key types: String (clone), &str (borrow), u32 (interned).
//!
//! Run with: cargo run --release

use std::collections::HashMap;
use std::hint::black_box;
use std::time::Instant;

use rand::prelude::*;
use rand::rngs::StdRng;
use rust_decimal::Decimal;

const ITERS: usize = 20;
const NUM_INSTRUMENTS: usize = 500;

struct Trade {
    instrument_str: String,
    instrument_idx: u32,
    quantity: Decimal,
}

fn main() {
    if cfg!(debug_assertions) {
        println!("WARNING: debug mode. Run `cargo run --release` for accurate timings.\n");
    }

    let trade_counts = [1_000, 10_000, 100_000, 1_000_000];

    println!("Trade Aggregation Benchmark");
    println!("===========================");
    println!("Unique instruments: {NUM_INSTRUMENTS} | Iterations: {ITERS}\n");

    println!(
        "{:<10} {:>14} {:>14} {:>14} {:>14}",
        "Trades", "String clone", "&str borrow", "u32 key", "Clone overhead"
    );
    println!("{}", "-".repeat(70));

    for &n in &trade_counts {
        let trades = generate_trades(n);

        // 1) String clone on each entry() — what calce does
        let t_clone = bench_median(|| {
            let mut net: HashMap<String, Decimal> = HashMap::new();
            for t in &trades {
                net.entry(t.instrument_str.clone())
                    .and_modify(|q| *q += t.quantity)
                    .or_insert(t.quantity);
            }
            black_box(net.len());
        });

        // 2) Borrowed &str key — zero-copy, borrows from trade slice
        let t_borrow = bench_median(|| {
            let mut net: HashMap<&str, Decimal> = HashMap::new();
            for t in &trades {
                net.entry(&t.instrument_str)
                    .and_modify(|q| *q += t.quantity)
                    .or_insert(t.quantity);
            }
            black_box(net.len());
        });

        // 3) u32 integer key — cheapest hash, no allocation
        let t_int = bench_median(|| {
            let mut net: HashMap<u32, Decimal> = HashMap::new();
            for t in &trades {
                net.entry(t.instrument_idx)
                    .and_modify(|q| *q += t.quantity)
                    .or_insert(t.quantity);
            }
            black_box(net.len());
        });

        let overhead = (t_clone / t_borrow - 1.0) * 100.0;

        println!(
            "{:<10} {:>11.2} ms {:>11.2} ms {:>11.2} ms {:>+12.0}%",
            format_count(n),
            t_clone,
            t_borrow,
            t_int,
            overhead,
        );
    }

    println!("\n\"Clone overhead\" = (String clone - &str borrow) / &str borrow");
}

fn generate_trades(n: usize) -> Vec<Trade> {
    let mut rng = StdRng::seed_from_u64(42);
    (0..n)
        .map(|_| {
            let idx = rng.gen_range(0..NUM_INSTRUMENTS as u32);
            let qty_raw = rng.gen_range(-500_i64..=500);
            Trade {
                instrument_str: format!("INST_{idx:04}"),
                instrument_idx: idx,
                quantity: Decimal::new(qty_raw, 0),
            }
        })
        .collect()
}

fn format_count(n: usize) -> String {
    if n >= 1_000_000 {
        format!("{}M", n / 1_000_000)
    } else if n >= 1_000 {
        format!("{}K", n / 1_000)
    } else {
        n.to_string()
    }
}

fn bench_median(mut f: impl FnMut()) -> f64 {
    for _ in 0..3 {
        f();
    }
    let mut times = Vec::with_capacity(ITERS);
    for _ in 0..ITERS {
        let start = Instant::now();
        f();
        times.push(start.elapsed());
    }
    times.sort();
    times[ITERS / 2].as_secs_f64() * 1000.0
}
