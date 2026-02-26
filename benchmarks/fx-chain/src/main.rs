//! FX conversion chain benchmark.
//!
//! Converts money through multi-hop currency chains (1-5 hops) comparing
//! f64 vs Decimal speed. Also tests round-trip precision drift by
//! converting back and forth many times.
//!
//! Run with: cargo run --release

use std::hint::black_box;
use std::time::Instant;

use rust_decimal::prelude::*;
use rust_decimal::Decimal;

const ITERS: usize = 50;
const CONVERSIONS: usize = 100_000;

fn main() {
    if cfg!(debug_assertions) {
        println!("WARNING: debug mode. Run `cargo run --release` for accurate timings.\n");
    }

    // Realistic FX rates
    let rates_f64: [(f64, f64); 5] = [
        (0.92, 1.0 / 0.92),       // USD->EUR, EUR->USD
        (0.86, 1.0 / 0.86),       // EUR->GBP, GBP->EUR
        (191.50, 1.0 / 191.50),   // GBP->JPY, JPY->GBP
        (0.0065, 1.0 / 0.0065),   // JPY->CHF, CHF->JPY
        (12.10, 1.0 / 12.10),     // CHF->SEK, SEK->CHF
    ];

    let rates_dec: [(Decimal, Decimal); 5] = rates_f64.map(|(fwd, inv)| {
        (
            Decimal::from_f64(fwd).unwrap(),
            Decimal::from_f64(inv).unwrap(),
        )
    });

    let amount_f64 = 1_000_000.0_f64;
    let amount_dec = Decimal::new(1_000_000, 0);

    // -----------------------------------------------------------------------
    // Part 1: Multi-hop chain speed + precision
    // -----------------------------------------------------------------------
    let chain_names = [
        "USD->EUR",
        "USD->EUR->GBP",
        "USD->EUR->GBP->JPY",
        "USD->EUR->GBP->JPY->CHF",
        "USD->EUR->GBP->JPY->CHF->SEK",
    ];

    println!("FX Conversion Chain Benchmark");
    println!("=============================");
    println!("Amount: $1,000,000 | Conversions: {CONVERSIONS} | Iterations: {ITERS}\n");

    println!("Part 1: Forward chain speed");
    println!(
        "{:<32} {:>10} {:>10} {:>10} {:>18} {:>18}",
        "Chain", "f64", "Decimal", "Speedup", "f64 result", "Decimal result"
    );
    println!("{}", "-".repeat(104));

    for hops in 1..=5 {
        // f64 chain
        let f64_result = chain_f64(amount_f64, &rates_f64[..hops]);
        let t_f64 = bench_median(|| {
            let mut v = 0.0_f64;
            for _ in 0..CONVERSIONS {
                v = chain_f64(amount_f64, &rates_f64[..hops]);
            }
            black_box(v);
        });

        // Decimal chain
        let dec_result = chain_dec(amount_dec, &rates_dec[..hops]);
        let t_dec = bench_median(|| {
            let mut v = Decimal::ZERO;
            for _ in 0..CONVERSIONS {
                v = chain_dec(amount_dec, &rates_dec[..hops]);
            }
            black_box(v);
        });

        let speedup = t_dec / t_f64;

        println!(
            "{:<32} {:>7.2} ms {:>7.2} ms {:>8.1}x {:>18.6} {:>18}",
            chain_names[hops - 1],
            t_f64,
            t_dec,
            speedup,
            f64_result,
            dec_result,
        );
    }

    // -----------------------------------------------------------------------
    // Part 2: Round-trip drift (USD->EUR->USD, repeated N times)
    // -----------------------------------------------------------------------
    println!("\nPart 2: Round-trip drift (USD -> EUR -> USD, repeated)");
    println!(
        "{:<14} {:>22} {:>22} {:>16} {:>16}",
        "Round-trips", "f64 value", "Decimal value", "f64 drift", "Decimal drift"
    );
    println!("{}", "-".repeat(96));

    let fwd_f64 = rates_f64[0].0;
    let inv_f64 = rates_f64[0].1;
    let fwd_dec = rates_dec[0].0;
    let inv_dec = rates_dec[0].1;

    for &trips in &[1, 10, 100, 1_000, 10_000, 100_000] {
        let mut v_f64 = amount_f64;
        for _ in 0..trips {
            v_f64 = v_f64 * fwd_f64 * inv_f64;
        }

        let mut v_dec = amount_dec;
        for _ in 0..trips {
            v_dec = v_dec * fwd_dec * inv_dec;
        }

        let drift_f64 = v_f64 - amount_f64;
        let drift_dec = (v_dec - amount_dec).to_f64().unwrap_or(0.0);

        println!(
            "{:<14} {:>22.10} {:>22} {:>+16.10} {:>+16.10}",
            format_count(trips),
            v_f64,
            v_dec,
            drift_f64,
            drift_dec,
        );
    }
}

#[inline]
fn chain_f64(mut amount: f64, rates: &[(f64, f64)]) -> f64 {
    for &(fwd, _) in rates {
        amount *= fwd;
    }
    amount
}

#[inline]
fn chain_dec(mut amount: Decimal, rates: &[(Decimal, Decimal)]) -> Decimal {
    for &(fwd, _) in rates {
        amount *= fwd;
    }
    amount
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
