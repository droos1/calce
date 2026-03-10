# calce-core

Core Rust library — no DB or async dependencies. Fast to compile, easy to test.

## Module Layout

| Module | Purpose |
|--------|---------|
| `domain/` | Data types only, no business logic |
| `calc/` | Pure business logic, no side effects |
| `accounting/` | Exact-precision ledger arithmetic (Decimal) |
| `reports/` | Composed views bundling multiple calc primitives |
| `services/` | Service traits + in-memory test implementations |
| `inputs.rs` | `CalcInputs` — trades + market data bundle for calculations |
| `outcome.rs` | `Outcome<T>` — partial results with warnings |

## Numeric Types

| Type | Use for | Module |
|------|---------|--------|
| `f64` | Market valuations, risk metrics, FX conversions, portfolio analytics | `domain/`, `calc/` |
| `rust_decimal::Decimal` | Ledger balancing, fee splits, any arithmetic that must be exact | `accounting/` |

Domain types (`Quantity`, `Price`, `Money`, `FxRate`) use `f64`. They derive `PartialEq` but **not** `Eq` (f64 is not `Eq`).

The `accounting` module uses `Decimal` for exact ledger arithmetic where debits and credits must balance to zero.

## Domain Types

Plain data carriers. No business logic beyond intrinsic operations (e.g. `Money::convert`, `FxRate::invert`).

- **Money** — amount + currency, the fundamental financial value
- **Trade** — a single execution (instrument, quantity, price, date)
- **Position** — aggregated holding for one instrument (quantity, no pricing)
- **FxRate** — directed exchange rate (from → to)

An **Account** groups trades under a user with its own currency and label. Account currency drives account-level reporting; cross-account aggregation converts to the base currency from `CalculationContext`.

## Contexts

**`CalculationContext`** — pure parameters (`base_currency`, `as_of_date`). No service references, no state. Passed into every calc function.

## Lint Config

Defined in `lib.rs`:
- `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]`
- `#![warn(clippy::pedantic)]`

These are strict — no unwrap/expect/panic anywhere in this crate.
