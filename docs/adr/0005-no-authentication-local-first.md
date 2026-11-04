# ADR-0005: No authentication — local-first, single-user

- **Status:** Accepted
- **Date:** 2026-11-04
- **Supersedes:** an earlier plan to include JWT authentication (access + rotating refresh tokens)
- **Related:** [PRD non-goals](../00-prd.md#4-non-goals) · [Security](../07-security-and-backups.md)

## Context

The original project brief listed "user authentication and profiles" and "JWT authentication" as highlights, and an initial design considered a multi-user model with JWT access tokens plus rotating refresh tokens in `httpOnly` cookies.

During planning, the owner decided DevVault is a **personal, local-first tool** — "it works locally for me, that's it." Authentication therefore serves no user and adds real surface area: password hashing, token rotation, refresh reuse detection, session revocation, account flows, and multi-tenant data isolation on every query.

## Decision

**Ship DevVault with no authentication.** There is no user table, no login, no tokens, and no per-user data scoping. Every entry in the database belongs to the single operator.

Security posture instead of auth:
1. The API **binds to `127.0.0.1` by default** (`BIND_ADDR`).
2. In Docker Compose, the API publishes a host port but **bound to `127.0.0.1`** (loopback); the database stays inside the Compose network.
3. Operators who expose it further are directed to front it with a VPN or a reverse proxy that terminates TLS and adds network-level access control (see [security doc](../07-security-and-backups.md#exposing-devvault-beyond-localhost)).
4. Optional, **future**: a single shared read-only API token for trusted-LAN use — still not multi-user.

## Consequences

**Positive**
- Dramatically smaller attack surface and codebase: no auth bugs because there is no auth.
- No per-query `WHERE user_id = $1`, no tenant-isolation class of bugs.
- Instant usability: `docker compose up`, open the browser, start saving.
- Faster to a polished v1; effort concentrates on search/DB/UX.

**Negative**
- **Not safe to expose to the public internet** as-is. The loopback/Compose-network binding is a hard requirement, documented prominently.
- Cannot be shared as a hosted service or used by teams without a future auth layer (a deliberate non-goal).
- The "JWT authentication" portfolio highlight from the original brief is dropped; the showcase value shifts to database design, FTS, cursor pagination and a typed API.

## Reversibility

If auth is ever wanted, the cleanest path is: keep the API in a separate crate/module, add a `users` table and a `user_id` FK on owned tables, introduce an auth middleware/extractor, and scope repo queries by the authenticated user. The layered handler→service→repository design keeps that change localized to the service/repo boundary.
