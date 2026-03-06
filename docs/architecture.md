# Architecture

Financial calculation engine for portfolio tracking, valuation, and analytics.

## Design Principles

1. **Pure calculations, impure boundaries** — calculation functions are pure (data in, result out). Side effects (data loading, auth) live at the edges.
2. **Dual API** — every calculation is available in two modes: _stateful_ (engine loads data, then calculates) and _stateless_ (caller provides data directly).
3. **Plain data types** — domain types carry data, not behavior. Business logic lives in `calc/`.
4. **Trait-based data access** — services are traits, swappable for testing, caching, or different backends.
5. **Sync core, async boundaries** — calce-core is 100% sync. Async data loading lives in calce-data. This keeps the core fast to compile, easy to test, and embeddable (PyO3, WASM).

## Crate Structure

```
calce-core    (sync, pure — domain types, calc functions, service traits)
    ↑
calce-data    (async — sqlx repos, AsyncCalcEngine, Postgres access)
    ↑
calce-api     (async — axum HTTP handlers, thin layer over AsyncCalcEngine)

calce-python  (PyO3 bindings, depends on calce-core only)
```

## The Sync/Async Bridge

The central architectural pattern. calce-core defines sync service traits (`MarketDataService`, `UserDataService`) with in-memory implementations. calce-data bridges the gap:

```
API handler → DataLoader.load_user_portfolio(security_ctx, user_id, ...)
  1. Check access via permissions module   (sync)
  2. Load trades from Postgres             (async)
  3. Batch-load prices + FX for positions  (async, avoids N+1)
  4. Build InMemoryMarketDataService       (sync bridge object)
  5. Return PortfolioData { trades, market_data }

API handler → aggregate_positions + value_positions  (sync, calce-core)
```

Data is loaded async in bulk, packed into in-memory structs, then handed to pure sync functions. calce-core never sees a database.

## Dual API

**Stateful** — caller identifies _what_ to calculate (which user). `DataLoader` in calce-data loads data and packs it into in-memory services, then the API handler calls pure calc functions. Used by the HTTP API.

**Stateless** — caller provides all input data directly. No data loading, no auth. Used for simulations, what-if analysis, testing, and as an embeddable library (PyO3).

Both modes call the same pure `calc/` functions underneath.

## Calculation Composition

Calculations compose in layers:

1. **Primitive** — single-purpose pure function: `value_positions(positions, ctx, market_data)`
2. **Composite** — calls primitives at multiple points: `value_change_summary` calls `aggregate_positions` + `value_positions` for each comparison date, then diffs
3. **Report** (`reports/`) — bundles composites into a consumer-facing result, sharing intermediate values to avoid redundant computation

Data loading is separate from calculations: `DataLoader` in calce-data handles async I/O, then the API handler or caller invokes the pure calc layer.

Each level is independently testable. The pure-function design means caching/memoization can be added later by wrapping the same functions.

## Partial Results

Calculations return partial results rather than failing on the first missing data point. A portfolio with 50 positions where 1 price is missing returns 49 valued positions plus a warning.

```rust
pub struct Outcome<T> {
    pub value: T,
    pub warnings: Vec<Warning>,
}
```

Functions return `CalceResult<Outcome<T>>` — the `Result` catches structural errors (e.g. currency mismatch, aggregation conflicts) while `Outcome` collects data-quality warnings (missing prices, missing FX rates) that allow partial computation.

Currently implemented for `value_positions`, `value_change_summary`, and `portfolio_report`.
