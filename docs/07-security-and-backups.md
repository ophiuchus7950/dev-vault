# DevVault — Security & Backups

- **Status:** Draft v1.0
- **Last updated:** 2026-11-04
- **Related:** [ADR-0005 (no auth)](../adr/0005-no-authentication-local-first.md) · [Deployment](./06-deployment.md) · [Architecture](./01-architecture.md)

DevVault has **no authentication by design** ([ADR-0005](../adr/0005-no-authentication-local-first.md)). Its security model is therefore primarily a **deployment/network** model: keep it local, and treat the network boundary as the access control.

---

## 1. Threat model

**Assets**
- The vault contents (snippets, notes, commands, links) — potentially sensitive (internal hostnames, credentials the user pasted, architecture notes).
- The Postgres database and its `pgdata` volume.
- Configuration/secrets (`.env`, DB password).

**Trust model**
- The host machine and its user are trusted.
- Any process/network peer that can reach the API port is **trusted implicitly** (no auth). Therefore reachability must be restricted.

**In scope (we defend against)**
| Threat | Mitigation |
| ------ | ---------- |
| Remote access to the API | API binds to loopback by default (`127.0.0.1:3000`); docs push VPN/TLS + access control for LAN |
| SQL injection | 100% parameterised SQLx macros; dynamic identifiers mapped through a fixed match |
| XSS via Markdown notes | `rehype-sanitize`; no `dangerouslySetInnerHTML`; restrictive CSP |
| Malicious import file | Strict JSON schema validation; size cap; atomic (all-or-nothing) apply |
| Dependency vulnerabilities | `cargo audit` + `pnpm audit` in CI; pinned versions |
| Secret leakage in repo | `.env` git-ignored; no secrets in code; PR checklist |
| Data loss | Documented, tested backup/restore |
| CSRF | Not applicable in the classic sense (no cookies/auth); CORS allowlist + same-origin serving limits cross-origin abuse |

**Out of scope (explicit non-goals)**
- Multi-user authorization and tenant isolation (no users).
- Hosted/multi-tenant operation.
- Protection against a malicious **local** user (they own the machine and the DB).
- Encryption at rest (rely on OS/disk encryption; Postgres data is plaintext on disk).
- AI/LLM data egress (there is none).

## 2. Network exposure — the primary control

### Default (recommended): localhost only

- The Compose stack publishes the API at **`127.0.0.1:3000`** — loopback only, so it is not reachable from the LAN.
- Run the API natively and `BIND_ADDR` defaults to `127.0.0.1:3000` too.
- The database is never published outside the Compose network.

To change the host port, set `API_PORT` in `.env`. To keep the loopback binding explicit:

```yaml
# docker-compose.override.yml
services:
  api:
    ports:
      - "127.0.0.1:3000:3000"
```

### Exposing DevVault beyond localhost

If you want to reach DevVault from another device on your network, **do not port-forward it to the internet.** Choose one:

| Approach | Notes |
| -------- | ----- |
| **VPN (recommended)** — WireGuard/Tailscale | Best. No public exposure; access control via the VPN. |
| **Reverse proxy + TLS + auth** — Caddy/Traefik/Authelia | Terminate HTTPS in front of `127.0.0.1:3000`; add HTTP basic auth or an SSO gate. |
| **SSH tunnel** — `ssh -L 3000:localhost:3000 host` | Great for occasional remote access with no standing exposure. |

If you expose it, you **must** add an authentication layer at the proxy, because the app itself has none.

### Optional LAN hardening (future)

A single shared **read-only API token** (`API_TOKEN`) could be added for trusted-LAN use — still not multi-user. Tracked as a backlog item, not v1.

## 3. Application hardening

| Area | Control |
| ---- | ------- |
| SQL | Only `sqlx::query_as!`/`query!` with bound parameters. Sort/filter identifiers resolved via `match` to `&'static str`. No string-built SQL. |
| Input validation | `validator` on all DTOs at the boundary; length/format caps (`title ≤ 300`, `url ≤ 2048`, tag `≤ 50`). Reject unknown/invalid `kind`, bad cursors, `format` values. |
| Output encoding | JSON serialisation only; Markdown rendered through `rehype-sanitize` (strips scripts/`on*` handlers/`javascript:` URLs). Never `dangerouslySetInnerHTML`. |
| CORS | `tower-http` `CorsLayer` with an explicit allowlist (`CORS_ALLOWED_ORIGINS`); credentials not allowed (no cookies). |
| Headers | If you front the API with a reverse proxy, it should set `X-Content-Type-Options: nosniff`, `Referrer-Policy`, `X-Frame-Options: DENY`, and a restrictive `Content-Security-Policy` for the SPA. |
| Uploads/imports | `IMPORT_MAX_BYTES` cap; JSON schema validated before any write; import applies transactionally. |
| Errors | 5xx bodies never include internal detail; details go to logs with `X-Request-Id`. |
| Logging | No secrets or entry bodies logged; log metadata only. |

### Suggested CSP (serve alongside the SPA, e.g. from a reverse proxy)

```
Content-Security-Policy:
  default-src 'self';
  script-src 'self';
  style-src 'self' 'unsafe-inline';
  img-src 'self' data:;
  connect-src 'self';
  font-src 'self';
  object-src 'none';
  base-uri 'none';
  frame-ancestors 'none';
```

(`'unsafe-inline'` for styles is required by some UI libraries; avoid it for scripts.)

## 4. Dependency & supply-chain hygiene

- CI runs `cargo audit` (RustSec) and `pnpm audit` on every PR and weekly.
- Dependabot/Renovate proposes updates; majors are reviewed manually.
- No `unsafe` Rust (enforced via `unsafe_code = "forbid"`).
- Frontend build has no runtime CDN dependencies (all bundled) — the CSP can stay strict.
- Container images: use official/`-slim` bases, run as a non-root user where possible, and pin base image digests for releases.

## 5. Secrets

- All secrets live in `.env` (git-ignored) or the environment; never in code or images.
- `POSTGRES_PASSWORD` should be strong if the host is shared; it's only reachable inside the Compose network.
- Rotate the DB password by updating `.env` and recreating the `db` + `api` services (the password lives only in Postgres + the connection string).

## 6. Backup & restore runbook

**Goal:** your vault survives disk failure, accidental purge, and bad upgrades.

### What to back up
1. The Postgres database (`pgdata` volume) — the entire vault.
2. Your `.env` (DB credentials) — store it with the backup or in a password manager.
3. Optionally, a JSON export for portability across versions.

### Backup (logical dump — preferred)

```bash
cd dev-vault
mkdir -p backups
docker compose exec -T db pg_dump -U "${POSTGRES_USER:-devvault}" -Fc "${POSTGRES_DB:-devvault}" \
  > "backups/devvault-$(date +%Y%m%d-%H%M%S).dump"
```

Automate daily via cron:
```cron
0 3 * * * cd /path/to/dev-vault && docker compose exec -T db pg_dump -U devvault -Fc devvault > backups/devvault-$(date +\%F).dump
```

**Also** produce a portable JSON export periodically (works across schema changes):
```bash
curl -s 'http://localhost:3000/api/v1/export?format=json' -o "backups/devvault-$(date +%F).json"
```

Copy backups **off the machine** (external drive, NAS, encrypted cloud). A backup on the same disk is not a backup.

### Restore (logical dump)

```bash
cd dev-vault
docker compose up -d db
# wait for healthy
cat backups/devvault-2026-11-04-030000.dump \
  | docker compose exec -T db pg_restore -U devvault -d devvault --clean --if-exists
docker compose up -d
```

### Restore (JSON export, portable)

```bash
docker compose up -d            # fresh, empty DB
curl -s -X POST 'http://localhost:3000/api/v1/import?on_conflict=overwrite' \
  -H 'content-type: application/json' \
  --data-binary @backups/devvault-2026-11-04.json
```

### Recover a single accidentally-purged entry

If not yet purged (still in Trash): use the UI Trash view or `POST /entries/{id}/restore`.
If purged: restore the entry from the most recent dump/export (the vault is single-user, so a full restore to a scratch DB and re-export of the one entry is straightforward), or re-import the JSON export and copy the entry out.

### Verification (do this after *every* setup change)

1. Take a backup.
2. Spin up a **second** Compose project (different `API_PORT` and volume name) or a scratch Postgres.
3. Restore into it.
4. Confirm entry counts and spot-check search: `GET /api/v1/entries?limit=1` and a known query.

Backup/restore is verified as an M7 release gate.

## 7. Data lifecycle & privacy

- **Soft delete:** deleting moves entries to Trash (`deleted_at`); they remain in the DB until purged. Purge is permanent (`DELETE`).
- **No telemetry:** DevVault makes no outbound network calls (no analytics, no crash reporting, no LLM). Everything stays on your machine.
- **Export anytime:** JSON export is the escape hatch; the format is versioned (`"version": 1`).
- **Uninstall:** `docker compose down -v` removes the volume and all data. Take a backup first.

## 8. Incident response (personal scale)

1. **Suspected exposure:** stop the stack (`docker compose down`), check who could reach the API port, add an auth layer (VPN/proxy) before restarting.
2. **Suspected tampering:** compare a fresh JSON export against your latest backup; restore from a known-good dump if needed.
3. **Vulnerability advisory:** update dependencies (`cargo update`, `pnpm update`), rebuild, redeploy; check `cargo audit`/`pnpm audit` output.

## 9. Reporting a vulnerability

This is a personal project without a formal security team. If you find a vulnerability:

- **Do not** open a public issue with exploitation details.
- Open a **private** security advisory on the repository (GitHub → Security → Advisories), or contact the maintainer privately.
- Include: affected version/commit, reproduction steps, impact, and any suggested fix.

There is no bug bounty.

## 10. Security checklist (release gate)

- [ ] The API is bound to loopback (`127.0.0.1`), not the LAN.
- [ ] `.env` is git-ignored and not in the image; no secrets in code.
- [ ] All SQL is parameterised; `unsafe_code` forbidden.
- [ ] Markdown renders through `rehype-sanitize`; no `dangerouslySetInnerHTML`.
- [ ] If exposed beyond localhost, a reverse proxy adds TLS, an auth layer, and security headers/CSP.
- [ ] `cargo audit` and `pnpm audit` clean (or advisories triaged).
- [ ] Backup taken, restore verified into a scratch environment.
- [ ] Import size cap enforced; import is atomic.
- [ ] No telemetry / outbound calls.
