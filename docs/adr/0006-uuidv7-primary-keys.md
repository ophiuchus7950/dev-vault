# ADR-0006: UUIDv7 primary keys

- **Status:** Accepted
- **Date:** 2026-11-04
- **Related:** [DB Schema §identifiers](../03-database-schema.md#identifiers) · [ADR-0004](./0004-cursor-pagination.md)

## Context

Every table needs a primary key. Options are auto-increment integers (`BIGSERIAL`/identity), random UUIDs (v4), or time-sortable UUIDs (v7). The choice affects cursor pagination, index behaviour, import/merge semantics, and URL safety.

## Decision

Use **UUIDv7** identifiers for all primary keys.

Rationale:
- **Time-sortable:** the leading 48-bit millisecond timestamp makes v7 UUIDs lexicographically increasing, so B-tree inserts stay near the right edge (avoiding the page-split churn of random v4) and `(created_at, id)` ordering is natural.
- **Non-enumerable:** unlike `BIGSERIAL`, IDs in URLs/exports don't reveal row counts or allow scanning neighbours.
- **Portable & merge-friendly:** IDs are globally unique and stable, so import/export round-trips and merging vaults never require remapping keys.
- **Cursor-friendly:** a unique, ordered id is the perfect tiebreaker for keyset pagination ([ADR-0004](./0004-cursor-pagination.md)).

Generation: on PostgreSQL 18 use the built-in `uuidv7()`; on PostgreSQL 16 (our target) generate in the application (`uuid::Uuid::now_v7()`) or via a `uuidv7()` helper/extension. The API always supplies the id on insert.

## Consequences

**Positive**
- Stable, sortable, non-guessable identifiers end to end.
- No import-collision remapping; safe to expose in URLs and export files.
- Good index locality (monotonic-ish inserts).

**Negative**
- 16 bytes vs 8, so slightly larger PKs and FKs and marginally larger indexes — negligible at single-user scale.
- Requires a UUID extension or app-side generation on PG16; adds a dependency on PG18 semantics being emulated correctly.

## Alternatives considered

| Option | Why not |
| ------ | ------- |
| **BIGSERIAL / identity** | Smallest and fastest, but enumerable/guessable, and import/merge requires id remapping — bad for a portable vault. |
| **UUIDv4** | Non-enumerable and portable, but random insert order causes index bloat and needs a separate sort column; strictly worse than v7 here. |
| **ULID / KSUID (text)** | Also sortable and portable, but non-standard in Postgres tooling, larger as text, and less ergonomic with `sqlx`'s `Uuid` mapping. |
