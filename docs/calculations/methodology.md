# Calculation Reference

Specification of calculation methodology used in Calce.

Each calculation is tagged (e.g. `#CALC_MV`). The same tag appears in the
implementing source code, enabling cross-referencing between specification and
implementation via simple text search.

---

## 1. Assumptions

- Markets are liquid and positions can be valued at the last observed price.
- FX rates are point-in-time spot rates; no bid/ask spread is modelled.
- Portfolio value is additive across positions (no netting or margin offsets).
- No intraday granularity; all calculations operate on daily snapshots.

## 2. Conventions

**Base currency** — All top level results for a user are expressed in a single base
currency (e.g. SEK). Cross-currency positions are converted to base currency
using the applicable FX rate. The base currency is a parameter to every
calculation that produces monetary totals.

**Signed quantities** — Positive = long, negative = short. A buy trade adds
positive quantity; a sell adds negative. Net quantity determines the current
direction of a position.

**FX rate directionality** — Rates always carry explicit `from` and `to`
currencies. `FxRate(USD, SEK, 10.5)` means 1 USD = 10.5 SEK. Conversion
validates that the rate direction matches the source currency.

## 3. Market Data

**Instrument prices** — Daily close prices. For a given valuation date T, the
price used is typically T-1 close (last available end-of-day price).

**FX rates** — Daily spot rates including the current date. Rates are directed:
an `FxRate(from, to, rate)` means 1 unit of `from` = `rate` units of `to`.

**Temporal scope** — All market data lookups are keyed by date. If data is
missing for a requested date the calculation fails explicitly — no interpolation
or fill-forward. This means valuations on non-business days (weekends, holidays)
will fail unless market data is explicitly provided for those dates.

## 4. Calculations

### 4.1 Position Aggregation `#CALC_POS_AGG`

Derives current holdings from trade history.

Given a set of trades and a valuation date T:

    net_quantity(instrument) = sum of trade.quantity
                               for all trades where trade.date <= T

Positions with net quantity of zero (fully closed) are excluded from the result.

---

### 4.2 Market Value `#CALC_MV`

Values each position at current market prices, converting to base currency
where needed.

For each position:

    market_value        = quantity * price(instrument, T)
    market_value_base   = market_value * fx_rate(position_ccy, base_ccy, T)

When position currency equals base currency, no FX conversion is applied.

Portfolio total:

    total = sum of market_value_base across all positions

---

### 4.3 Value Change `#CALC_VCHG`

Measures the change in portfolio market value between two points in time.

Given portfolio value V(T) and a prior value V(T-n):

    change     = V(T) - V(T-n)
    change_pct = change / V(T-n)

Percentage change is undefined when V(T-n) = 0.

**Standard periods:**

| Period  | Comparison date              |
|---------|------------------------------|
| Daily   | T - 1 day                    |
| Weekly  | T - 7 days                   |
| Yearly  | T - 1 year (leap-year safe)  |
| YTD     | Dec 31 of previous year      |

Leap year handling: when T is Feb 29 and the prior year has no Feb 29, Feb 28
is used.

---

### 4.4 Portfolio Report `#CALC_REPORT`

Composed view that bundles market value and value changes into a single result,
avoiding redundant computation.

Internally:

1. Aggregate trades into positions (`#CALC_POS_AGG`)
2. Value positions at current date (`#CALC_MV`) → `MarketValueResult`
3. Pass the pre-computed current snapshot into value change summary (`#CALC_VCHG`)

The current-date market value is computed once and shared between the MV result
and the value change calculation.

Result:

    PortfolioReport {
        market_value:      MarketValueResult,    // positions + total
        value_changes:     ValueChangeSummary,   // daily/weekly/yearly/YTD
        type_allocation:   TypeAllocation,        // breakdown by instrument type (#CALC_ALLOC_INSTYPE)
        sector_allocation: AllocationResult,      // breakdown by GICS sector (#CALC_ALLOC_SECTOR)
    }

---

### 4.5 Type Allocation `#CALC_ALLOC_INSTYPE`

Groups portfolio positions by instrument type and computes each type's share of
total portfolio value. Operates on the output of `#CALC_MV` and resolves
instrument types from market data metadata.

**Instrument types:** Stock, Bond, Etf, MutualFund, Certificate, Option,
Warrant, StructuredProduct, Future, Other (default for unknown).

**Inputs:**

- `positions` — valued positions from `#CALC_MV`
- `total` — portfolio total in base currency
- `market_data` — provides instrument type lookup per instrument

**Computation:**

For each instrument type T present in the portfolio:

    type_value(T) = sum of market_value_base for positions where type = T
    weight(T)     = type_value(T) / total

Result entries are sorted by descending weight.

When total market value is zero, all weights are zero.

**Short positions** — Short positions have negative `market_value_base`, so
allocation computes net exposure per type. Weights can fall outside `[0, 1]`
(e.g. long stocks 150k, short bonds -50k, total 100k → stock weight 1.5,
bond weight -0.5). This is correct for a net allocation view. If gross
exposure or separate long/short buckets are needed, this should be extended.

---

### 4.6 Weighted Allocation `#CALC_ALLOC_WEIGHTED`

Generic engine for computing portfolio allocation across any weighted
classification dimension (sector, geography, asset class, etc.).

Unlike instrument type allocation (`#CALC_ALLOC_INSTYPE`), which assigns each
instrument a single label, weighted allocation supports instruments that span
multiple categories. A stock has a single sector at weight 1.0, but a mutual
fund or ETF distributes across many sectors (e.g. 30% Information Technology,
13% Health Care). This achieves look-through allocation without requiring
explicit fund holdings data.

**Inputs:**

- `positions` — valued positions from `#CALC_MV`
- `total` — portfolio total in base currency
- `dimension` — the classification dimension (e.g. "sector", "geography")
- `get_weights` — function returning allocation weights per instrument:
  `instrument → [(category, weight)]`

**Allocation weights** are stored per instrument as `{category: weight}` maps.
Stocks typically have one entry at weight 1.0. Funds have multiple entries that
should sum to approximately 1.0 (may be less due to cash drag or rounding).

**Computation:**

For each position P with market value V (in base currency):

    weights = get_weights(P.instrument_id)
    for each (category, w) in weights:
        attributed_value(category) += V * w

Then for each category C:

    total_value(C) = sum of attributed_value(C) across all positions
    portfolio_weight(C) = total_value(C) / total

Result entries are sorted by descending portfolio weight.

**Edge cases:**

- Instrument with no allocation data for the dimension → maps to
  "Uncategorized" with weight 1.0.
- Allocation weights that sum to less than 1.0 → the unallocated portion is
  silently lost (not attributed to any category). This is intentional: cash
  drag and rounding gaps should not inflate any category.
- Allocation weights that sum to more than 1.0 → the excess is passed through.
  The data provider is responsible for normalization.
- When total market value is zero, all portfolio weights are zero.
- Short positions distribute negative value across categories, same as
  `#CALC_ALLOC_INSTYPE`.

**Result:**

    AllocationResult {
        dimension: String,
        entries: Vec<AllocationEntry>,   // sorted by descending weight
        total: Money,
    }

    AllocationEntry {
        key: String,           // category name (e.g. "Information Technology")
        market_value: Money,   // value attributed to this category
        weight: f64,           // fraction of total portfolio
    }

---

### 4.7 Sector Allocation `#CALC_ALLOC_SECTOR`

Portfolio allocation by GICS sector, computed via `#CALC_ALLOC_WEIGHTED` with
dimension = "sector".

Reads per-instrument sector weights from market data metadata. Stocks are
assigned a single GICS sector at weight 1.0. Funds and ETFs carry a
multi-sector breakdown reflecting their underlying holdings.

**GICS top-level sectors** (11, per MSCI/S&P standard): Communication Services,
Consumer Discretionary, Consumer Staples, Energy, Financials, Health Care,
Industrials, Information Technology, Materials, Real Estate, Utilities.

Sector names are free-form strings (not an enum) to accommodate different
classification providers (GICS, ICB, Morningstar). The calculation does not
validate sector names.

---

### 4.8 Volatility `#CALC_VOL`

Historical realized volatility for a single instrument, computed as the
annualized standard deviation of logarithmic daily returns.

**Inputs:**

- `instrument` — the instrument to compute volatility for
- `as_of_date` — the reference date (typically today)
- `lookback_days` — number of calendar days of history to use

**Step 1 — Collect prices:**

Retrieve all available closing prices for the instrument in the window
`[as_of_date − lookback_days, as_of_date]`. Discard any prices that are zero
or negative — these represent missing or invalid data, not genuine market
observations.

**Step 2 — Validate data quality:**

The remaining valid prices must satisfy three conditions, otherwise the calculation fails:

1. *Minimum count* — at least 2 valid prices (otherwise no return can be
   computed).
2. *Minimum history depth* — the earliest valid price must be at least 60
   calendar days before `as_of_date`, ensuring the sample is not too short
   to be meaningful.
3. *Completeness* — of all price records returned by the data source for
   the period `[earliest_valid_price_date, as_of_date]`, at least 80% must
   have a positive (non-zero) value. This catches instruments where the
   data provider returns rows with zero or null prices rather than omitting
   them entirely.


**Step 3 — Compute log returns:**

    r_i = ln(P_i / P_{i-1})    for consecutive valid prices P_0 .. P_n

**Step 4 — Compute volatility:**

    daily_volatility     = sample_std_dev(r_0 .. r_{n-1})
    annualized_volatility = daily_volatility * sqrt(252)

Sample standard deviation uses Bessel's correction (divide by n-1).

**Result:**

    VolatilityResult {
        annualized_volatility: f64,
        daily_volatility:      f64,
        num_observations:      usize,   // count of returns = valid prices - 1
        start_date:            date,    // first valid price date
        end_date:              date,    // last valid price date
    }

When the standard deviation is NaN or infinite (e.g. all prices identical),
`InsufficientData` is returned.

This calculation requires a `get_price_history` method on `MarketDataService`
that returns all available prices for an instrument over a date range.

**TODO:** This implementation assumes daily trading and annualizes with
`√252`. Instruments that trade monthly (e.g. some mutual funds with monthly
NAVs) should annualize with `√12` instead. Rather than auto-detecting
frequency from the price data — which is fragile and can misclassify illiquid
instruments — trading frequency should be an explicit attribute on the
instrument metadata. When instrument metadata is available, add a frequency
parameter and branch the annualization factor accordingly.

---

## 5. Accounting

### 5.1 Ledger Balance `#CALC_LEDGER_BAL`

Sums ledger entries to produce an exact balance. Uses fixed-point decimal
arithmetic to guarantee that debits and credits balance to zero without
floating-point rounding errors.

    balance = sum of entry.amount for all entries

All entries must share the same currency; mixed-currency summation is rejected.
