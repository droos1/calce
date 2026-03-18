# Sector Allocation Plan

Add portfolio allocation by sector (`#CALC_ALLOC_SECTOR`), following the same
pattern as the existing type allocation (`#CALC_ALLOC`).

## Context

The old njorda engine supported allocation across 7+ dimensions including sector.
We start with sector as the next dimension after instrument type. The DB
currently has **no sector column** — we need a migration and schema change.

Sectors are metadata on instruments, like `instrument_type`. The allocation
calculation itself is structurally identical to type allocation: group positions
by sector, sum `market_value_base`, compute weights.

## Design Decisions

**Sector as a free-form string, not an enum.** Unlike instrument types (10 fixed
values), sector taxonomies vary by data provider (GICS has 11, ICB has 11,
Morningstar has 16, the old njorda engine had 16). Hard-coding an enum would
couple us to one taxonomy. Instead:
- Store sector as `Option<String>` on instruments
- Positions without a sector map to `"Uncategorized"`
- The allocation function works with `&str` keys, not enum variants

This matches how real portfolio systems handle sectors — the classification
comes from the data provider and changes over time.

**Reuse the allocation machinery.** The actual grouping logic is identical to
type allocation. We could generalize into a single `group_allocation()` function
that takes a key-extraction closure, then both type and sector allocation become
thin wrappers. This avoids code duplication and makes adding future dimensions
(geography, asset class) trivial.

## Phases (per /new-calculation skill)

### Phase 1: Methodology docs
Write `#CALC_ALLOC_SECTOR` section in `docs/calculations/methodology.md`.
Consider refactoring the existing `#CALC_ALLOC` docs to reference a shared
"grouped allocation" concept.

### Phase 2: Tests
- Unit tests in `calc/allocation.rs` for sector allocation
- Test the generalized grouping function with custom key extractors
- Integration test with multi-sector portfolio in `tests/integration_test.rs`
- Python test for `sector_allocation` on `PortfolioReport`

### Phase 3: Core implementation

#### 3a. Domain — sector on `MarketDataService`
- Add `get_sector(&self, instrument: &InstrumentId) -> Option<&str>` default
  method to `MarketDataService` returning `None`
- Implement on `TestMarketData` with `HashMap<InstrumentId, String>`
- Implement on `InMemoryMarketDataService` with `HashMap<InstrumentId, String>`

#### 3b. Generalize allocation — `calc/allocation.rs`
Refactor to extract a generic core:

```rust
pub fn group_allocation<F>(
    positions: &[ValuedPosition],
    total: Money,
    key_fn: F,
) -> Vec<AllocationEntry<String>>
where
    F: Fn(&InstrumentId) -> String,
```

Then `type_allocation` and `sector_allocation` become:
```rust
pub fn type_allocation(...) -> TypeAllocation {
    // calls group_allocation with |id| market_data.get_instrument_type(id).to_string()
}

pub fn sector_allocation(...) -> SectorAllocation {
    // calls group_allocation with |id| market_data.get_sector(id).unwrap_or("Uncategorized")
}
```

Existing `TypeAllocation` keeps its typed `InstrumentType` field — the generic
core returns strings, and `type_allocation` maps back to the enum.

#### 3c. `PortfolioReport`
- Add `pub sector_allocation: SectorAllocation` field
- Compute alongside type allocation using the already-available `market_data`

### Phase 4: Data layer

#### 4a. DB migration
- Add `sector VARCHAR(50) DEFAULT NULL` to `instruments` table
- New Alembic migration in `services/calce-db/`

#### 4b. Queries
- `list_instruments()` → SELECT `sector` too; return 5-tuple
- `insert_instrument()` → accept optional `sector` param

#### 4c. Loader
- Map the sector from `InstrumentSummary` into `md.add_sector()` before freeze

#### 4d. `InstrumentSummary`
- Add `pub sector: Option<String>`

### Phase 5: Seed data & sanity check

#### 5a. API seed (`seed.rs`)
- Assign sectors to AAPL ("Technology"), VOW3 ("Consumer Cyclical"), SPY (None —
  ETFs span sectors)
- Verify allocation shows in portfolio report

#### 5b. DB seed tool (`seed_db.py`)
- Add `SECTORS` list with weighted random assignment
- Update `gen_instruments` to include sector

### Phase 6: Python bindings & API

#### 6a. Python
- `results.rs`: Add `SectorAllocationEntry` and `SectorAllocation` pyclass wrappers
- Add `sector_allocation` getter to `PortfolioReport`
- `services.rs`: Add `add_sector(instrument_id, sector)` to `MarketData`
- `lib.rs`: Register new classes

#### 6b. API
- `PortfolioReport` serde already covers it — no new endpoints needed
- Sector data appears in the existing portfolio report response

### Phase 7: Tests pass, final verification
- `invoke check` — formatting, clippy
- `invoke test` — full suite (Rust + Python)

## What does NOT change
- `ValuedPosition`, `Position`, `Trade` — sector is instrument metadata
- `CalculationContext` — no new parameters
- Existing `TypeAllocation` API — fully backward compatible
- `value_change.rs`, `aggregation.rs`, `volatility.rs` — untouched

## Open Questions
1. Should we generalize allocation now (extracting `group_allocation`) or just
   copy the pattern from type allocation? Generalizing is cleaner but adds scope.
2. Sector values: free-form string, or a loose enum with `Other` fallback like
   `InstrumentType`? Free-form is more flexible but loses type safety.
