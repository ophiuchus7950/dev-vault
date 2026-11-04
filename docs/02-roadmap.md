# DevVault — Product Roadmap

- **Status:** Draft v1.0
- **Last updated:** 2026-11-04
- **Related:** [PRD](./00-prd.md) · [Architecture](./01-architecture.md) · [DB Schema](./03-database-schema.md)

This roadmap is organised as **milestones (M0–M7)**. Each milestone is independently shippable and ends with a demo you can actually use. Durations are intentionally omitted — order and scope matter, wall-clock time does not.

---

## Milestone overview

| # | Milestone | Outcome | Key FRs |
| - | --------- | ------- | ------- |
| **M0** | Foundation | Repo, toolchain, CI, DB up, empty API + SPA skeleton | — |
| **M1** | Core CRUD | Create/read/update/delete entries of every kind | FR-1..FR-8 |
| **M2** | Organise | Tags + collections + filtering/sorting | FR-9..FR-13 |
| **M3** | Search | FTS + fuzzy search, ranked, paginated | FR-14..FR-17, FR-27 |
| **M4** | UX polish | Dark mode, command palette, shortcuts, toasts | FR-8, NFR-4 |
| **M5** | Move data | Import/export JSON + Markdown | FR-18..FR-21 |
| **M6** | CLI | `devvault add` / `devvault search` | FR-22..FR-24 |
| **M7** | Release | Packaging, docs, demo, hardening | NFR-1..NFR-7 |

**Dependency sketch**

```
M0 ─▶ M1 ─▶ M2 ─▶ M3 ─┬─▶ M4 ─▶ M5 ─▶ M7
                      └─▶ M6 ─────────▶ M7
```

M4 and M6 can proceed in parallel after M3. M5 (export) depends only on M1 but is sequenced later for focus.

---

## M0 — Foundation

**Goal:** a running skeleton — Postgres, migrations, health-checked Axum API, Vite SPA, CI green.

**Scope**
- Monorepo scaffold: `backend/`, `frontend/`, `cli/`, `docs/`, root `docker-compose.yml`.
- `docker-compose.yml`: `postgres:16` + `api`, with healthchecks + named volume.
- Axum app: config loading (`dotenvy`), `tracing`, `tower-http` (trace/CORS/compression), `/health` and `/ready` endpoints.
- SQLx connection pool + `sqlx migrate` wired to run on startup.
- First migration: extensions (`pgcrypto`/`pg_trgm`/`unaccent`), `updated_at` trigger function.
- SPA skeleton: Vite + React + TS, router, API client, one page calling `/health`.
- utoipa scaffold serving `/docs` (empty spec initially).
- GitHub Actions: backend (fmt/clippy/test with Postgres service), frontend (biome/tsc/build).
- `justfile` (or Makefile) with `dev`, `check`, `test`, `e2e`.

**Definition of done**
- [ ] `docker compose up` starts all services; `/health` returns `200 {"status":"ok"}`.
- [ ] SPA loads and shows API health.
- [ ] `/docs` renders Swagger UI.
- [ ] CI runs on a PR and passes.
- [ ] `.env.example` documents every variable.

---

## M1 — Core CRUD

**Goal:** you can capture and manage every kind of entry via the API and UI.

**Scope**
- Migration: `entries` table (see [DB schema](./03-database-schema.md#entries)).
- DTOs + validation (`validator`): create/update payloads per kind; kind-specific required fields (link→url, command→shell optional, etc.).
- Repository layer with `sqlx::query_as!` for create/get/list/update/soft-delete/restore/purge.
- Handlers: `POST/GET/PATCH/DELETE /api/v1/entries`, `POST /entries/{id}/restore`, `DELETE /entries/{id}/purge`.
- ETag + `If-Match` optimistic concurrency on updates; `409` on stale.
- Frontend: entry list (cursor-paginated), detail view, create/edit forms with CodeMirror for snippets/commands and Markdown for notes, copy button, Shiki rendering.
- Soft-delete → Trash view with restore/purge.

**Definition of done**
- [ ] All five kinds can be created, edited, viewed, trashed, restored, purged.
- [ ] Create returns `201` + `Location`; delete returns `204`.
- [ ] Integration tests cover each kind's happy path + validation failures.
- [ ] Keyboard users can complete create/edit without a mouse.

---

## M2 — Organise

**Goal:** a large vault stays navigable.

**Scope**
- Migrations: `tags`, `entry_tags`, `collections`, `entry_collections`.
- Tag endpoints: list (with counts), rename, attach/detach on entries.
- Collection endpoints: CRUD, assign/unassign entries.
- Filtering on list endpoint: `kind`, `language`, `tag`, `collection`, `is_pinned`, `is_archived`.
- Sorting: `updated_at`, `created_at`, `title`, (relevance once M3 lands).
- Frontend: tag chips + autocomplete, tag cloud, collection sidebar/tree, filter bar, sort control, pin/archive toggles.

**Definition of done**
- [ ] Renaming a tag updates all entries instantly.
- [ ] Combining filters works (kind + tag + pinned, etc.).
- [ ] Tag counts match reality after create/delete.
- [ ] Tests cover many-to-many integrity and filter combinations.

---

## M3 — Search

**Goal:** find anything fast, typos included.

**Scope**
- Migration: generated `search tsvector` column (weighted title>body>tags), GIN index, `pg_trgm` GIN index, `unaccent` normalization.
- `GET /api/v1/search` with `q`, all filters, cursor pagination, and `?mode=fts|trgm|hybrid` (default hybrid).
- Ranking via `ts_rank_cd` combined with `similarity()` fallback for low-FTS-hit queries.
- Frontend: search bar with debounce, highlighted matches (Shiki/`<mark>`), empty-state suggestions, "search within collection/tag" affordance.
- Curate a fixed query set (incl. deliberate typos) as a search-relevance regression suite.

**Definition of done**
- [ ] `useEfect` returns `useEffect` snippets in the top 3.
- [ ] p95 query latency < 50 ms at 10k entries (NFR-1).
- [ ] Search respects every filter from M2.
- [ ] Relevance test suite passes in CI.

---

## M4 — UX polish

**Goal:** it feels like a fast, professional dev tool.

**Scope**
- Theme system: light/dark, persisted (Zustand), respecting `prefers-color-scheme`.
- Command palette (`Cmd/Ctrl-K`): fuzzy jump to entry/collection, run actions (new snippet, new note, toggle theme, export).
- Global shortcuts: `n` new, `/` focus search, `⌘S` save, `Esc` close, `?` shortcut help.
- Optimistic updates for pin/tag/archive via TanStack Query mutations.
- Toasts/undo for destructive actions; skeletons for loading; empty states.
- Accessibility pass: focus rings, ARIA labels, contrast, keyboard trap checks.

**Definition of done**
- [ ] Entire app is navigable by keyboard; palette reachable everywhere.
- [ ] React mutations feel instant (optimistic) and roll back on error.
- [ ] Lighthouse a11y ≥ 95 on main views.
- [ ] No layout shift on load (CLS ≈ 0).

---

## M5 — Move data

**Goal:** your vault is portable, in and out.

**Scope**
- `GET /api/v1/entries/{id}/export?format=md|json`.
- `GET /api/v1/export?format=json|zip` (whole vault).
- `POST /api/v1/import` (JSON), with `?on_conflict=skip|overwrite` (default skip; dedupe by UUID).
- Export serialisation includes kind, title, body, language/shell/url, tags, collections, pinned/archived, timestamps.
- Frontend: export buttons (entry + vault), import dialog with dry-run summary (N new / M skipped / K conflicts).
- Round-trip test: export → wipe → import → deep-equal.

**Definition of done**
- [ ] Vault round-trips losslessly (JSON) and readably (Markdown archive with front-matter).
- [ ] Re-import is idempotent (no duplicates).
- [ ] Import reports counts and never partially applies a failed file.

---

## M6 — CLI

**Goal:** capture and search without leaving the terminal.

**Scope**
- `devvault` binary in `cli/` (Rust) using `clap` + `reqwest`.
- Commands: `add` (flags + STDIN, `--kind/--lang/--tag/--collection`), `search`, `get`, `list`, `export`, `config`.
- Config resolution: `--base-url` > env `DEVVVAULT_URL` > config file `~/.config/devvault/config.toml`.
- Friendly output (tables) + `--json` for scripting.
- Shell-completion generation; pipe-friendly (`echo "$cmd" | devvault add --kind command`).

**Definition of done**
- [ ] `devvault search "pg_trgm"` returns results over HTTP against the API.
- [ ] Output pipes cleanly into `jq` with `--json`.
- [ ] Distributed as a release binary built in CI (linux/mac, amd64/arm64).

---

## M7 — Release & hardening

**Goal:** a polished, documented, reproducible v1.0.0.

**Scope**
- Compose image publishes (multi-arch), healthchecks finalised, graceful shutdown.
- Docs complete and accurate: README, PRD, architecture, schema, API spec, standards, deployment, security.
- Seed/demo data script + screenshots/GIF in README.
- Performance pass: explain-analyse the hot queries, confirm indexes are used.
- Security pass: input validation review, dependency audit (`cargo audit`, `pnpm audit`), localhost binding default documented.
- Backup/restore runbook tested end-to-end (`pg_dump` + restore).
- `v1.0.0` tag, CHANGELOG, release notes.

**Definition of done**
- [ ] A fresh clone reaches a working app with one command.
- [ ] Backup → restore verified against a non-trivial vault.
- [ ] All non-goals still absent (no auth, no cloud).
- [ ] `docs/` matches shipped behaviour.

---

## Backlog (post-v1, unordered)

- Entry revision history (undo edits).
- Public read-only share links (`/s/<slug>`).
- Usage counters ("most used", "recently used").
- Optional read-only API token for trusted-LAN exposure.
- File/image attachments.
- Browser extension / "save from page".
- Semantic/embeddings search alongside FTS.
- Hierarchical tags as an alternative to collections.

## How we prioritise

1. **MUST requirements + dependencies first** (per PRD priorities).
2. **Dogfood value** — does it make the maintainer's own use better this week?
3. **Showcase value** — does it teach/demonstrate something meaningful (FTS, cursor paging, typed API)?
4. **Cost** — prefer the smallest change that ships the milestone's outcome.

Anything that doesn't fit a milestone goes to the backlog. Extras never block a milestone's definition of done.
