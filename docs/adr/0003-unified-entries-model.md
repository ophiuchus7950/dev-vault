# ADR-0003: Unified `entries` table with a `kind` enum

- **Status:** Accepted
- **Date:** 2026-11-04
- **Related:** [DB Schema §entries](../03-database-schema.md#entries)

## Context

DevVault stores five related content types — snippets, commands, notes, links and docs. They share most attributes (title, body, tags, collections, pinned/archived, timestamps) and differ only in a few kind-specific fields. We must decide between one polymorphic table, one table per type, or a base table with extension tables.

## Decision

Model all content as **one `entries` table** with a `kind` enum column and a small number of nullable kind-specific columns (`language`, `shell`, `url`).

Rationale:
- **One search index, one ranking function, one set of filters** across all content — search is the core feature, so unifying it is the biggest win.
- **One CRUD surface**: `POST/GET/PATCH/DELETE /entries` with `?kind=` filtering, rather than five resource families.
- Adding a new kind (e.g. `doc`) is a one-line enum change, not a new table, endpoints, and union query.
- Cross-kind views ("everything I saved this week", "everything tagged `postgres`") are trivial.

## Consequences

**Positive**
- Minimal API and frontend surface; uniform pagination, tagging, collections, export.
- Simpler migrations and fewer joins on reads.
- Trivial to add filters/sorts consistently.

**Negative**
- Kind-specific columns are nullable, so the schema alone doesn't enforce "snippets must have a language". This is enforced in the **service/validation layer** (DTO rules) instead.
- A row can hold fields irrelevant to its kind; discipline is required in validation and UI.

## Alternatives considered

| Option | Why not |
| ------ | ------- |
| **Separate tables per kind** | Tight per-kind constraints, but four+ search indexes, four+ endpoint families, `UNION ALL` for cross-kind listings, and more migrations — heavy for a single-user tool. |
| **Base table + per-kind detail tables** | Fully normalized with no nullable columns, but every read joins and the complexity is unjustified at this scale. |
| **JSONB `data` column for kind specifics** | Flexible but untyped and hard to index/validate; loses the strong typing we want. |
