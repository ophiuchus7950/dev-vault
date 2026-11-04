# DevVault — Product Requirements Document (PRD)

- **Status:** Draft v1.0
- **Owner:** DevVault maintainers
- **Last updated:** 2026-11-04
- **Related docs:** [Architecture](./01-architecture.md) · [Roadmap](./02-roadmap.md) · [DB Schema](./03-database-schema.md) · [API Spec](./04-api-specification.md)

---

## 1. Summary

DevVault is a **local-first, single-user developer knowledge manager**. It gives developers one fast, searchable, self-hosted place to keep the artifacts they otherwise scatter across notes apps, browser bookmarks, gists, Slack DMs to themselves, and `.txt` files.

It ships as a Docker Compose stack: a Rust REST API, a PostgreSQL database, and a React SPA — plus an optional CLI. There is **no authentication**, no account system, and no cloud dependency. The database is the single source of truth and everything is exportable.

## 2. Problem

Developers repeatedly re-solve the same small problems:

- "What was that `find` command with `-exec` again?"
- "Where did I put that Postgres upsert snippet?"
- "I know I wrote a note about our deploy topology — where is it?"

Existing options fail in specific ways:

| Current workaround | Why it fails |
| ------------------ | ------------ |
| Browser bookmarks / gists | No unified search, not local, mixed with other bookmarks |
| Note apps (Notion/Obsidian) | Heavy, cloud-locked, poor code formatting, no provenance |
| Editor snippets | Only code, no commands/notes/links, not searchable across kinds |
| Scattered `.txt` files | No tags, no search, no structure |

The core unmet need: **one place, locally owned, where any kind of developer knowledge is stored once and found fast.**

## 3. Goals

- **G1 — Capture is trivial.** Adding a snippet/command/note/link takes seconds, from the web UI or CLI.
- **G2 — Retrieval is fast and forgiving.** Full-text *and* fuzzy search returns the right item even with typos or partial identifiers.
- **G3 — Everything is organised.** Tags (and optional collections) make large vaults navigable; filter/sort by kind, language, tag, pinned, archived.
- **G4 — Your data is yours.** One Postgres database; lossless import/export as JSON and Markdown.
- **G5 — Runs anywhere, simply.** `docker compose up` and it works; no accounts, no external services.
- **G6 — It's a good engineering showcase.** Demonstrates DB design, FTS, a typed Rust API, and a modern React frontend.

## 4. Non-goals

Explicitly **out of scope** for v1 (and mostly forever — see [ADR-0005](./adr/0005-no-authentication-local-first.md)):

- **Authentication, accounts, multi-user, roles, permissions.** Deliberately removed; DevVault is single-user and local.
- **Cloud sync, remote hosting, SaaS.** Self-hosted only.
- **Public share links.** Cut from v1 scope.
- **Real-time collaboration or CRDTs.**
- **AI/LLM features** (auto-tagging, semantic search, summarisation).
- **Mobile-native apps.** Responsive web only.
- **File/binary attachments** (images, PDFs) in v1.
- **Entry revision history** (cut from v1; possible later).

## 5. Users & personas

| Persona | Description | Primary needs |
| ------- | ----------- | ------------- |
| **The Solo Developer (primary)** | One developer running DevVault on their laptop or home server | Fast capture, forgiving search, tags, dark mode, CLI |
| **The Learner** | Studying Rust/Postgres/React and wants a real project to run and read | Clear module structure, meaningful tests, docs |
| **The Self-hoster** | Runs services on a home server / NAS behind a VPN | Compose packaging, backups, low resource use |

Explicitly **not** served: teams, public consumers, non-technical users.

## 6. User stories

**Capture**
- As a developer, I can save a code snippet with a title, body, language, and tags.
- As a developer, I can save a shell command with the shell it targets.
- As a developer, I can write a Markdown note with a live preview.
- As a developer, I can save a link with a URL and description.
- As a developer, I can add an entry from the CLI without opening a browser.

**Organise**
- As a developer, I can tag any entry with multiple tags and rename a tag everywhere at once.
- As a developer, I can group entries into collections and browse a collection as a tree/list.
- As a developer, I can pin important entries to the top and archive ones I'm done with.
- As a developer, I can move entries to Trash and restore them later.

**Find**
- As a developer, I can search by title, body, language, or tag.
- As a developer, I can find a snippet even when I misspell an identifier (`useEfect` → `useEffect`).
- As a developer, I can filter results by kind, language, tag, collection, pinned, archived.
- As a developer, I can sort by relevance, newest, oldest, or title.

**Consume & move**
- As a developer, I can copy a snippet's code in one click.
- As a developer, I can export a single entry or my entire vault as JSON or Markdown.
- As a developer, I can import a previously exported vault.
- As a developer, I can navigate the whole app from the keyboard (command palette + shortcuts).

## 7. Functional requirements

Requirement IDs are referenced by the roadmap and tests. **P** = priority (MUST / SHOULD / MAY).

### 7.1 Entries

| ID | Requirement | P |
| -- | ----------- | - |
| FR-1 | Create an entry of `kind` ∈ {snippet, command, note, link, doc} with title + body. | MUST |
| FR-2 | Snippets carry an optional `language`; commands carry an optional `shell`; links carry a `url`. | MUST |
| FR-3 | Edit and delete entries; deletion is soft (Trash). | MUST |
| FR-4 | Restore or permanently purge an entry from Trash. | MUST |
| FR-5 | Pin (`is_pinned`) and archive (`is_archived`) entries. | MUST |
| FR-6 | Every entry records `created_at` / `updated_at` (UTC). | MUST |
| FR-7 | IDs are UUIDv7 (time-sortable, non-enumerable). | MUST |
| FR-8 | Copy code/body to clipboard from the UI. | MUST |

### 7.2 Organising

| ID | Requirement | P |
| -- | ----------- | - |
| FR-9 | Attach zero or more tags; tags are normalized and shared. | MUST |
| FR-10 | Rename a tag and have it reflected everywhere. | SHOULD |
| FR-11 | List tags with usage counts (tag cloud / autocomplete). | SHOULD |
| FR-12 | Create collections and assign entries to one or more collections. | SHOULD |
| FR-13 | Filter the entry list by any combination of kind, language, tag, collection, pinned, archived. | MUST |

### 7.3 Search

| ID | Requirement | P |
| -- | ----------- | - |
| FR-14 | Full-text search across title, body, language and tags. | MUST |
| FR-15 | Results ranked by relevance with a title-weighted `tsvector`. | MUST |
| FR-16 | Fuzzy/partial matching via `pg_trgm` so typos and partial identifiers still match. | MUST |
| FR-17 | Search results are paginated and filterable like the list endpoint. | MUST |

### 7.4 Import / Export

| ID | Requirement | P |
| -- | ----------- | - |
| FR-18 | Export a single entry as Markdown or JSON. | MUST |
| FR-19 | Export the whole vault as a single JSON document or a Markdown archive. | MUST |
| FR-20 | Import a previously exported vault, preserving kinds, tags, collections and timestamps. | SHOULD |
| FR-21 | Import is idempotent-safe: re-importing the same file does not duplicate entries (dedupe by id). | SHOULD |

### 7.5 CLI

| ID | Requirement | P |
| -- | ----------- | - |
| FR-22 | `devvault add` creates an entry from flags/STDIN. | SHOULD |
| FR-23 | `devvault search <query>` prints matching entries. | SHOULD |
| FR-24 | CLI reads base URL and defaults from a config file/env. | SHOULD |

### 7.6 API & docs

| ID | Requirement | P |
| -- | ----------- | - |
| FR-25 | A versioned REST API at `/api/v1` with consistent conventions. | MUST |
| FR-26 | Auto-generated OpenAPI served with Swagger UI at `/docs`. | MUST |
| FR-27 | All list endpoints use cursor-based pagination. | MUST |

## 8. Non-functional requirements

| ID | Requirement | Target |
| -- | ----------- | ------ |
| NFR-1 | Local search/list latency | p95 < 50 ms for a 10k-entry vault on commodity hardware |
| NFR-2 | Cold start (`docker compose up`) | Ready in < 15 s; migrations run automatically |
| NFR-3 | Bundle size (frontend, gzipped) | Initial route < 300 KB gz; editor/Shiki lazy-loaded |
| NFR-4 | Accessibility | WCAG 2.1 AA for keyboard nav, contrast, focus states |
| NFR-5 | Browser support | Latest Chromium, Firefox, Safari (desktop) |
| NFR-6 | Data durability | Everything in Postgres; documented backup/restore |
| NFR-7 | Portability | Runs on amd64 + arm64 with Docker |
| NFR-8 | Observability | Structured JSON logs; request tracing with `tracing` |

## 9. Scope: v1 vs later

**In v1:** unified entries CRUD, tags, collections, FTS + fuzzy search, filtering/sorting, pin/archive/trash, Markdown notes, syntax highlighting + editing, import/export (JSON + MD), dark mode, command palette, CLI, Docker Compose, OpenAPI docs.

**Later (post-v1, not committed):** entry revision history, public share links, usage counters, file attachments, browser extension, optional read-only API token for LAN exposure, semantic search.

## 10. Success metrics

Because DevVault is a personal, local tool, "success" is measured by the builder's usage and the project's engineering quality:

| Metric | Target |
| ------ | ------ |
| Dogfooding | Maintainer uses DevVault as their primary snippet store |
| Search quality | ≥ 90% of test queries (typos included) return the intended entry in top 3 |
| Test coverage of critical paths | CRUD + search + import/export covered by integration tests |
| CI health | `main` green; clippy/fmt/biome/tsc enforced |
| Docs completeness | PRD, architecture, schema, API, standards all current |
| Repo signal | Clear README, working `docker compose up`, one-command demo |

## 11. Risks & mitigations

| Risk | Impact | Mitigation |
| ---- | ------ | ---------- |
| FTS relevance feels wrong | Poor search UX | Weight title > body > tag; combine `ts_rank` with `pg_trgm` similarity; tune with a fixed query set |
| Cursor pagination bugs | Skipped/duplicated rows | Composite `(created_at, id)` keyset + tests for boundary conditions |
| Random PK insert order (if UUIDv4) | Index bloat | Use **UUIDv7** so inserts stay roughly sorted |
| Soft-delete leaks into queries | Trashed rows appear | Enforce `deleted_at IS NULL` via partial indexes and a repository-level default filter |
| Markdown XSS | Injecting scripts into notes | `rehype-sanitize`; never `dangerouslySetInnerHTML` |
| Scope creep from extras | v1 never ships | Strict milestone gates in the roadmap; extras are SHOULD/MAY |
| No-auth exposure if bound to 0.0.0.0 | Anyone on the network can read/write | Bind to localhost by default; document LAN hardening (see security doc) |

## 12. Open questions

1. **`doc` kind** — is a long-form "doc" distinct enough from a "note", or should we drop it to `note`? *(Leaning: keep `kind` enum but treat `doc` as a note with a `#` heading convention.)*
2. **Collections vs tags overlap** — do collections earn their keep, or do hierarchical tags suffice? *(v1 builds collections; validate via dogfooding.)*
3. **CLI distribution** — ship as a standalone binary, a `cargo install`, or a Node wrapper? *(Leaning: standalone Rust binary, built in CI.)*
4. **Import merge policy** — on id collision, skip or overwrite? *(Leaning: skip by default, `--overwrite` flag opt-in.)*
5. **Default sort** — `updated_at DESC` vs `created_at DESC` for the home list? *(Leaning: `updated_at DESC`.)*

## 13. Definitions

- **Entry** — any saved item; a row in `entries`, typed by `kind`.
- **Kind** — snippet | command | note | link | doc.
- **Tag** — a normalized label, many-to-many with entries.
- **Collection** — an optional folder-like grouping, many-to-many with entries.
- **Trash** — soft-deleted entries (`deleted_at` non-null), restorable or purgeable.
- **Vault** — the entire contents of one DevVault database.
