# DevVault — Deployment

- **Status:** Draft v1.0
- **Last updated:** 2026-11-04
- **Related:** [Architecture §9](../01-architecture.md#9-deployment-topology) · [Security & Backups](./07-security-and-backups.md)

DevVault ships as a **Docker Compose** stack: PostgreSQL and the Rust API. Only the API's port is published, and only on loopback. The frontend is deployed separately (its own dev server or static host).

---

## 1. Topology

```mermaid
flowchart TB
    Host(["Host (localhost:3000)"])
    subgraph net["docker compose network"]
        api["api (Rust) — publishes 127.0.0.1:3000"]
        db["postgres:16 — internal :5432, volume pgdata"]
        api --> db
    end
    Host --> api
    SPA["frontend (Vite dev server / static host)\npoints at http://localhost:3000/api/v1"]
    SPA -.->|CORS| api
```

- Only the **`api`** service publishes a host port, and it is bound to **`127.0.0.1`** so it is not reachable from the LAN.
- The database is reachable **only inside the Compose network**.
- The API runs migrations automatically on start (`sqlx::migrate!`), so schema changes apply on deploy.
- Swagger UI is at `http://localhost:3000/docs`.

## 2. Prerequisites

| Requirement | Version | Notes |
| ----------- | ------- | ----- |
| Docker Engine | 24+ | With Compose v2 (`docker compose`) |
| Arch | amd64 or arm64 | Images built multi-arch |
| Disk | ~500 MB + data | Postgres volume grows with your vault |

## 3. Environment variables

Copy `.env.example` → `.env` and adjust. Compose reads it.

| Variable | Default | Service | Purpose |
| -------- | ------- | ------- | ------- |
| `POSTGRES_USER` | `devvault` | db | DB user |
| `POSTGRES_PASSWORD` | *(set me)* | db | DB password |
| `POSTGRES_DB` | `devvault` | db | DB name |
| `DATABASE_URL` | `postgres://devvault:…@db:5432/devvault` | api | SQLx connection string |
| `BIND_ADDR` | `0.0.0.0:3000` (in container) | api | Bind inside container; host exposure is controlled by Compose |
| `CORS_ALLOWED_ORIGINS` | `http://localhost:5173,http://localhost:8080` | api | Allowed browser origins |
| `RUST_LOG` | `info,devvault=info` | api | Log filter |
| `LOG_FORMAT` | `json` | api | `json` in prod |
| `MAX_PAGE_SIZE` | `100` | api | Pagination cap |
| `DEFAULT_PAGE_SIZE` | `20` | api | Default `limit` |
| `SEARCH_MIN_TRGM_SCORE` | `0.3` | api | Fuzzy-match threshold |
| `API_PORT` | `3000` | api | Host port (bound to 127.0.0.1) |
| `IMPORT_MAX_BYTES` | `16777216` | api | Max import payload |

> **Security note:** never commit `.env`. Use strong `POSTGRES_PASSWORD` if the host is shared.

## 4. `docker compose` reference

```yaml
# docker-compose.yml (repo root)
services:
  db:
    image: postgres:16
    environment:
      POSTGRES_USER: ${POSTGRES_USER:-devvault}
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD:?set POSTGRES_PASSWORD}
      POSTGRES_DB: ${POSTGRES_DB:-devvault}
    volumes:
      - pgdata:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U ${POSTGRES_USER:-devvault} -d ${POSTGRES_DB:-devvault}"]
      interval: 5s
      timeout: 3s
      retries: 10
    restart: unless-stopped

  api:
    build:
      context: .
      dockerfile: backend/Dockerfile
    environment:
      DATABASE_URL: postgres://${POSTGRES_USER:-devvault}:${POSTGRES_PASSWORD}@db:5432/${POSTGRES_DB:-devvault}
      BIND_ADDR: 0.0.0.0:3000
      LOG_FORMAT: json
    depends_on:
      db:
        condition: service_healthy
    healthcheck:
      test: ["CMD", "curl", "-fsS", "http://localhost:3000/health"]
      interval: 10s
      timeout: 3s
      retries: 5
    restart: unless-stopped

volumes:
  pgdata:
```

- `depends_on: condition: service_healthy` ensures the API starts only after Postgres is accepting connections.
- `restart: unless-stopped` keeps the stack up across reboots (combined with Docker's own restart policy).
- The frontend is deployed separately and points at the API; there is no `web` service in this stack.

## 5. First run

```bash
git clone <your-repo-url> dev-vault && cd dev-vault
cp .env.example .env
# edit .env: set a strong POSTGRES_PASSWORD
docker compose up --build -d
docker compose ps          # wait for healthy
```

Then the API is at **<http://localhost:3000>** and its Swagger UI at **<http://localhost:3000/docs>**.

Optionally seed demo data (dev only):
```bash
docker compose exec -e SEED_DEMO=1 api /app/devvault seed
```

## 6. Development workflow

For local development you usually want only Postgres in Docker and both apps running natively for hot-reload:

```bash
docker compose up -d db

# backend
cd backend
export DATABASE_URL=postgres://devvault:devpass@localhost:5432/devvault
cargo sqlx migrate run
cargo watch -x run            # http://localhost:3000

# frontend
cd frontend
pnpm install
pnpm dev                      # http://localhost:5173 (proxies /api to :3000)
```

`just` aliases (see `justfile`): `just dev`, `just check`, `just test`, `just e2e`, `just migrate`.

### Frontend dev proxy

Vite proxies `/api` and `/docs` to `http://localhost:3000` so the SPA uses same-origin paths in dev and prod. No CORS pain.

## 7. Exposing the API beyond localhost

The Compose stack publishes the API only on `127.0.0.1:3000`. To reach DevVault from another device:

- **VPN (recommended)** — WireGuard/Tailscale. No public exposure; access controlled by the VPN.
- **Reverse proxy + TLS** (Caddy/Traefik) in front of `127.0.0.1:3000`. The same proxy can optionally serve the built SPA and forward `/api`, `/api-docs`, and `/docs` to the API.
- **SSH tunnel** — `ssh -L 3000:localhost:3000 host`.

Because the app has **no authentication**, never port-forward it directly to the internet. Add an auth layer at the proxy or use a VPN. See [security doc § Exposing DevVault](./07-security-and-backups.md#exposing-devvault-beyond-localhost).

If you front it with a proxy, set these headers there: `X-Content-Type-Options: nosniff`, `Referrer-Policy: strict-origin-when-cross-origin`, and a restrictive `Content-Security-Policy`.

## 8. Backups

The whole vault is one Postgres database in the `pgdata` volume. Two supported methods:

**A. Logical dump (recommended, portable)**
```bash
docker compose exec -T db pg_dump -U devvault -Fc devvault > devvault-$(date +%F).dump
```
Restore:
```bash
cat devvault-2026-11-04.dump | docker compose exec -T db pg_restore -U devvault -d devvault --clean --if-exists
```

**B. Application export (portable across versions)**
```bash
curl -s 'http://localhost:3000/api/v1/export?format=json' -o devvault-backup.json
# re-import: POST /api/v1/import
```

**C. Volume snapshot** — `docker compose stop db` then back up the `pgdata` volume directory. Use only when the DB is stopped for a consistent snapshot.

Automate A or B with cron; keep the dumps **off the machine**. Restore procedure is tested as part of M7. See [security doc](./07-security-and-backups.md#backup--restore-runbook) for the full runbook.

## 9. Upgrades

```bash
git pull
docker compose build
docker compose up -d      # API runs migrations on start
docker compose logs -f api
```

- Migrations are additive and forward-only, so rolling forward is safe.
- **Always take a backup before upgrading.** There is no automatic down-migration.
- To roll back code: `git checkout <previous-tag> && docker compose up -d --build` (schema stays; additive migrations are backwards-compatible).

## 10. Operations

| Task | Command |
| ---- | ------- |
| Logs (all) | `docker compose logs -f` |
| Logs (api only) | `docker compose logs -f api` |
| Shell into DB | `docker compose exec db psql -U devvault -d devvault` |
| Check readiness | `docker compose exec api curl -fsS localhost:3000/ready` |
| Restart API | `docker compose restart api` |
| Stop (keep data) | `docker compose down` |
| Stop + wipe data | `docker compose down -v` ⚠️ destroys the volume |

## 11. Resource sizing

Single-user, 10k entries:

| Service | CPU | RAM |
| ------- | --- | --- |
| postgres | ~0.5 core | ~256 MB |
| api | ~0.1 core idle | ~50–100 MB |

A Raspberry Pi 4 / small VPS handles this comfortably.

## 12. Troubleshooting

| Symptom | Likely cause | Fix |
| ------- | ------------ | --- |
| SPA can't reach the API | CORS or wrong base URL | Add the SPA origin to `CORS_ALLOWED_ORIGINS`; point the client at `http://localhost:3000/api/v1` |
| API exits on boot, "connection refused" | DB not healthy yet | Ensure `depends_on: service_healthy`; check `pg_isready` |
| Migrations fail | Permission or extension missing | DB user must own the schema; ensure `pg_trgm`/`unaccent` installable (postgres image includes them) |
| Search returns nothing | FTS stopwords or short query | Queries < 2 chars are rejected; try `mode=trgm`; check `SEARCH_MIN_TRGM_SCORE` |
| 409 on edit | Stale `ETag` | Refetch, reapply, retry with the new `If-Match` |
| Port 3000 in use | Another service | Set `API_PORT` in `.env` |
| ARM build issues | Missing multi-arch image | Use the published multi-arch images or build with `--platform` |

## 13. Published images / releases

On a `vX.Y.Z` tag, CI builds and publishes multi-arch images (`linux/amd64`, `linux/arm64`) and the `devvault` CLI binaries. Self-hosters can pin: `image: ghcr.io/<owner>/devvault-api:v1.0.0`.
