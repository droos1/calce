# Backlog

Deferred improvements and future work, roughly prioritized.

## High Priority

### Auth integration tests against test DB

Full test coverage for auth flows against a real Postgres instance:
- Login: happy path, wrong password, account lockout after 10 failures, lockout expiry
- Refresh: rotation, replay detection, grace period, expired token
- Logout: family revocation, double-logout idempotency
- Rate limiting: 429 after threshold, Retry-After header value
- API key CRUD: create, list (excludes revoked), revoke, cache eviction
- API key authentication: valid key, expired key, revoked key
- Org scoping: API key can only manage its own org's keys
- Email validation: missing @, empty parts, no domain dot

### Org-scoped user data access for API keys

API keys can't access user-scoped routes (`/v1/users/{id}/...`) because
the permissions layer denies org-scoped admins by default (safe default).
Route handlers need an async org-membership check: verify the target user
belongs to the API key's org via DB lookup before granting access.

### JWT instant revocation

15-min TTL-based expiration is the current plan. If compliance or security
requirements demand instant revocation, add a JWT blacklist with pub/sub
cache invalidation. See `working_docs/auth-implementation-plan.md`.

### Refresh token cleanup job

`refresh_tokens` table grows indefinitely with expired/revoked rows.
Add periodic cleanup: `DELETE FROM refresh_tokens WHERE expires_at < NOW() - INTERVAL '7 days'`.
Could be pg_cron, an in-process background task, or a scheduled Cloud Run job.

## Medium Priority

### Trade ID for audit trails

`Trade` currently has no unique identifier. Needed for:
- Deduplication (same trade arriving twice from different sources)
- Reconciliation (matching trades to broker confirmations)
- Audit logs (which trades contributed to a position)

## Low Priority

### InstrumentId with cheap cloning

`InstrumentId(String)` allocates on clone. For hot paths (HashMap lookups), consider `InstrumentId(Arc<str>)` for refcount-based cloning. Profile before optimizing.

### Price currency

`Price` is just a `Decimal` with no currency. In reality, a price has a currency (the instrument's listing currency). Currently the position's currency serves this role. Consider `Price { amount: Decimal, currency: Currency }` if cross-listed instruments become relevant.
