# DevVault — Technical Architecture

- **Status:** Draft v1.0
- **Last updated:** 2026-11-04
- **Related:** [PRD](./00-prd.md) · [DB Schema](./03-database-schema.md) · [API Spec](./04-api-specification.md) · [ADRs](./adr/)

---

## 1. Design principles

1. **Local-first, single-user.** No accounts, no network calls to third parties. The API binds to localhost by default. See [ADR-0005](./adr/0005-no-authentication-local-first.md).
2. **Postgres does the heavy lifting.** Search, ranking, filtering, pagination and integrity live in the database (FTS + `pg_trgm` + partial indexes), not in application code.
3. **One source of truth.** Entry data is defined once in SQL; the API is a thin, typed layer over it.
4. **Boring, explicit layers.** Handlers → services → repositories. No ORM magic; raw, compile-time-checked SQL.
5. **Portable by construction.** UUIDv7 IDs, RFC3339 UTC timestamps, lossless JSON export.
6. **Typed end to end.** Rust types ↔ OpenAPI schema ↔ TypeScript types generated from the same source.

## 2. System context

```mermaid
flowchart LR
    Dev([Developer])
    CLI[devvault CLI]
    Browser[Browser SPA]

    subgraph Host["Your machine / home server (Docker Compose)"]
        API[api<br/>Rust · Axum · :3000 loopback]
        DB[(postgres:16<br/>+ pg_trgm)]
    end

    Dev --> Browser
    Dev --> CLI
    Browser -->|HTTP /api/v1| API
    CLI -->|HTTP /api/v1| API
    API -->|SQLx pool| DB
```

**Trust boundary:** the entire stack is assumed to run on a trusted host. There is no auth layer, so the API **MUST** be bound to loopback (or a private network) and never exposed directly to the internet. The frontend is deployed separately and talks to the API over HTTP/CORS.

## 3. Technology decisions at a glance

| Concern | Choice | ADR / notes |
| ------- | ------ | ----------- |
| Web framework | Axum (Tokio/Tower) | [ADR-0001](./adr/0001-rust-axum-backend.md) |
| Database access | SQLx, raw SQL, compile-time checked | [ADR-0002](./adr/0002-postgres-sqlx.md) |
| Data model | Unified `entries` + `kind` enum | [ADR-0003](./adr/0003-unified-entries-model.md) |
| Search | `tsvector` + GIN, `pg_trgm` fallback | §7 |
| Pagination | Cursor/keyset on `(created_at, id)` | [ADR-0004](./adr/0004-cursor-pagination.md) |
| Primary keys | UUIDv7 | [ADR-0006](./adr/0006-uuidv7-primary-keys.md) |
| Auth | None (local-first) | [ADR-0005](./adr/0005-no-authentication-local-first.md) |
| API docs | utoipa derive → OpenAPI + Swagger UI | §6 |
| Frontend | Vite + React + TS, TanStack Query + Zustand | §8 |
| Packaging | Docker Compose | §9 |

## 4. Repository layout

```
dev-vault/
├── backend/
│   ├── migrations/               # sqlx migrations (timestamped SQL)
│   ├── src/
│   │   ├── main.rs               # bootstrap: config, tracing, pool, router, serve
│   │   ├── config.rs             # typed Config from env
│   │   ├── state.rs              # AppState { pool, config }
│   │   ├── error.rs              # AppError (thiserror) + IntoResponse
│   │   ├── telemetry.rs          # tracing subscriber setup
│   │   ├── router.rs             # route table + layers
│   │   ├── openapi.rs            # utoipa #[derive(OpenApi)]
│   │   ├── domain/               # pure types & enums (Entry, Kind, ...)
│   │   │   ├── entry.rs
│   │   │   ├── tag.rs
│   │   │   └── collection.rs
│   │   ├── dto/                  # request/response shapes + validation
│   │   │   ├── entry.rs
│   │   │   ├── search.rs
│   │   │   └── pagination.rs
│   │   ├── repo/                 # SQL lives here (query_as!)
│   │   │   ├── entries.rs
│   │   │   ├── tags.rs
│   │   │   ├── collections.rs
│   │   │   └── search.rs
│   │   ├── service/              # business rules, orchestration
│   │   │   ├── entries.rs
│   │   │   ├── tagging.rs
│   │   │   └── portability.rs    # import/export
│   │   └── routes/               # Axum handlers + utoipa paths
│   │       ├── entries.rs
│   │       ├── tags.rs
│   │       ├── collections.rs
│   │       ├── search.rs
│   │       ├── portability.rs
│   │       └── health.rs
│   └── tests/                    # integration tests (ephemeral Postgres)
├── frontend/                     # Vite + React SPA (see frontend-build-prompt.md)
├── cli/                          # `devvault` terminal client
├── docker-compose.yml            # db + api stack (repo root)
├── infra/                        # nginx.conf and other deploy assets (optional)
├── docs/                         # this documentation set
└── README.md
```

## 5. Backend architecture

### 5.1 Layering

```mermaid
flowchart TD
    H[Handlers<br/>routes/*.rs] -->|DTO| S[Services<br/>service/*.rs]
    S -->|domain types| R[Repositories<br/>repo/*.rs]
    R -->|sqlx query_as!| DB[(PostgreSQL)]
    H -->|IntoResponse| E[AppError]
    S -->|?| E
    R -->|sqlx::Error| E
```

- **Handlers** parse/validate input (extractors + `validator`), call a service, and serialise output. No SQL here.
- **Services** own business rules: e.g. "renaming a tag must not create duplicates", "purge requires the entry to already be in Trash", "import dedupes by id". Services return domain types.
- **Repositories** own every SQL string. Only `sqlx::query_as!` / `query!` macros — compile-time checked against a real schema.
- **Domain** types are plain structs/enums with no framework coupling.

### 5.2 Request lifecycle

```mermaid
sequenceDiagram
    participant C as Client
    participant T as tower-http (TraceLayer)
    participant R as Router
    participant H as Handler
    participant S as Service
    participant P as Postgres
    C->>T: HTTP request
    T->>R: trace span start
    R->>H: route match + extractors
    H->>H: validate DTO (validator)
    H->>S: call service
    S->>P: sqlx query (compile-checked)
    P-->>S: rows
    S-->>H: domain type
    H-->>C: JSON + status (+ ETag/Location)
```

### 5.3 Error handling

A single `AppError` enum (`thiserror`) maps to HTTP responses via `IntoResponse`, producing the standard envelope (see [API spec §2](./04-api-specification.md#2-error-model)):

```rust
pub enum AppError {
    NotFound { resource: &'static str },
    Validation(ValidationErrors),
    Conflict { reason: String },      // 409 (stale ETag, duplicate)
    UnprocessableEntity(String),      // 422 (bad cursor, bad filter)
    Internal(anyhow::Error),          // 500, logged with span context
}
```

Rules:
- Repositories never construct HTTP types; they return `sqlx::Error`.
- `From<sqlx::Error> for AppError` distinguishes "no rows" (404) from real failures (500).
- 5xx responses never leak internal detail to the body; the detail goes to `tracing` logs with a correlation id.

### 5.4 Configuration

All config via environment (`.env` in dev). Typed, validated at startup — the process refuses to boot with an invalid config.

| Variable | Default | Purpose |
| -------- | ------- | ------- |
| `DATABASE_URL` | — | `postgres://user:pass@host:5432/devvault` |
| `BIND_ADDR` | `127.0.0.1:3000` | API bind address (loopback by default) |
| `RUST_LOG` | `info,devvault=debug` | tracing filter |
| `CORS_ALLOWED_ORIGINS` | `http://localhost:5173,http://localhost:8080` | dev origins |
| `MAX_PAGE_SIZE` | `100` | hard cap on `limit` |
| `DEFAULT_PAGE_SIZE` | `20` | default `limit` |
| `SEARCH_MIN_TRGM_SCORE` | `0.3` | minimum similarity for trigram fallback |
| `LOG_FORMAT` | `pretty` | `pretty` (dev) or `json` (prod) |

### 5.5 Observability

- `tracing` + `tracing-subscriber`; `TraceLayer` emits one span per request with method, path, status, latency.
- Dev: human-readable (`pretty`). Prod: JSON lines for log shippers.
- `/health` (liveness) and `/ready` (DB ping + migration version) endpoints for Compose healthchecks.

## 6. API layer

- Base path `/api/v1`; versioned so breaking changes ship as `/api/v2` without breaking clients.
- **utoipa** derives generate an OpenAPI 3.1 document from handlers and DTOs; served at `/api-docs/openapi.json` and rendered with **Swagger UI** at `/docs`. The spec can never silently drift from the code.
- Strict REST conventions: JSON `snake_case`, RFC3339 UTC timestamps, `201 Created` + `Location` on create, `204 No Content` on delete, `ETag`/`If-Match` for optimistic concurrency.

Full details in [04-api-specification.md](./04-api-specification.md).

## 7. Search architecture

Search is **Postgres-native** — no external search service ([ADR-0002](./adr/0002-postgres-sqlx.md)).

### 7.1 Indexed artefacts

| Artefact | Type | Purpose |
| -------- | ---- | ------- |
| `entries.search` | `tsvector`, **generated stored** column | Weighted full-text index: title(A) + language/tags(B) + body(C) |
| `idx_entries_search` | GIN on `search` | Fast FTS match + rank |
| `idx_entries_title_trgm` | GIN `gin_trgm_ops` on `title` | Fuzzy/partial title matching |
| `idx_entries_body_trgm` | GIN `gin_trgm_ops` on `body` | Fuzzy body matching (identifier typos) |

### 7.2 Query strategy (`mode=hybrid`, default)

```sql
-- FTS branch: exact stemming + ranking
SELECT ..., ts_rank_cd(search, websearch_to_tsquery('english', $1)) AS rank
FROM entries
WHERE search @@ websearch_to_tsquery('english', $1)
  AND deleted_at IS NULL
ORDER BY rank DESC, created_at DESC, id DESC;

-- Trigram fallback (when FTS returns few/no rows, or user asks for fuzzy)
SELECT ..., similarity(title, $1) AS sim
FROM entries
WHERE (title % $1 OR body % $1)
  AND deleted_at IS NULL
ORDER BY sim DESC, created_at DESC, id DESC;
```

- `mode=fts` → FTS only. `mode=trgm` → trigram only. `mode=hybrid` (default) → run FTS; if it yields fewer than a threshold of rows, blend in trigram results ranked by `similarity()`.
- Normalisation via `unaccent` keeps accented text searchable.
- Filters (kind/tag/collection/pinned/archived) apply identically to the list and search endpoints, so search is just "list + a query predicate".

> Exact DDL lives in [03-database-schema.md](./03-database-schema.md#search-artefacts).

## 8. Frontend architecture

Full build instructions live in [frontend-build-prompt.md](./frontend-build-prompt.md); this is the shape.

- **Vite + React + TypeScript** single-page app; no SSR (it's a private, logged-out tool with no SEO needs).
- **Routing:** React Router; routes for list, detail, create/edit, collections, tags, trash, settings, search.
- **Server state:** **TanStack Query** — all API interaction goes through query/mutation hooks. `useInfiniteQuery` maps directly onto cursor pagination. Optimistic updates for pin/tag/archive/delete.
- **Client state:** **Zustand** — theme, sidebar, active filters, command-palette open state. Kept small; anything from the server lives in Query's cache.
- **API client:** a thin typed `fetch` wrapper that injects the base URL, serialises query params, unwraps the error envelope, and (optionally) carries `If-Match` for updates. Types are generated from the OpenAPI document so the client can't drift.
- **Editor/render:** CodeMirror 6 for input; Shiki for read-only highlighted rendering; react-markdown (+ remark-gfm + rehype-sanitize) for notes.
- **Styling:** Tailwind + shadcn/ui (Radix) — utility CSS, accessible primitives, easy theming and dark mode.

```mermaid
flowchart LR
    UI[React components] --> Q[TanStack Query]
    Q --> AC[API client /api/v1]
    AC --> API[(Rust API)]
    UI --> Z[Zustand UI store]
    UI --> CM[CodeMirror / Shiki / react-markdown]
```

## 9. Deployment topology

```mermaid
flowchart TB
    subgraph compose["docker compose"]
        direction TB
        APIS["api (Rust) — publishes 127.0.0.1:3000"]
        PGS["postgres:16\nvolume: pgdata\nhealthcheck: pg_isready"]
        APIS -->|5432| PGS
    end
    User([localhost:3000]) --> APIS
    SPA["frontend (dev server / static host)"] -.->|CORS| APIS
```

- Only the API publishes a host port, and it is bound to **`127.0.0.1`**.
- Migrations run automatically on API start (`sqlx::migrate!`) before the server begins listening.
- Data persists in the `pgdata` named volume; see [deployment](./06-deployment.md) for backup/restore.

## 10. Data flow examples

**Create a snippet**

```mermaid
sequenceDiagram
    participant U as UI
    participant A as API
    participant S as Service
    participant P as Postgres
    U->>A: POST /api/v1/entries {kind:snippet,title,body,language,tags[]}
    A->>A: validate DTO
    A->>S: create_entry(dto)
    S->>P: BEGIN
    S->>P: INSERT INTO entries (...) RETURNING *
    S->>P: upsert tags, INSERT INTO entry_tags
    S->>P: COMMIT
    P-->>S: entry row
    S-->>A: Entry
    A-->>U: 201 + Location + body(ETag)
```

**Search**

```mermaid
sequenceDiagram
    participant U as UI
    participant A as API
    participant P as Postgres
    U->>A: GET /api/v1/search?q=useEfect&kind=snippet&limit=20
    A->>P: FTS query (ts_rank_cd)
    alt few/no FTS hits
        A->>P: trigram fallback (similarity)
    end
    P-->>A: ranked rows
    A-->>U: {data:[...], next_cursor}
```

## 11. Cross-cutting concerns

| Concern | Approach |
| ------- | -------- |
| **Concurrency** | `ETag` = hash of row `updated_at`+`id`; updates require `If-Match`; mismatch → `409`. |
| **Idempotency of import** | Dedupe by UUID; `on_conflict=skip|overwrite`. |
| **Soft delete** | `deleted_at IS NULL` enforced via partial indexes; repository default filter; explicit `include_deleted`/trash endpoints. |
| **Time** | All timestamps `timestamptz` stored UTC; serialised RFC3339. |
| **Validation** | `validator` on DTOs at the boundary; domain invariants in services. |
| **Security** | Parameterised SQL only; `rehype-sanitize` for Markdown; loopback bind; CORS allowlist. |
| **Testing** | Unit (logic) + integration (real ephemeral Postgres) + Playwright smoke. See [standards § Testing](./05-coding-standards.md#testing). |

## 12. Scalability & limits

DevVault targets a **single user, tens of thousands of entries**. The design comfortably handles that on a laptop:

- Indexes keep list/search at sub-50 ms for a 10k vault.
- Cursor pagination avoids deep-offset scans.
- Connection pool sized small (e.g. `max_connections = 10`) — appropriate for one user.
- Beyond that scale, options are: raise pool size, add read replicas, or (only if needed) an external search service — none required for v1.

## 13. Future architecture considerations

- **Optional read-only token** for exposing the API on a trusted LAN (a single shared secret; still not multi-user). See [security doc](./07-security-and-backups.md#optional-lan-hardening).
- **Revision history** would add an append-only `entry_revisions` table written by the service layer on update.
- **Embeddings/semantic search** would add a `pgvector` column and a hybrid ranker alongside the existing FTS branch — additive, not a rewrite.
- **Single-binary distribution** via `rust-embed` if desktop-style distribution is wanted later.
