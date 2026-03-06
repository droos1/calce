# calce-data

Async data access layer. Implements calce-core's service traits against real databases.

## Module Layout

| Module | Purpose |
|--------|---------|
| `repo/market_data.rs` | Prices and FX rates (single lookups, history ranges, batch loading) |
| `repo/user_data.rs` | Users, accounts, and trades |
| `loader.rs` | AsyncCalcEngine — async orchestration bridging repos to sync calce-core |
| `config.rs` | Database connection configuration |
| `njorda/` | Read-only njorda backend access (feature-gated: `njorda`) |

## Database

Local Postgres via Docker (port 5433). Schema managed by sqlx migrations.

```sh
invoke db       # start
invoke db-stop  # stop
```
