# ADR-0002: PostgreSQL + SQLx with raw, compile-time-checked SQL

- **Status:** Accepted
- **Date:** 2026-11-04
- **Related:** [Architecture §7](../01-architecture.md#7-search-architecture) · [DB Schema](../03-database-schema.md)

## Context

DevVault's headline technical feature is **PostgreSQL full-text search combined with fuzzy matching** (`tsvector`, GIN indexes, `pg_trgm`, generated columns, partial indexes, `ts_rank_cd`). We need a database layer that (a) exposes Postgres's full feature set rather than hiding it, and (b) gives type safety.

The brief explicitly names PostgreSQL full-text search, so an external search engine is a fallback, not the default.

## Decision

Use **PostgreSQL 16** with **SQLx**, writing hand-authored SQL verified at compile time via `sqlx::query_as!` / `query!`.

Rationale:
- Compile-time verification catches column/type drift the moment the schema changes — ideal for a schema we own and evolve.
- Raw SQL keeps advanced features (FTS, trigram, window functions, `ON CONFLICT`, partial indexes) first-class instead of fighting an ORM's DSL.
- Built-in migrations (`sqlx migrate`) keep schema changes versioned in plain SQL.
- Demonstrates genuine database design, which is part of the project's purpose.

## Consequences

**Positive**
- No ORM abstraction tax; the queries are the documentation.
- Search/ranking/filtering logic is optimisable and explainable (`EXPLAIN ANALYZE`) without escape hatches.
- Migrations are plain, reviewable SQL files.

**Negative**
- `sqlx` macros require a live database at compile time (or checked-in offline metadata via `cargo sqlx prepare`, `SQLX_OFFLINE=true`).
- More SQL written by hand than with an ORM; repetition in list/filter queries must be factored carefully (query builders or macros).
- On-the-fly schema changes require recompiling with the DB present.

## Alternatives considered

| Option | Why not |
| ------ | ------- |
| **Diesel** | Excellent compile-time safety, but sync-first (needs `spawn_blocking`), a heavy DSL, and awkward for FTS/trigram (requires `sql_query`). |
| **SeaORM** | Ergonomic async CRUD over SQLx, but its entity abstraction obscures the advanced SQL that is the point of the project. |
| **External search (Meilisearch/Typesense)** | Best relevance/typo UX, but adds a service to run, sync, and deploy — moves away from the "Postgres FTS" highlight. Kept as a documented future option. |
