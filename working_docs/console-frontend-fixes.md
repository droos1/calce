# Console Frontend Fixes

Tracking fixes from the code review of `services/calce-console/`.

## High Severity

### 1. UserDetailPage fetches ALL users to find one by ID
- **Problem:** `UserDetailPage` fetches up to 1000 users via `api.getUsers()` and filters client-side with `.find()`. Doesn't scale.
- **Fix:**
  - Add `get_user(id)` method to `UserDataStore` (backend)
  - Add `GET /v1/data/users/{user_id}` route in `calc.rs` (backend)
  - Add `api.getUser(id)` to `client.ts` (frontend)
  - Rewrite `UserDetailPage` to use the new endpoint
- **Status:** [x] Done

### 2. Query key collision in UserDetailPage
- **Problem:** Uses `queryKey: ['users', { page: 1, search: '', pageSize: 1000 }]` which pollutes the UsersPage cache.
- **Fix:** Change to `['user', id]` with dedicated endpoint.
- **Status:** [x] Done (fixed as part of #1)

## Medium Severity

### 3. ThemeToggle duplication
- **Problem:** `Sidebar.tsx` has inline theme logic. `ThemeToggle.tsx` component exists but is unused.
- **Fix:** Use `ThemeToggle` in Sidebar, delete duplicate logic.
- **Status:** [x] Done

### 4. No error states on data pages
- **Problem:** API failures leave blank pages — `useQuery` errors are ignored.
- **Fix:** Add error UI to all data-fetching pages.
- **Status:** [x] Done

### 5. Debounced search + pagination duplicated
- **Problem:** `UsersPage` and `InstrumentsPage` implement identical 300ms debounce + pagination state.
- **Fix:** Extract `usePaginatedSearch()` hook.
- **Status:** [x] Done

## Low Severity

### 6. Hardcoded page sizes
- **Problem:** `PAGE_SIZE = 30` in two pages, `USERS_PAGE_SIZE = 1000` in another.
- **Fix:** Centralize as shared constants.
- **Status:** [x] Done

### 7. Hardcoded colors in CSS
- **Problem:** `components.css` has `#c12a1f` (danger hover) and `#ffffff` (toggle knob) instead of tokens.
- **Fix:** Add `--color-error-hover` token, use `--color-text-on-primary` for toggle.
- **Status:** [x] Done
