# Live-Update UI Pattern

The console receives real-time entity change notifications from the backend and automatically refreshes affected queries. **All pages displaying mutable data must use this pattern.**

## Architecture (one sentence)

Postgres CDC detects row changes → backend PubSub coalesces and fans out → SSE pushes `{"table","id"}` events to the browser → `useEntityEvents` invalidates matching TanStack Query caches → React re-renders with fresh data.

See `docs/cdc.md` for the backend/database side.

## How to wire up a page

### 1. Use table-name query keys

TanStack Query keys must start with the **plural table name** so the invalidation logic can match them:

```ts
// List query — key starts with plural table name
useQuery({ queryKey: ['instruments', { page, search }], queryFn: ... })

// Detail query — key starts with singular form + id
useQuery({ queryKey: ['instrument', id], queryFn: ... })
```

The invalidation hook matches on prefix: when an `instruments` CDC event arrives it invalidates all `['instruments', ...]` queries and also `['instrument', <id>]`.

### 2. Call `useEntityEvents` in the page component

```ts
import { useEntityEvents } from '../hooks/useEntityEvents'

export default function InstrumentsPage() {
  useEntityEvents(['instruments'])
  // ... queries, table, etc.
}
```

Pass the table names this page cares about. Pass `[]` to react to all tables.

That's it — no polling, no manual refetch, no WebSocket plumbing.

## Query key naming convention

| Query type | Key shape | Example |
|------------|-----------|---------|
| List / paginated | `[tablePlural, filterParams]` | `['users', { page: 1, search: '' }]` |
| Single entity | `[tableSingular, id]` | `['user', 'abc-123']` |

The `useEntityEvents` hook strips the trailing `s` to derive the singular form, so stick to this convention.

## Mutation + CDC dual invalidation

When a page mutates data (e.g. editing a user), invalidate manually in `onSuccess` **and** let CDC handle external changes:

```ts
const mutation = useMutation({
  mutationFn: (data) => api.updateUser(id, data),
  onSuccess: () => {
    queryClient.invalidateQueries({ queryKey: ['user', id] })
    queryClient.invalidateQueries({ queryKey: ['users'] })
  },
})

// CDC covers changes made by other users/systems
useEntityEvents(['users'])
```

The manual invalidation gives instant feedback; CDC catches everything else.

## Constraints and gotchas

- **Admin-only**: SSE events are only sent to admin users (checked in `useEntityEvents`). Non-admin pages that need live data will require a separate mechanism.
- **Monitored tables**: Only tables configured in the CDC listener emit events. If you add a new table, wire it into `crates/calce-data/src/cdc.rs`.
- **Signal-only**: Events carry the table name and row ID but no payload. The frontend always refetches — it never patches local state from the event.
- **Coalescing**: The backend deduplicates rapid events for the same key within a 100ms window, so bursts of writes produce a single refetch.

## Reference implementation

- `src/hooks/useEventSource.ts` — SSE connection with auto-reconnect
- `src/hooks/useEntityEvents.ts` — TanStack Query invalidation bridge
- `src/pages/UsersPage.tsx` — list page with live updates
- `src/pages/UserDetailPage.tsx` — detail page with edit + live updates
