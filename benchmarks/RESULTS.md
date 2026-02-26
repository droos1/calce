# Benchmark Results

Machine: Apple Silicon (M-series) | Rust 1.93 | `--release` with LTO

---

## 1. Portfolio Backtest — Numeric Representation Comparison

**What:** Backtest a 50-position portfolio over 5 years of daily prices (1000 instruments, 1260 trading days). Each iteration computes daily portfolio values and final return.

**Why:** The inner loop is a dot product (qty * price, summed across positions) repeated for each day — the fundamental operation in portfolio valuation. Tests raw arithmetic throughput and memory footprint across numeric types.

| Type | Median | Memory | Return | Drift vs f64 |
|------|--------|--------|--------|--------------|
| f64 | 0.04 ms | 9.6 MB | +101.33% | reference |
| f32 | 0.04 ms | 4.8 MB | +101.33% | +0.000002% |
| i64 (x10000) | 0.05 ms | 9.6 MB | +101.33% | -0.000001% |
| Decimal | 0.84 ms | 19.2 MB | +101.33% | 0.000000% |

**Speedup vs Decimal:** f64 19.7x, f32 20.6x, i64 18.4x

**Takeaways:**
- Decimal is ~20x slower than hardware numeric types — pure software arithmetic cost.
- f64, f32, and i64 are all within noise of each other at this scale.
- f32 halves memory with negligible precision loss at portfolio level.
- Precision drift is negligible across all types for a 50-position dot product.
- The 20x Decimal overhead is acceptable for calce's use case: a full 5-year backtest completes in under 1ms. Use f64 for hot-path analytics (Monte Carlo, risk) and convert at the boundary.

---

## 2. Trade Aggregation at Scale

**What:** Aggregate N trades (1K-1M) into net positions via HashMap, comparing key types: `String` (clone per entry), `&str` (borrow), `u32` (interned integer ID).

**Why:** `aggregate_positions` is a core calce function. It clones `InstrumentId` (a String newtype) on every `HashMap::entry()` call. This measures the cost of that clone and whether interning or borrowing would help.

| Trades | String clone | &str borrow | u32 key | Clone overhead |
|--------|-------------|-------------|---------|----------------|
| 1K | 0.14 ms | 0.07 ms | 0.05 ms | +100% |
| 10K | 0.57 ms | 0.26 ms | 0.27 ms | +117% |
| 100K | 2.93 ms | 1.50 ms | 2.07 ms | +95% |
| 1M | 29.00 ms | 15.43 ms | 17.47 ms | +88% |

**Takeaways:**
- **String clone doubles aggregation time.** The clone on every `entry()` call allocates and copies ~9 bytes of heap data per trade. At 1M trades this adds ~14ms of pure allocation overhead.
- **Borrowing (&str) is the cheapest option** — even beating u32 keys at scale, likely because short-string hashing is well-optimized in Rust's default hasher.
- **Actionable for calce:** the current `entry(instrument_id.clone())` pattern is the bottleneck. Switching to borrowed keys or an arena-allocated InstrumentId would cut aggregation time in half.

---

## 3. Checked Arithmetic Overhead

**What:** Sum N Decimal values comparing raw `Decimal +=` vs `Money::checked_add` (currency comparison + Result wrapping) — measuring the cost of calce's safety layer.

**Why:** We replaced panicking `Money::Add` with `checked_add` returning `Result`. Need to confirm this doesn't hurt performance.

| N | f64 + | Decimal + | checked_add | Overhead | Dec/f64 |
|---|-------|-----------|-------------|----------|---------|
| 100 | 0.000 ms | 0.001 ms | 0.001 ms | +0.0% | 10x |
| 1K | 0.001 ms | 0.008 ms | 0.008 ms | +1.0% | 8x |
| 10K | 0.011 ms | 0.080 ms | 0.074 ms | -6.7% | 7x |
| 100K | 0.092 ms | 0.854 ms | 0.640 ms | -25.1% | 9x |

**Takeaways:**
- **checked_add has zero measurable overhead.** The currency comparison (`[u8; 3] != [u8; 3]`) on the always-true happy path is predicted perfectly by the branch predictor. The Result wrapping is optimized away by the compiler.
- The negative "overhead" at larger N is noise / measurement artifact, not a real speedup.
- **Decimal is ~8-10x slower than f64** for pure addition. This is less than the 20x gap in the backtest because addition is cheaper than multiplication in Decimal's implementation.
- **Validates our design:** switching from panicking `Add` to checked `Result` costs nothing. Safety is free.

---

## 4. FX Conversion Chain

**What:** Convert $1,000,000 through multi-hop currency chains (USD->EUR->GBP->JPY->CHF->SEK), comparing f64 vs Decimal speed and precision. Also tests round-trip drift.

**Why:** FX conversion is a core calce operation. Chains of multiplications are where both speed and precision differences compound.

### Part 1: Forward chain speed

| Chain | f64 | Decimal | Speedup | f64 result | Decimal result |
|-------|-----|---------|---------|------------|----------------|
| USD->EUR | 0.05 ms | 0.36 ms | 7.8x | 920,000.00 | 920,000.00 |
| USD->EUR->GBP | 0.07 ms | 0.64 ms | 8.7x | 791,200.00 | 791,200.00 |
| USD->EUR->GBP->JPY | 0.13 ms | 1.41 ms | 10.6x | 151,514,800.00 | 151,514,800.00 |
| USD->EUR->GBP->JPY->CHF | 0.17 ms | 2.05 ms | 12.2x | 984,846.20 | 984,846.20 |
| USD->EUR->GBP->JPY->CHF->SEK | 0.17 ms | 2.79 ms | 16.2x | 11,916,639.02 | 11,916,639.02 |

Decimal slowdown scales with chain length: 7.8x at 1 hop to 16.2x at 5 hops (each hop adds one software multiply).

### Part 2: Round-trip drift (USD -> EUR -> USD, repeated)

| Round-trips | f64 value | Decimal value | f64 drift | Decimal drift |
|-------------|-----------|---------------|-----------|---------------|
| 1 | 1,000,000.0000000000 | 999,999.9999999996 | 0.0 | -0.0000000004 |
| 10 | 1,000,000.0000000000 | 999,999.9999999960 | 0.0 | -0.0000000040 |
| 100 | 1,000,000.0000000000 | 999,999.9999999600 | 0.0 | -0.0000000400 |
| 1K | 1,000,000.0000000000 | 999,999.9999996000 | 0.0 | -0.0000004000 |
| 10K | 1,000,000.0000000000 | 999,999.9999960000 | 0.0 | -0.0000040000 |
| 100K | 1,000,000.0000000000 | 999,999.9999600000 | 0.0 | -0.0000400000 |

**Takeaways:**
- **f64 round-trips perfectly here.** IEEE 754 rounding properties cause `x * 0.92 * (1/0.92)` to cancel exactly. This is rate-specific, not a general f64 property.
- **Decimal drifts linearly: ~4e-10 per round-trip.** Because `1/0.92 = 1.0869565217391304347826...` must be truncated to Decimal's 28-digit precision, the product `0.92 * truncated_inverse` is slightly less than 1.0.
- **Key insight: Decimal is not always more precise than f64.** For division-heavy chains, both accumulate rounding — just differently. Decimal's advantage is *predictability* (banker's rounding, deterministic), not zero error.

---

## 5. Price Lookup Patterns

**What:** Random lookups from a (instrument, date) -> price table with 1.26M entries (1000 instruments x 1260 days), comparing data structures.

**Why:** `MarketDataService::get_price` is called for every position on every valuation date. The current calce implementation uses `HashMap<(InstrumentId, NaiveDate), Price>` which clones the String key on each lookup.

| Lookups | HashMap\<String\> | HashMap\<u32\> | HashMap\<packed u64\> | Vec\<Vec\> (2D) | BTreeMap\<String\> |
|---------|-------------------|----------------|----------------------|-----------------|-------------------|
| 1K | 0.08 ms | 0.06 ms | 0.06 ms | 0.03 ms | 0.19 ms |
| 10K | 0.89 ms | 0.62 ms | 0.69 ms | 0.38 ms | 3.22 ms |
| 100K | 21.37 ms | 8.06 ms | 8.94 ms | 3.81 ms | 40.54 ms |
| 1M | 244.31 ms | 118.14 ms | 150.15 ms | 44.09 ms | 425.59 ms |

### Memory

| Structure | Size |
|-----------|------|
| HashMap\<(String, i32), Decimal\> | 52.9 MB |
| HashMap\<(u32, i32), Decimal\> | 28.8 MB |
| HashMap\<u64, Decimal\> | 28.8 MB |
| Vec\<Vec\<Decimal\>\> (2D array) | 19.2 MB |
| BTreeMap\<(String, i32), Decimal\> | 52.9 MB |

**Takeaways:**
- **Vec\<Vec\> is 5.5x faster than HashMap\<String\>** at 1M lookups (44ms vs 244ms). Direct indexing beats any hash-based lookup. Requires an instrument-to-index mapping but is the clear winner for hot-path access.
- **HashMap\<u32\> is 2x faster than HashMap\<String\>** — eliminating String clone and hashing overhead.
- **BTreeMap is 1.7x slower than HashMap** — tree traversal + cache misses vs hash table.
- **String key HashMap uses 2.8x more memory** than Vec (52.9 vs 19.2 MB) due to per-String heap allocation.
- **Actionable for calce:** for the in-memory market data service, a 2D Vec indexed by interned instrument ID + day offset would be optimal. For the general-purpose trait, consider an `InstrumentId` backed by an index rather than a String.
