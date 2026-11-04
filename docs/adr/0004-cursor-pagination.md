# ADR-0004: Cursor/keyset pagination

- **Status:** Accepted
- **Date:** 2026-11-04
- **Related:** [API Spec §5](../04-api-specification.md#5-pagination)

## Context

List and search endpoints return potentially large sets of entries. We must choose how clients page through them: offset-based (`?page=&per_page=`) or cursor/keyset-based (`?limit=&cursor=`). The PRD locks this decision for v1.

## Decision

Use **cursor/keyset pagination** ordered by `(created_at DESC, id DESC)` (configurable to `updated_at`), returning an opaque `next_cursor`.

Rationale:
- **Stability:** offset pagination skips or duplicates rows when items are inserted/deleted mid-scroll — a real problem for a live-editable library. Keyset pagination is immune.
- **Performance:** `WHERE (created_at, id) < ($1, $2)` uses the index and stays O(log n) at any depth, unlike `OFFSET 10000`.
- **UI fit:** TanStack Query's `useInfiniteQuery` maps directly onto cursor pagination for infinite scroll / "load more".
- UUIDv7 gives a naturally sortable, tie-breaking id.

## Consequences

**Positive**
- Consistent results under concurrent writes.
- Constant-time page fetches regardless of depth.
- Clean client caching with TanStack Query.

**Negative**
- **No total count and no "jump to page N"** out of the box. If a total count is ever needed for display, it requires a separate `COUNT(*)` query (cheap enough for a single-user 10k vault, but not free).
- The cursor must be opaque (base64-encoded `(created_at, id)`) so it can evolve without breaking clients.
- Sorting must always include a unique tiebreaker (`id`) to keep the keyset total.

## Alternatives considered

| Option | Why not |
| ------ | ------- |
| **Offset/limit** | Simplest, supports page numbers and totals, but degrades at depth and shifts under writes. Acceptable for tiny static sets only. |
| **Both modes** | Maximum flexibility, but doubles the code paths and test surface for marginal benefit. |
