//! Checked arithmetic overhead benchmark.
//!
//! Compares raw `Decimal + Decimal` vs a `checked_add` that validates
//! currency equality and returns Result — measuring the cost of calce's
//! safety layer.
//!
//! Run with: cargo run --release

use std::hint::black_box;
use std::time::Instant;

use rust_decimal::Decimal;

const ITERS: usize = 50;

/// Mirrors calce's Money struct.
#[derive(Clone, Copy)]
struct Money {
    amount: Decimal,
    currency: [u8; 3],
}

impl Money {
    #[inline]
    fn checked_add(self, other: Self) -> Result<Self, ()> {
        if self.currency != other.currency {
            return Err(());
        }
        Ok(Money {
            amount: self.amount + other.amount,
            currency: self.currency,
        })
    }
}

fn main() {
    if cfg!(debug_assertions) {
        println!("WARNING: debug mode. Run `cargo run --release` for accurate timings.\n");
    }

    let counts = [100, 1_000, 10_000, 100_000];

    println!("Checked Arithmetic Overhead Benchmark");
    println!("=====================================");
    println!("Compares raw Decimal += vs Money::checked_add (currency check + Result)\n");

    println!(
        "{:<8} {:>12} {:>12} {:>12} {:>12} {:>10}",
        "N", "f64 +", "Decimal +", "checked_add", "Overhead", "Dec/f64"
    );
    println!("{}", "-".repeat(72));

    for &n in &counts {
        // Data: all same currency, stored in structs to prevent constant folding
        let moneys: Vec<Money> = (0..n)
            .map(|i| Money {
                amount: Decimal::new(i as i64 * 100 + 1, 2),
                currency: *b"USD",
            })
            .collect();
        let decimals: Vec<Decimal> = moneys.iter().map(|m| m.amount).collect();
        let floats: Vec<f64> = (0..n).map(|i| i as f64 + 0.01).collect();

        // f64 raw sum
        let t_f64 = bench_median(|| {
            let mut sum = 0.0_f64;
            for &v in &floats {
                sum += v;
            }
            black_box(sum);
        });

        // Decimal raw sum (just +=, no currency check)
        let t_raw = bench_median(|| {
            let mut sum = Decimal::ZERO;
            for &v in &decimals {
                sum += v;
            }
            black_box(sum);
        });

        // Checked add (currency comparison + Result unwrap)
        let t_checked = bench_median(|| {
            let mut acc = Money {
                amount: Decimal::ZERO,
                currency: *b"USD",
            };
            for &m in &moneys {
                acc = acc.checked_add(m).unwrap_or(acc);
            }
            black_box(acc.amount);
        });

        let overhead = (t_checked / t_raw - 1.0) * 100.0;
        let dec_vs_f64 = t_raw / t_f64;

        println!(
            "{:<8} {:>9.3} ms {:>9.3} ms {:>9.3} ms {:>+11.1}% {:>9.0}x",
            format_count(n),
            t_f64,
            t_raw,
            t_checked,
            overhead,
            dec_vs_f64,
        );
    }

    println!("\n\"Overhead\"  = (checked_add - raw Decimal) / raw Decimal");
    println!("\"Dec/f64\"   = how much slower Decimal is than f64 for pure addition");
}

fn format_count(n: usize) -> String {
    if n >= 1_000 {
        format!("{}K", n / 1_000)
    } else {
        n.to_string()
    }
}

fn bench_median(mut f: impl FnMut()) -> f64 {
    for _ in 0..5 {
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
