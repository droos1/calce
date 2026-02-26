//! Portfolio backtest benchmark comparing numeric representations.
//!
//! Generates 5 years of daily prices for 1000 instruments via geometric
//! Brownian motion, builds a random 50-position portfolio, and backtests
//! (daily portfolio value + final return) using f64, f32, i64 fixed-point,
//! and rust_decimal::Decimal.
//!
//! Run with: cargo run --release

use std::hint::black_box;
use std::mem::size_of;
use std::time::{Duration, Instant};

use rand::prelude::*;
use rand::rngs::StdRng;
use rand_distr::Normal;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;

const INSTRUMENTS: usize = 1_000;
const DAYS: usize = 252 * 5; // 1260 trading days
const POSITIONS: usize = 50;
const ITERS: usize = 20;

/// Fixed-point scale: price × 10,000 stored as i64.
/// 4 decimal places covers sub-cent precision ($150.4325 -> 1_504_325).
const FP_SCALE: f64 = 10_000.0;

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() {
    if cfg!(debug_assertions) {
        println!("WARNING: debug mode. Run `cargo run --release` for accurate timings.\n");
    }

    let mut rng = StdRng::seed_from_u64(42);

    // Portfolio: 50 random instruments, 10-500 shares each
    let (indices, quantities) = generate_portfolio(&mut rng);

    // Base prices as f64 [instrument][day] — geometric Brownian motion
    let prices_f64 = generate_prices(&mut rng);

    // Convert to other representations
    let prices_f32: Vec<Vec<f32>> = prices_f64
        .iter()
        .map(|s| s.iter().map(|&p| p as f32).collect())
        .collect();

    let prices_i64: Vec<Vec<i64>> = prices_f64
        .iter()
        .map(|s| s.iter().map(|&p| (p * FP_SCALE).round() as i64).collect())
        .collect();

    let prices_dec: Vec<Vec<Decimal>> = prices_f64
        .iter()
        .map(|s| {
            s.iter()
                .map(|&p| Decimal::from_f64(p).unwrap_or_default())
                .collect()
        })
        .collect();

    // Quantities per representation
    let qty_f64: Vec<f64> = quantities.iter().map(|&q| q as f64).collect();
    let qty_f32: Vec<f32> = quantities.iter().map(|&q| q as f32).collect();
    let qty_dec: Vec<Decimal> = quantities.iter().map(|&q| Decimal::from(q)).collect();

    // --- Run benchmarks ---
    println!("Portfolio Backtest Benchmark");
    println!("===========================");
    println!(
        "Instruments: {INSTRUMENTS} | Days: {DAYS} (~5y) | Positions: {POSITIONS} | Iterations: {ITERS}"
    );
    println!();

    let results = [
        bench("f64", || backtest_f64(&prices_f64, &indices, &qty_f64)),
        bench("f32", || backtest_f32(&prices_f32, &indices, &qty_f32)),
        bench("i64 (x10000)", || {
            backtest_i64(&prices_i64, &indices, &quantities)
        }),
        bench("Decimal", || {
            backtest_decimal(&prices_dec, &indices, &qty_dec)
        }),
    ];

    // --- Results table ---
    let ref_return = results[0].final_return;
    let mems = [
        data_memory::<f64>(),
        data_memory::<f32>(),
        data_memory::<i64>(),
        data_memory::<Decimal>(),
    ];
    let precisions = ["reference", "~7 digits", "exact (4dp)", "28 digits"];

    println!(
        "{:<15} {:>10} {:>10} {:>10} {:>10} {:>10} {:>12} {:>12}",
        "Type", "Median", "Min", "Max", "Memory", "Return", "Drift vs f64", "Precision"
    );
    println!("{}", "-".repeat(99));

    for (i, r) in results.iter().enumerate() {
        let drift = r.final_return - ref_return;
        println!(
            "{:<15} {:>7.2} ms {:>7.2} ms {:>7.2} ms {:>7.1} MB {:>+9.2}% {:>+11.6}% {:>12}",
            r.name,
            r.median_ms(),
            r.min_ms(),
            r.max_ms(),
            mems[i] as f64 / (1024.0 * 1024.0),
            r.final_return * 100.0,
            drift * 100.0,
            precisions[i],
        );
    }

    // Speedup summary
    let dec_median = results[3].median_ms();
    println!("\nSpeedup vs Decimal:");
    for r in &results {
        println!("  {:<15} {:>6.1}x", r.name, dec_median / r.median_ms());
    }
}

// ---------------------------------------------------------------------------
// Data generation
// ---------------------------------------------------------------------------

fn generate_portfolio(rng: &mut StdRng) -> (Vec<usize>, Vec<i64>) {
    let mut indices: Vec<usize> = (0..INSTRUMENTS).collect();
    indices.shuffle(rng);
    indices.truncate(POSITIONS);
    indices.sort_unstable();
    let quantities: Vec<i64> = (0..POSITIONS).map(|_| rng.gen_range(10_i64..=500)).collect();
    (indices, quantities)
}

fn generate_prices(rng: &mut StdRng) -> Vec<Vec<f64>> {
    let normal = Normal::new(0.0, 1.0).expect("valid distribution params");
    let mu = 0.000_2; // ~5% annualized drift
    let sigma = 0.02; // ~32% annualized vol

    (0..INSTRUMENTS)
        .map(|_| {
            let mut price = rng.gen_range(10.0_f64..500.0);
            (0..DAYS)
                .map(|_| {
                    let p = price;
                    let z: f64 = rng.sample(normal);
                    price = (price * (mu + sigma * z).exp()).max(0.01);
                    p
                })
                .collect()
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Backtest implementations
// ---------------------------------------------------------------------------

fn backtest_f64(prices: &[Vec<f64>], indices: &[usize], qty: &[f64]) -> f64 {
    let mut values = Vec::with_capacity(DAYS);
    for day in 0..DAYS {
        let mut v = 0.0_f64;
        for (pos, &inst) in indices.iter().enumerate() {
            v += qty[pos] * prices[inst][day];
        }
        values.push(v);
    }
    let ret = values[DAYS - 1] / values[0] - 1.0;
    black_box(&values);
    ret
}

fn backtest_f32(prices: &[Vec<f32>], indices: &[usize], qty: &[f32]) -> f64 {
    let mut values = Vec::with_capacity(DAYS);
    for day in 0..DAYS {
        let mut v = 0.0_f32;
        for (pos, &inst) in indices.iter().enumerate() {
            v += qty[pos] * prices[inst][day];
        }
        values.push(v);
    }
    let ret = (values[DAYS - 1] / values[0] - 1.0) as f64;
    black_box(&values);
    ret
}

fn backtest_i64(prices: &[Vec<i64>], indices: &[usize], qty: &[i64]) -> f64 {
    let mut values = Vec::with_capacity(DAYS);
    for day in 0..DAYS {
        let mut v = 0_i64;
        for (pos, &inst) in indices.iter().enumerate() {
            // qty (shares) * price (scaled) = value in scaled units
            v += qty[pos] * prices[inst][day];
        }
        values.push(v);
    }
    let ret = values[DAYS - 1] as f64 / values[0] as f64 - 1.0;
    black_box(&values);
    ret
}

fn backtest_decimal(prices: &[Vec<Decimal>], indices: &[usize], qty: &[Decimal]) -> f64 {
    let mut values = Vec::with_capacity(DAYS);
    for day in 0..DAYS {
        let mut v = Decimal::ZERO;
        for (pos, &inst) in indices.iter().enumerate() {
            v += qty[pos] * prices[inst][day];
        }
        values.push(v);
    }
    let ret = (values[DAYS - 1] / values[0])
        .to_f64()
        .unwrap_or(0.0)
        - 1.0;
    black_box(&values);
    ret
}

// ---------------------------------------------------------------------------
// Benchmark harness
// ---------------------------------------------------------------------------

struct BenchResult {
    name: &'static str,
    times: Vec<Duration>,
    final_return: f64,
}

impl BenchResult {
    fn sorted_times(&self) -> Vec<Duration> {
        let mut t = self.times.clone();
        t.sort();
        t
    }
    fn median_ms(&self) -> f64 {
        let s = self.sorted_times();
        s[s.len() / 2].as_secs_f64() * 1000.0
    }
    fn min_ms(&self) -> f64 {
        self.times
            .iter()
            .min()
            .unwrap_or(&Duration::ZERO)
            .as_secs_f64()
            * 1000.0
    }
    fn max_ms(&self) -> f64 {
        self.times
            .iter()
            .max()
            .unwrap_or(&Duration::ZERO)
            .as_secs_f64()
            * 1000.0
    }
}

fn bench(name: &'static str, mut f: impl FnMut() -> f64) -> BenchResult {
    // Warmup
    for _ in 0..3 {
        black_box(f());
    }

    let mut times = Vec::with_capacity(ITERS);
    let mut last_ret = 0.0;
    for _ in 0..ITERS {
        let start = Instant::now();
        last_ret = black_box(f());
        let elapsed = start.elapsed();
        times.push(elapsed);
    }

    BenchResult {
        name,
        times,
        final_return: last_ret,
    }
}

// ---------------------------------------------------------------------------
// Memory estimation
// ---------------------------------------------------------------------------

/// Total data memory for one representation: price matrix + quantities + output.
fn data_memory<T>() -> usize {
    let prices = INSTRUMENTS * DAYS * size_of::<T>();
    let qty = POSITIONS * size_of::<T>();
    let output = DAYS * size_of::<T>();
    prices + qty + output
}
