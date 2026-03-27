# calce-api

Axum HTTP server that wires `calce-core` calculations to REST endpoints.

## Module Structure

```
src/
├── main.rs           — router setup, server startup, integration tests
├── routes/
│   ├── mod.rs        — route registration
│   ├── calc.rs       — calculation + data exploration endpoints
│   ├── users.rs      — user CRUD endpoints (admin-only create/delete)
│   ├── auth.rs       — POST /auth/login, POST /auth/refresh
│   └── api_keys.rs   — API key CRUD (admin-only)
├── auth.rs           — JWT Bearer token extraction → SecurityContext
├── rate_limit.rs     — per-IP token bucket rate limiter (auth endpoints)
├── error.rs          — CalceError → HTTP status code mapping
├── state.rs          — AppState (shared service references)
└── seed.rs           — in-memory test data + seed sanity tests
```

## URL Convention

Two resource scopes:

All calculation and data endpoints require authentication (see `docs/auth.md`).

**User-scoped** — auth + user access check, operates on a user's portfolio:
```
GET /v1/users/{user_id}/market-value?as_of_date=...&base_currency=...
GET /v1/users/{user_id}/portfolio?as_of_date=...&base_currency=...
```

**Instrument-scoped** — auth only (no user access check), operates on market data:
```
GET /v1/instruments/{instrument_id}/volatility?as_of_date=...&lookback_days=...
```

When adding a new endpoint, decide which scope it belongs to:
- If it needs user trades/positions → user-scoped, pass SecurityContext to DataService, call calc functions
- If it only needs market data → instrument-scoped, auth only, call calc function directly

## Authentication

JWT Bearer token, extracted in `auth.rs` via `Authorization: Bearer <token>`.
See `docs/auth.md` for the full auth design (login, refresh, API keys, rate limiting).

Missing or invalid token on any authenticated route → 401 Unauthorized.

## Error Handling

`error.rs` maps both `DataError` (from calce-data) and `CalceError` (from calce-core) to HTTP responses.

| Error variant               | HTTP status | Notes |
|-----------------------------|-------------|-------|
| `DataError::Unauthorized`   | 403         | User lacks access to target user's data |
| `DataError::NoTradesFound`  | 404         | |
| `DataError::NotFound`       | 404         | Generic not-found (e.g. user CRUD) |
| `DataError::Conflict`       | 409         | Unique/FK constraint violation |
| `DataError::InvalidCredentials` | 401     | Wrong email or password |
| `DataError::AccountLocked`  | 423         | Too many failed login attempts |
| `DataError::InvalidRefreshToken` | 401    | Expired or invalid refresh token |
| `DataError::TokenReplayDetected` | 401    | Refresh token reuse — family revoked |
| `ApiError::RateLimited`     | 429         | Per-IP rate limit exceeded |
| `DataError::Sqlx`           | 500         | Database error |
| `DataError::InvalidDbData`  | 500         | Corrupt data in DB |
| `DataError::Calc(inner)`    | delegates   | Maps the inner `CalceError` |
| `CalceError::CurrencyMismatch` | 400      | Client sent conflicting currencies |
| `CalceError::PriceNotFound` | 422         | Missing market data — not a server bug |
| `CalceError::FxRateNotFound` | 422        | Missing market data — not a server bug |
| `CalceError::InsufficientData` | 422      | Not enough price history for calculation |
| `CalceError::CurrencyConflict` | 422      | Instrument currency mismatch |

**Important:** 500 should only occur for genuine server bugs (panics, DB connection
failures), never for missing data or bad input.

Response format:
```json
{"error": "ERROR_CODE", "message": "Human-readable description"}
```

## Testing

Integration tests live in `main.rs` `#[cfg(test)]` and use `tower::ServiceExt::oneshot`
to send requests directly to the router (no real server needed).

Pattern:
```rust
let app = build_router(test_state());
let response = app.oneshot(request).await.unwrap();
```

`test_state()` uses `seed::seed_market_data()` / `seed::seed_user_data()`.
Seed data uses weekday dates only (prices and FX rates skip weekends).
The canonical test date is `2025-03-14` (Friday).
