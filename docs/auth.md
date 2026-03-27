# Authentication & Authorization

## Principle

Every route requires authentication. The only exceptions are health checks
and similar infrastructure endpoints. Even instrument-scoped calculations
(e.g. volatility) that aren't tied to a specific user still require the
caller to be an authenticated user of the system.

## Three Levels of Authorization

1. **Authenticated** — the caller is a valid user. Required for all
   calculation and data endpoints, including instrument-scoped ones.

2. **Admin-only** — the caller must be an admin. Used for user
   management (create, list, delete), organization endpoints, and API key
   management.

3. **User-scoped** — the caller is accessing a specific user's data
   (portfolios, trades, market value). Requires authentication *plus*
   an access check: can this user see that user's data?


## User Model

Every authenticated request produces a `SecurityContext` (defined in `calce-data::auth`) containing:

- **user_id** — the authenticated user
- **role** — currently `User` or `Admin`

Route handlers enforce access checks via helper functions in `calce-api/src/auth.rs`:

- `require_admin(ctx)` — returns 403 unless `ctx.role == Admin`
- `require_access(ctx, target_user_id)` — returns 403 unless
  `ctx.can_access(target)` (delegates to `calce-data::permissions`)

calce-core has no auth types — it is a pure calculation engine.

### Access Rules (User-Scoped)

A user can access their own data. An advisor or admin can access other users'
data. The check is `SecurityContext::can_access(target_user_id)`:

- `Role::User` — can only access data where `target == self`
- `Role::Admin` — can access any user's data

This will evolve to support an advisor model where a user may be granted
access to specific clients, but the core pattern stays the same:
authenticate → build `SecurityContext` → pass it through to data layer →
data layer enforces access.

## Authentication Methods

Two authentication methods, both producing the same `SecurityContext`:

### 1. User Login (JWT)

Users authenticate with email + password via `POST /auth/login`. The server
returns a short-lived JWT access token (EdDSA/Ed25519, 15-minute expiry) and
an opaque refresh token (30-day rolling expiry).

- **Access token**: Stateless validation — signature + expiry check, no DB hit.
  Carried in `Authorization: Bearer <jwt>` header.
- **Refresh token**: Stored as HMAC-SHA256 hash (server-keyed). Rotation on
  each use with 30-second grace period and family-based replay detection.
- **Password hashing**: Argon2id with OWASP-recommended parameters (19 MiB,
  2 iterations, 1 parallelism).

### 2. API Keys (Service Consumers)

Organizations get long-lived API keys for service-to-service integration.
Format: `calce_live_<random>` (or `calce_test_` for test environments).

- Stored as HMAC-SHA256 hash — full key returned once at creation.
- Looked up via in-memory cache (moka, 60s TTL) → DB fallback.
- Managed via org-admin CRUD: `POST/GET/DELETE /organizations/{org_id}/api-keys`.
- **Org-scoped**: API keys carry an `org_id` on their `SecurityContext`. The
  permissions layer denies cross-org user-data access by default; route handlers
  must explicitly verify org membership for user-scoped routes.

### Unified Validation Flow

Single middleware entry point (`calce-data::auth::middleware::validate_bearer_token`):

```
Bearer token received
  │
  ├─ Try JWT decode (signature + expiry, no DB)
  │   ├─ Valid → SecurityContext → done
  │   └─ Not a JWT → fall through
  │
  └─ Look up as API key (cache → DB)
      ├─ Found + valid → SecurityContext → done
      └─ Not found → 401
```

## Security Measures

### Account Lockout

10 consecutive failed login attempts → account locked for 15 minutes.
Tracked via `failed_attempts` and `locked_until` on `user_credentials`.
Covers distributed credential stuffing that per-IP limiting misses.

### Rate Limiting

Per-IP token bucket (governor, 10 req/min) on `/auth/login`, `/auth/refresh`,
and `/auth/logout`. Returns 429 Too Many Requests with `Retry-After` header.
IP extraction uses GCP-aware X-Forwarded-For parsing (skips trusted proxy hops).

### Input Validation

- Login email must be valid format (contains `@` with non-empty parts).
- Password capped at 128 characters to prevent Argon2 resource exhaustion.
- Timing-safe login: a dummy hash comparison runs when the email doesn't exist,
  so response time doesn't reveal valid accounts.

### Refresh Token Rotation

Each refresh invalidates the old token and issues a new one. Within the
30-second grace period, the old token returns a fresh access token but does
not mint a new refresh token (prevents token proliferation). Reuse after the
grace period triggers family-wide revocation.

### Logout

`POST /auth/logout` with `{ "refresh_token": "..." }` revokes the entire
token family. The JWT access token remains valid until its 15-minute expiry.

## Configuration

Required environment variables:

- **`CALCE_JWT_PRIVATE_KEY`** — base64-encoded Ed25519 PKCS#8 v2 DER private key
  (must be generated by `ring` — standard OpenSSL v1 keys are not compatible)
- **`CALCE_HMAC_SECRET`** — server-side secret for token/key HMAC hashing

## Module Layout

```
calce-data/src/auth/
├── mod.rs            — SecurityContext, Role, AuthConfig, AuthMode
├── jwt.rs            — EdDSA JWT encode/decode
├── password.rs       — Argon2id hash + verify
├── tokens.rs         — secure token generation, HMAC-SHA256 hashing
├── api_key.rs        — API key generation, validation, moka cache
└── middleware.rs     — unified token validation

calce-data/src/queries/auth.rs  — SQL for credentials, refresh tokens, API keys
calce-data/src/permissions.rs   — access-control rules (unchanged)
calce-api/src/auth.rs           — Axum extractor (thin, calls middleware)
calce-api/src/routes/auth.rs    — POST /auth/login, POST /auth/refresh, POST /auth/logout
calce-api/src/routes/api_keys.rs — API key CRUD
calce-api/src/rate_limit.rs     — governor rate limiter
```

## Database Tables

- `users.role` — `"user"` or `"admin"` (default: `"user"`)
- `user_credentials` — password hash, failed_attempts, locked_until (1:1 with users)
- `refresh_tokens` — family_id, token_hash, superseded_at, revoked_at, expires_at
- `api_keys` — organization_id, name, key_prefix, key_hash, expires_at, revoked_at

## What's Coming

- JWT blacklist / instant revocation via pub/sub (currently TTL-based, 15-min max)
- OAuth / social login (Google, Apple) — auth module designed for extensibility
- Email verification + password reset
- Advisor role: per-client access grants
