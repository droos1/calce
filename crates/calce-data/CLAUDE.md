# calce-data

Async data access layer. Provides `DataBackend` trait and implementations for loading
trades and market data from different sources.

## Module Layout

| Module | Purpose |
|--------|---------|
| `auth.rs` | `SecurityContext`, `Role` — user identity and role types |
| `permissions.rs` | `can_access_user_data()` — centralized access-control checks |
| `backend/mod.rs` | `DataBackend` trait — read-only interface used by `DataService` |
| `backend/postgres.rs` | Postgres backend — orchestrates queries into `DataBackend` |
| `backend/in_memory.rs` | In-memory backend for tests |
| `queries/market_data.rs` | Postgres query layer: prices, FX rates (reads + writes) |
| `queries/user_data.rs` | Postgres query layer: users, accounts, trades (reads + writes) |
| `service.rs` | `DataService` — wraps a `DataBackend`, adds auth checks and input assembly |
| `config.rs` | Database connection configuration |

### Layer overview

```
DataService  →  DataBackend (trait)  →  PostgresBackend  →  queries/ (SQL)
                                    →  InMemoryBackend
```

`queries/` is Postgres-specific and includes write methods (inserts) not exposed
through `DataBackend`. It is only used by `PostgresBackend`.

## Database

Local Postgres via Docker (port 5433). Schema managed by sqlx migrations.

```sh
invoke db       # start
invoke db-stop  # stop
```
