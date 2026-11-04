# DevVault Documentation

This directory holds the product and engineering documentation for DevVault. Read in order if you're new.

## Index

| # | Document | Purpose | Audience |
| - | -------- | ------- | -------- |
| 00 | [PRD](./00-prd.md) | Problem, users, scope, requirements, metrics | Everyone |
| 01 | [Technical Architecture](./01-architecture.md) | System design, components, data flow, module layout | Engineers |
| 02 | [Product Roadmap](./02-roadmap.md) | Milestones M0–M7, scope, definition of done | Everyone |
| 03 | [Database Schema](./03-database-schema.md) | Full DDL, indexes, triggers, migrations | Backend |
| 04 | [API Specification](./04-api-specification.md) | Conventions, endpoints, examples, pagination | Frontend/CLI/API |
| 05 | [Coding Standards](./05-coding-standards.md) | Rust + TS conventions, git, reviews, testing | Engineers |
| 06 | [Deployment](./06-deployment.md) | Compose, config, backups, upgrades, troubleshooting | Ops/self-hosters |
| 07 | [Security & Backups](./07-security-and-backups.md) | Threat model, hardening, backup/restore | Ops/security |
| — | [Frontend Build Prompt](./frontend-build-prompt.md) | Self-contained spec to build the frontend | Frontend |
| — | [ADRs](./adr/) | Why we chose what we chose | Engineers |

## Architecture Decision Records

| ADR | Decision |
| --- | -------- |
| [0001](./adr/0001-rust-axum-backend.md) | Rust + Axum for the backend |
| [0002](./adr/0002-postgres-sqlx.md) | PostgreSQL + SQLx (raw SQL, compile-time checked) |
| [0003](./adr/0003-unified-entries-model.md) | Unified `entries` table with a `kind` enum |
| [0004](./adr/0004-cursor-pagination.md) | Cursor/keyset pagination |
| [0005](./adr/0005-no-authentication-local-first.md) | No authentication — local-first, single-user |
| [0006](./adr/0006-uuidv7-primary-keys.md) | UUIDv7 primary keys |

## Documentation conventions

- One topic per file; numbered for reading order.
- Keep the OpenAPI-derived spec and `04-api-specification.md` in sync.
- Prefer tables and short code blocks over prose.
- When a decision changes, write a **new** ADR that supersedes the old one (don't rewrite history).
