# ADR-0001: Use Rust + Axum for the backend

- **Status:** Accepted
- **Date:** 2026-11-04
- **Related:** [Architecture §3](../01-architecture.md#3-technology-decisions-at-a-glance)

## Context

DevVault needs a small, fast, type-safe HTTP API over PostgreSQL. The brief specifies a Rust REST API. We must pick a framework and commit to the surrounding ecosystem (Tokio, middleware, extractors, testing).

## Decision

Use **Axum** on the Tokio runtime.

Rationale:
- Axum is built on `tower`/`tower-http`, so middleware (tracing, CORS, compression, timeouts) is composed from battle-tested, reusable layers.
- Type-safe extractors (`State`, `Path`, `Query`, `Json`) map naturally onto a thin handler → service → repository layering.
- Strong ecosystem momentum and the most current tutorials/examples, which matters for a portfolio-quality project.
- Clean integration with `sqlx`, `tracing`, and `utoipa`.

## Consequences

**Positive**
- Idiomatic async Rust with minimal framework magic.
- Reusable middleware; easy structured logging via `TraceLayer`.
- Simple to test handlers with `axum::Router` + `tower::ServiceExt`.

**Negative**
- Middleware composition requires understanding `tower` service bounds (`Service`, `Layer`), which has a learning curve.
- Fewer batteries than Rocket (we assemble more ourselves).

## Alternatives considered

| Option | Why not |
| ------ | ------- |
| **Actix-web** | Faster in raw benchmarks and very mature, but its actor model and historical `unsafe`/ecosystem baggage add concepts without benefit at this scale; weaker `tower` interop. |
| **Rocket** | Excellent DX and request guards, but slower release cadence and smaller ecosystem momentum in 2025. |
| **Warp** | Composable filters are elegant but less ergonomic for a CRUD API and declining mindshare. |
