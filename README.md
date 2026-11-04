# DevVault

> A local-first, single-user developer knowledge manager. Save code snippets, shell commands, notes, links and docs — tag them, search them, export them.

DevVault is a self-hosted personal workspace for the things developers keep re-Googling: that tricky SQL query, the `ffmpeg` incantation, the regex you always forget, the architecture note you wrote six months ago. It runs on your machine via Docker Compose, stores everything in PostgreSQL, and gives you fast full-text + fuzzy search across everything you've saved.

**No accounts. No cloud. No telemetry.** Your machine, your data.

---

## Features (v1)

- **Unified entries** — snippets, commands, notes, links and docs in one library, distinguished by `kind`.
- **Syntax highlighting & editing** — CodeMirror 6 editor, Shiki rendering, copy-to-clipboard.
- **Markdown notes** — GitHub-flavoured Markdown with live preview.
- **Tags** — normalized, renameable, with a tag cloud and autocomplete.
- **Collections** — optional folders on top of flat tags (e.g. "Rust", "Interviews").
- **Search** — PostgreSQL full-text search (`tsvector`) **plus** `pg_trgm` fuzzy matching (`useEfect` → `useEffect`).
- **Filtering & sorting** — by kind, language, tag, collection, pinned, archived.
- **Pin / archive / trash** — pin favourites, soft-delete to a restorable Trash.
- **Import / Export** — per-entry and whole-vault, as JSON or Markdown.
- **Dark mode + command palette** — `Cmd/Ctrl-K` to jump anywhere, run anything.
- **Auto-generated API docs** — OpenAPI + Swagger UI at `/docs`.
- **CLI client** *(planned, M6)* — add and search from your terminal.

## Tech stack

| Layer      | Choice                                                                    |
| ---------- | ------------------------------------------------------------------------- |
| Backend    | Rust, [Axum](https://github.com/tokio-rs/axum), Tokio                     |
| Database   | PostgreSQL 16, [SQLx](https://github.com/launchbadge/sqlx) (compile-time checked SQL) |
| Search     | Postgres `tsvector` + GIN, `pg_trgm` for fuzzy matching                   |
| Frontend   | Vite + React + TypeScript                                                 |
| Data/state | [TanStack Query](https://tanstack.com/query) + [Zustand](https://zustand-demo.pmnd.rs/) |
| Editor     | CodeMirror 6 (edit) + Shiki (render)                                      |
| Markdown   | react-markdown + remark-gfm + rehype                                      |
| Tooling    | Biome (lint/format), pnpm, cargo fmt + clippy                             |
| Packaging  | Docker Compose (Postgres + API + Nginx)                                   |
| CI         | GitHub Actions                                                            |

## Quickstart

One command starts the whole app — UI, API, and database:

```bash
git clone https://github.com/<you>/dev-vault.git
cd dev-vault
cp .env.example .env        # adjust if you like
docker compose up --build -d
```

| What | Where |
| --- | --- |
| **App (UI + API)** | http://localhost:8080 |
| Swagger UI | http://localhost:8080/docs |
| API (direct, loopback) | http://localhost:3000 |
| OpenAPI JSON | http://localhost:8080/api-docs/openapi.json |

The API runs migrations automatically on start. The `web` container serves the built SPA with nginx and proxies `/api` to the API container — same-origin, so no CORS in production. Everything is bound to `127.0.0.1`.

Hot-reload development instead? `docker compose up -d db api && cd frontend && pnpm install && pnpm dev` → http://localhost:5173.

## Repository layout

```
dev-vault/
├── backend/            # Rust API (Axum + SQLx)
│   ├── migrations/     # sqlx SQL migrations
│   └── src/
├── frontend/           # Vite + React SPA (nginx-served in Docker)
├── docs/               # product + engineering documentation
├── docker-compose.yml  # db + api + web, one command
├── justfile            # task shortcuts (optional; needs `just`)
└── README.md
```

## Documentation

All planning and design docs live in [`docs/`](./docs/README.md):

| Doc | What it covers |
| --- | --- |
| [PRD](./docs/00-prd.md) | Problem, users, scope, requirements, success metrics |
| [Architecture](./docs/01-architecture.md) | System design, components, data flow, module layout |
| [Roadmap](./docs/02-roadmap.md) | Milestones M0–M7 with scope and definition of done |
| [Database schema](./docs/03-database-schema.md) | Full DDL, indexes, triggers, migrations |
| [API specification](./docs/04-api-specification.md) | Conventions, endpoints, examples, pagination |
| [Coding standards](./docs/05-coding-standards.md) | Rust + TS conventions, git workflow, reviews |
| [Deployment](./docs/06-deployment.md) | Compose, backups, upgrades, troubleshooting |
| [Security & backups](./docs/07-security-and-backups.md) | Threat model, hardening, backup/restore |
| [Frontend build prompt](./docs/frontend-build-prompt.md) | Standalone spec to hand off frontend work |
| [ADRs](./docs/adr/) | Architecture Decision Records |

## Development

```bash
# the whole stack, rebuilt
docker compose up --build -d

# backend (Rust)
cd backend && cargo run          # needs DATABASE_URL + a running Postgres
cargo test && cargo clippy

# frontend (Vite dev server on :5173, proxies /api to :3000)
cd frontend && pnpm install && pnpm dev
pnpm lint && pnpm typecheck      # biome + tsc

# task shortcuts if you have `just` installed
just up        # start everything → http://localhost:8080
just dev       # db + api in Docker, frontend with hot reload
just check     # backend: fmt + clippy + tests
just check-web # frontend: lint + typecheck + build
just down      # stop (data is kept)
```

See [CONTRIBUTING.md](./CONTRIBUTING.md) for the full workflow.

## Non-goals

DevVault deliberately does **not** include authentication, multi-user support, cloud sync, public share links, or AI features. See the [PRD non-goals](./docs/00-prd.md#non-goals) and [ADR-0005](./docs/adr/0005-no-authentication-local-first.md).

## License

[MIT](./LICENSE)
