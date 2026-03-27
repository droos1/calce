# Auth Implementation Plan

Real authentication replacing the current header-based placeholder.

## Goals

- Users authenticate with email + password, receive a JWT
- Service consumers (customer integrations) authenticate with API keys
- Both resolve to the same `SecurityContext` in a single middleware
- Validation centralized in one module (`calce-data::auth`)
- Standard, simple, minimal surface area — complexity is the enemy of security
- Fast: JWT validation requires no DB hit; API key lookups are cached

## Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| User tokens | JWT (access) + opaque (refresh) | Stateless validation = no DB hit per request |
| API keys | Opaque, DB-stored | Long-lived, need revocability |
| JWT signing | EdDSA (Ed25519) | Asymmetric from day one — calce-ai and other services can verify tokens without holding the signing key. Faster and smaller than RSA. Avoids a key migration later if we add more verifiers |
| JWT expiry | 15 minutes | Short enough that TTL-based revocation is acceptable |
| Refresh token expiry | 30 days | Rolling — extended on use |
| Password hashing | Argon2id (19 MiB memory, 2 iterations, 1 parallelism) | OWASP current recommended minimums, pinned explicitly. Compatible across Rust (`argon2` crate) and Python (`argon2-cffi`) |
| Token/key hashing | HMAC-SHA256 (server-side secret) | Defense-in-depth: DB leak alone is not enough to verify tokens offline. Cheap upgrade over bare SHA-256 |
| Rate limiting | Per-IP token bucket (`governor`) + per-account lockout | Token bucket is more forgiving for legitimate bursts. Account lockout covers distributed attacks |
| Token transport | `Authorization: Bearer` header only | No browser SPA today — clients are services (calce-ai, customer integrations). No cookies = no CSRF. Revisit if a browser frontend is added |
| Instant JWT revocation | Deferred | TTL-based for now. Blacklist + pub/sub noted for future |

## Token Validation Flow (Unified)

Every authenticated request carries `Authorization: Bearer <token>`.

The middleware (single function in `calce-data::auth`) resolves it:

```
Bearer token received
  │
  ├─ Try JWT decode (signature + expiry check, no DB)
  │   ├─ Valid → extract SecurityContext from claims → done
  │   └─ Invalid signature / not a JWT → fall through
  │
  └─ Look up as API key (DB, with in-memory cache)
      ├─ Found + not expired → build SecurityContext → done
      └─ Not found → 401
```

This is the **only** entry point for auth validation. Routes never inspect
tokens directly.

### JWT Claims

```json
{
  "sub": "<user external_id>",
  "role": "user",
  "org": "<organization external_id or null>",
  "iat": 1711000000,
  "exp": 1711000900
}
```

Minimal claims. No sensitive data in the token.

### SecurityContext (unchanged interface)

```rust
pub struct SecurityContext {
    pub user_id: UserId,
    pub role: Role,
    // future: organization_id, grants, etc.
}
```

Routes continue using `SecurityContext` exactly as today. The only change is
how it's constructed (from JWT claims or API key lookup instead of headers).

## Database Schema

Three new tables, managed by calce-db (Alembic).

### user_credentials

Stores login credentials. Separate from `users` so the user table stays
clean and credential methods can evolve independently.

| Column | Type | Notes |
|--------|------|-------|
| id | bigint PK | |
| user_id | bigint FK users, unique | One credential per user (for now) |
| password_hash | varchar(255) | Argon2id PHC-format string |
| failed_attempts | int, default 0 | Consecutive failed logins |
| locked_until | timestamptz NULL | Account lockout expiry |
| created_at | timestamptz | |
| updated_at | timestamptz | |

### refresh_tokens

```
| Column          | Type              | Notes                              |
|-----------------|-------------------|------------------------------------|
| id              | bigint PK         |                                    |
| user_id         | bigint FK users   |                                    |
| family_id       | uuid              | Shared by all tokens from one login session |
| token_hash      | varchar(128)      | HMAC-SHA256 (keyed with server secret) |
| superseded_at   | timestamptz NULL  | Set when this token is rotated out |
| revoked_at      | timestamptz NULL  | Set on family revocation           |
| expires_at      | timestamptz       |                                    |
| created_at      | timestamptz       |                                    |
```

Token value is never stored in plaintext — only an HMAC-SHA256 hash
(keyed with a server-side secret). DB leak alone is not enough to forge
or verify tokens.

**Token states:**
- **Active**: `superseded_at IS NULL AND revoked_at IS NULL AND expires_at > now`
- **Grace**: `superseded_at IS NOT NULL AND now - superseded_at < 30s` — still
  accepted, returns the same new token pair (handles network drops)
- **Replay detected**: `superseded_at IS NOT NULL AND now - superseded_at >= 30s`
  — triggers family revocation: `UPDATE SET revoked_at = now WHERE family_id = X`

The `family_id` makes revocation a single query. No linked-list lineage
needed — `superseded_at` + grace window handles both rotation and replay.

Index: `(token_hash)` unique — lookup by hash on refresh.
Index: `(family_id)` — for family revocation queries.

### api_keys

```
| Column          | Type                    | Notes                            |
|-----------------|-------------------------|----------------------------------|
| id              | bigint PK               |                                  |
| organization_id | bigint FK organizations |                                  |
| name            | varchar(100)            | e.g. "production", "staging"     |
| key_prefix      | varchar(20)             | Structured prefix, e.g. `calce_live_` or `calce_test_` |
| key_hash        | varchar(128)            | HMAC-SHA256 of full key (keyed with server secret) |
| expires_at      | timestamptz NULL        | Optional expiry                  |
| created_at      | timestamptz             |                                  |
| revoked_at      | timestamptz NULL        | Soft-revoke without deleting     |
```

Full key format: `calce_live_<random>` (or `calce_test_` for test
environments). The structured prefix enables automatic secret scanning
(GitHub, GitGuardian), makes keys self-identifying in logs, and
distinguishes environments. The full key is returned only once at
creation. Stored as an HMAC-SHA256 hash.

Index: `(key_hash)` unique.
Index: `(organization_id)` for listing an org's keys.

## API Key Cache

In-memory cache in calce-data to avoid DB hits on every API key request:

- `moka` concurrent cache with built-in TTL eviction
- TTL: 60 seconds — short enough that revocation propagates quickly
- Cache miss → DB lookup → populate cache
- On key revocation → evict from cache (best-effort, TTL is the safety net)

Using `moka` rather than raw `DashMap` because it handles TTL expiry
internally — no need to build a background reaper.

Sufficient for single-instance. For multi-instance, add Redis or pub/sub
cache invalidation.

## API Endpoints

### Login

```
POST /auth/login
Body: { "email": "...", "password": "..." }
Response: {
  "access_token": "<JWT>",
  "refresh_token": "<opaque>",
  "token_type": "Bearer",
  "expires_in": 900
}
```

### Refresh

```
POST /auth/refresh
Body: { "refresh_token": "..." }
Response: {
  "access_token": "<JWT>",
  "refresh_token": "<new opaque>",
  "token_type": "Bearer",
  "expires_in": 900
}
```

Refresh token rotation: each refresh issues a new refresh token and
invalidates the old one. Detects token reuse (replay attack) — if a
used refresh token is presented, revoke the entire family.

**Grace period**: the old refresh token remains valid for 30 seconds after
rotation. This handles the case where the client sends a refresh request
but the network drops before it receives the new token pair — without the
grace period, the client would be locked out.

### API Key Management (admin-only)

```
POST   /organizations/{org_id}/api-keys    — create (returns full key once)
GET    /organizations/{org_id}/api-keys    — list (prefix + name only)
DELETE /organizations/{org_id}/api-keys/{id} — revoke
```

## Module Layout

All auth logic in `calce-data::auth`, split into submodules:

```
calce-data/src/auth/
├── mod.rs            — SecurityContext, Role (existing, moved here)
├── jwt.rs            — JWT encode/decode, claims
├── password.rs       — Argon2id hash + verify
├── api_key.rs        — API key generation, hashing, cache, validation
├── refresh_token.rs  — refresh token generation, hashing, rotation
└── middleware.rs      — unified token validation (the single entry point)

calce-data/src/permissions.rs  — unchanged
```

`calce-api/src/auth.rs` becomes thin: just the Axum `FromRequestParts`
extractor that calls `calce-data::auth::middleware::validate_token()`.

## Rate Limiting + Account Lockout

Two complementary layers, both in Phase 1:

### Per-IP rate limiting (auth endpoints only)

- **Method**: Token bucket via `governor` crate (integrates with Tower)
- **Limit**: 10 requests per minute per IP
- **Why token bucket**: more forgiving for legitimate bursts (user typos
  password twice, gets it right on third try) vs sliding window
- **Response**: 429 Too Many Requests with `Retry-After` header
- **Storage**: In-memory (governor's built-in keyed rate limiter)

Applied to `/auth/login` and `/auth/refresh` only. Not applied to normal
API requests (those are gated by valid auth).

### Per-account lockout

Covers distributed credential stuffing (different IPs, same account)
that per-IP limiting misses.

- Add `failed_attempts` (int, default 0) and `locked_until` (timestamptz,
  nullable) columns to `user_credentials`
- After 10 consecutive failed login attempts → lock account for 15 minutes
- Successful login → reset `failed_attempts` to 0
- Locked account returns 423 with `Retry-After`

## Migration Path

The transition from header-based to real auth:

1. **Build auth module** — JWT, password, API keys alongside existing code
2. **Add login/refresh endpoints** — new routes, existing routes unchanged
3. **Dual-mode middleware** — accepts both JWT/API key AND legacy headers.
   Environment variable `CALCE_AUTH_MODE=real|dev` controls which is active.
   In dev mode, headers still work for local development and testing.
4. **Switch over** — set `CALCE_AUTH_MODE=real`, remove header fallback
5. **Clean up** — remove dev mode, legacy header code

This means zero downtime migration and existing tests/tooling keep working
during development.

## Dev/Test Ergonomics

- `CALCE_AUTH_MODE=dev` preserves current X-User-Id/X-Role headers for
  local development and integration tests
- `invoke dev` sets dev mode automatically
- A seed task creates test users with known credentials
- API key creation via `invoke` task for local testing

## Implementation Phases

### Phase 1: Core Auth Module ✓
- [x] Restructure `calce-data::auth` into submodules
- [x] Implement `password.rs` (argon2id hash/verify, pinned OWASP params)
- [x] Implement `jwt.rs` (encode/decode with EdDSA/Ed25519)
- [x] Add `user_credentials` table with lockout columns (calce-db migration 0004)
- [x] Add `refresh_tokens` table (calce-db migration 0004)
- [x] Implement `refresh_token.rs` (generation, HMAC hashing, family tracking, rotation with 30s grace, replay detection)
- [x] Implement `middleware.rs` (unified validation, dual-mode)
- [x] Implement per-account lockout logic in login flow
- [x] Update Axum extractor to use new middleware
- [x] Add `POST /auth/login` and `POST /auth/refresh`
- [x] Tests: 11 new unit tests (password, JWT, tokens), all 86 workspace tests pass

### Phase 2: API Keys ✓
- [x] Add `api_keys` table (calce-db migration 0005)
- [x] Implement `api_key.rs` (generation, `calce_live_`/`calce_test_` prefix, HMAC hashing, moka cache)
- [x] Wire API key validation into unified middleware (JWT → cache → DB fallback)
- [x] Add API key CRUD endpoints (admin-only): POST/GET/DELETE /organizations/{org_id}/api-keys
- [x] Tests: 8 new unit tests (key generation, validation, cache ops), 100 total pass

### Phase 3: Rate Limiting ✓
- [x] Implement per-IP token bucket rate limiter (`governor`, 10 req/min)
- [x] Apply to `/auth/login` and `/auth/refresh`
- [x] `ApiError::RateLimited` → 429 with error body

### Phase 4: Cutover ✓
- [x] Seed script creates credentials for all users (password: "password", first user: admin)
- [x] `docs/auth.md` fully rewritten to match implementation
- [x] Legacy `X-User-Id`/`X-Role` header auth removed
- [x] `AuthMode` / `CALCE_AUTH_MODE` removed — always real auth
- [x] Tests updated to use JWT Bearer tokens
- [x] `.env` populated with `CALCE_JWT_PRIVATE_KEY` and `CALCE_HMAC_SECRET`
- [x] `CLAUDE.md` files updated (calce-api, calce-data)

## Rust Dependencies

| Crate | Purpose |
|-------|---------|
| `jsonwebtoken` | JWT encode/decode (supports EdDSA) |
| `argon2` | Password hashing (argon2id variant) |
| `moka` | Concurrent cache with TTL eviction for API keys |
| `governor` | Token bucket rate limiter (Tower integration) |
| `hmac` + `sha2` | HMAC-SHA256 for token/key hashing |
| `rand` + `base64` | Secure token generation |
| `ed25519-dalek` | Ed25519 key generation (used with jsonwebtoken) |

## Future Work (Not In Scope)

Tracked separately, not part of this plan:

- **JWT blacklist / instant revocation** — pub/sub or shared cache for
  revoking JWTs before expiry. Needed if 15-min TTL is too long for
  compliance. → backlog
- **OAuth / social login** — Google, Apple, etc. The auth module is
  designed to support additional authentication methods without changing
  the token/session layer.
- **Email verification + password reset** — one-time codes, email sending.
- **Advisor role** — per-client access grants (already noted in auth.md).
- **Multi-instance rate limiting** — Redis-backed when we scale beyond
  one instance.

## Reference

- njorda auth implementation: `../njorda/services/api/src/njorda/libs/auth.py`
- njorda token model: `../njorda/services/api/src/njorda/models/auth_token/model.py`
- OWASP password storage cheat sheet: argon2id recommended
- Current calce auth: `crates/calce-data/src/auth.rs`, `crates/calce-api/src/auth.rs`
