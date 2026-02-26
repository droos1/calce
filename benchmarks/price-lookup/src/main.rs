//! Price lookup benchmark.
//!
//! Compares data structures for the (instrument, date) -> price lookup
//! that dominates the market value calculation hot path.
//!
//! Run with: cargo run --release

use std::collections::{BTreeMap, HashMap};
use std::hint::black_box;
use std::time::Instant;

use rand::prelude::*;
use rand::rngs::StdRng;
use rust_decimal::Decimal;

const ITERS: usize = 20;
const INSTRUMENTS: usize = 1_000;
const DAYS: usize = 1_260;
const TOTAL_ENTRIES: usize = INSTRUMENTS * DAYS;

fn main() {
    if cfg!(debug_assertions) {
        println!("WARNING: debug mode. Run `cargo run --release` for accurate timings.\n");
    }

    println!("Price Lookup Benchmark");
    println!("======================");
    println!("Entries: {INSTRUMENTS} instruments x {DAYS} days = {TOTAL_ENTRIES}");
    println!("Iterations: {ITERS}\n");

    let mut rng = StdRng::seed_from_u64(42);

    // Instrument names (like calce's InstrumentId)
    let names: Vec<String> = (0..INSTRUMENTS)
        .map(|i| format!("INST_{i:04}"))
        .collect();

    // Generate prices
    let prices: Vec<f64> = (0..TOTAL_ENTRIES)
        .map(|_| rng.gen_range(1.0..1000.0))
        .collect();

    // Build data structures
    // 1) HashMap<(String, i32), Decimal> — similar to calce's current approach
    let hm_string: HashMap<(String, i32), Decimal> = build_hashmap_string(&names, &prices);

    // 2) HashMap<(u32, i32), Decimal> — interned instrument ID
    let hm_int: HashMap<(u32, i32), Decimal> = build_hashmap_int(&prices);

    // 3) HashMap<u64, Decimal> — packed key: inst << 32 | day
    let hm_packed: HashMap<u64, Decimal> = build_hashmap_packed(&prices);

    // 4) Vec<Vec<Decimal>> — 2D array [instrument][day], O(1)
    let vec_2d: Vec<Vec<Decimal>> = build_vec_2d(&prices);

    // 5) BTreeMap<(String, i32), Decimal>
    let btree: BTreeMap<(String, i32), Decimal> = build_btree_string(&names, &prices);

    // Generate random lookup keys
    let lookup_counts = [1_000, 10_000, 100_000, 1_000_000];

    println!(
        "{:<10} {:>16} {:>16} {:>16} {:>16} {:>16}",
        "Lookups",
        "HashMap<String>",
        "HashMap<u32>",
        "HashMap<packed>",
        "Vec<Vec> (2D)",
        "BTreeMap<String>"
    );
    println!("{}", "-".repeat(96));

    for &n in &lookup_counts {
        let lookups: Vec<(usize, usize)> = (0..n)
            .map(|_| {
                (
                    rng.gen_range(0..INSTRUMENTS),
                    rng.gen_range(0..DAYS),
                )
            })
            .collect();

        // HashMap<(String, i32), Decimal> — clone String for key
        let t_string = bench_median(|| {
            let mut sum = Decimal::ZERO;
            for &(inst, day) in &lookups {
                if let Some(&p) = hm_string.get(&(names[inst].clone(), day as i32)) {
                    sum += p;
                }
            }
            black_box(sum);
        });

        // HashMap<(u32, i32), Decimal> — integer key
        let t_int = bench_median(|| {
            let mut sum = Decimal::ZERO;
            for &(inst, day) in &lookups {
                if let Some(&p) = hm_int.get(&(inst as u32, day as i32)) {
                    sum += p;
                }
            }
            black_box(sum);
        });

        // HashMap<u64, Decimal> — packed key
        let t_packed = bench_median(|| {
            let mut sum = Decimal::ZERO;
            for &(inst, day) in &lookups {
                let key = ((inst as u64) << 32) | (day as u64);
                if let Some(&p) = hm_packed.get(&key) {
                    sum += p;
                }
            }
            black_box(sum);
        });

        // Vec<Vec<Decimal>> — direct index
        let t_vec = bench_median(|| {
            let mut sum = Decimal::ZERO;
            for &(inst, day) in &lookups {
                sum += vec_2d[inst][day];
            }
            black_box(sum);
        });

        // BTreeMap<(String, i32), Decimal>
        let t_btree = bench_median(|| {
            let mut sum = Decimal::ZERO;
            for &(inst, day) in &lookups {
                if let Some(&p) = btree.get(&(names[inst].clone(), day as i32)) {
                    sum += p;
                }
            }
            black_box(sum);
        });

        println!(
            "{:<10} {:>13.2} ms {:>13.2} ms {:>13.2} ms {:>13.2} ms {:>13.2} ms",
            format_count(n),
            t_string,
            t_int,
            t_packed,
            t_vec,
            t_btree,
        );
    }

    // Memory comparison
    println!("\nMemory estimate (data only, excluding HashMap overhead):");
    let entry_sizes = [
        ("HashMap<(String,i32), Dec>", (std::mem::size_of::<String>() + 4 + 16) * TOTAL_ENTRIES),
        ("HashMap<(u32,i32), Dec>", (4 + 4 + 16) * TOTAL_ENTRIES),
        ("HashMap<u64, Dec>", (8 + 16) * TOTAL_ENTRIES),
        ("Vec<Vec<Dec>> (2D)", 16 * TOTAL_ENTRIES),
        ("BTreeMap<(String,i32), Dec>", (std::mem::size_of::<String>() + 4 + 16) * TOTAL_ENTRIES),
    ];
    for (name, bytes) in &entry_sizes {
        println!("  {:<30} {:>6.1} MB", name, *bytes as f64 / (1024.0 * 1024.0));
    }
}

// ---------------------------------------------------------------------------
// Data structure builders
// ---------------------------------------------------------------------------

fn build_hashmap_string(
    names: &[String],
    prices: &[f64],
) -> HashMap<(String, i32), Decimal> {
    let mut m = HashMap::with_capacity(TOTAL_ENTRIES);
    for inst in 0..INSTRUMENTS {
        for day in 0..DAYS {
            let p = Decimal::from_f64_retain(prices[inst * DAYS + day]).unwrap_or_default();
            m.insert((names[inst].clone(), day as i32), p);
        }
    }
    m
}

fn build_hashmap_int(prices: &[f64]) -> HashMap<(u32, i32), Decimal> {
    let mut m = HashMap::with_capacity(TOTAL_ENTRIES);
    for inst in 0..INSTRUMENTS {
        for day in 0..DAYS {
            let p = Decimal::from_f64_retain(prices[inst * DAYS + day]).unwrap_or_default();
            m.insert((inst as u32, day as i32), p);
        }
    }
    m
}

fn build_hashmap_packed(prices: &[f64]) -> HashMap<u64, Decimal> {
    let mut m = HashMap::with_capacity(TOTAL_ENTRIES);
    for inst in 0..INSTRUMENTS {
        for day in 0..DAYS {
            let p = Decimal::from_f64_retain(prices[inst * DAYS + day]).unwrap_or_default();
            let key = ((inst as u64) << 32) | (day as u64);
            m.insert(key, p);
        }
    }
    m
}

fn build_vec_2d(prices: &[f64]) -> Vec<Vec<Decimal>> {
    (0..INSTRUMENTS)
        .map(|inst| {
            (0..DAYS)
                .map(|day| {
                    Decimal::from_f64_retain(prices[inst * DAYS + day]).unwrap_or_default()
                })
                .collect()
        })
        .collect()
}

fn build_btree_string(
    names: &[String],
    prices: &[f64],
) -> BTreeMap<(String, i32), Decimal> {
    let mut m = BTreeMap::new();
    for inst in 0..INSTRUMENTS {
        for day in 0..DAYS {
            let p = Decimal::from_f64_retain(prices[inst * DAYS + day]).unwrap_or_default();
            m.insert((names[inst].clone(), day as i32), p);
        }
    }
    m
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

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
