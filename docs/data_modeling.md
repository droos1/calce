# Data Modeling

## Primary Keys

Every table gets a `BIGINT GENERATED ALWAYS AS IDENTITY` primary key. Never reused after deletion. Business identifiers (external_id, ticker) are separate unique columns.

## Timestamps

- `created_at` — server-default `now()`, set once on insert
- `updated_at` — server-default `now()`, updated via Postgres `BEFORE UPDATE` trigger

Both are server-managed only — no application code sets them. Add to mutable entity tables. Skip on immutable facts (trades) and bulk time-series (prices, fx_rates).

## Indexes

Only create indexes that serve a known query pattern. No speculative indexes. Review queries in `crates/calce-data/src/queries/` before adding or removing indexes.

## Foreign Keys

All FKs are `BIGINT` referencing the parent's `id`. Default `ON DELETE` is `NO ACTION` (blocks deletion of referenced rows). The Rust query layer JOINs to resolve business keys when loading domain objects.

## Check Constraints

Enforce data invariants at the DB level — don't rely on application validation alone. Examples: non-negative prices, non-zero quantities, positive FX rates.

## Schema Management

Alembic in `services/calce-db/`. Models in `services/calce-db/calce_db/models.py`.

```sh
invoke db-reset    # wipe + migrate + seed (confirmation + 3s countdown)
invoke db-migrate  # apply pending migrations
```
