# Contributing to DevVault

Thanks for wanting to help. DevVault is a solo-friendly project, but the process below keeps it tidy and reviewable.

By participating you agree to keep discussions respectful and technical. There is no CLA; contributions are accepted under the [MIT license](./LICENSE).

---

## 1. Prerequisites

| Tool            | Version   | Notes                                        |
| --------------- | --------- | -------------------------------------------- |
| Rust            | 1.79+     | `rustup` recommended                          |
| Docker          | 24+       | For Postgres and the full stack               |
| Node.js         | 20+       | Frontend                                      |
| pnpm            | 9+        | `corepack enable && corepack use pnpm@9`      |
| sqlx-cli        | 0.7+      | `cargo install sqlx-cli --no-default-features --features postgres` |
| just (optional) | latest    | Task runner for aliases                       |

## 2. Local setup

```bash
git clone <your-fork> dev-vault
cd dev-vault
cp .env.example .env

# start just Postgres for local dev
docker compose up -d postgres

# backend
cd backend
cargo sqlx migrate run
cargo watch -x run

# frontend (separate terminal)
cd frontend
pnpm install
pnpm dev
```

## 3. Branch & commit conventions

We use **trunk-based development** with short-lived branches and **Conventional Commits** (see [Coding standards § Git](./docs/05-coding-standards.md#git-workflow)).

- Branch: `feat/<slug>`, `fix/<slug>`, `docs/<slug>`, `chore/<slug>`, `refactor/<slug>`.
- Keep branches short-lived; rebase on `main` before opening a PR.
- Commit messages: `<type>(<scope>): <subject>` — e.g. `feat(search): add trigram fallback`.

Allowed types: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`, `revert`.

## 4. Before you push

Run the full local gate — it mirrors CI:

```bash
just check          # or run the two commands below manually
```

```bash
# backend
cd backend
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test

# frontend
cd frontend
pnpm biome check .
pnpm tsc --noEmit
pnpm test
```

## 5. Pull requests

- One logical change per PR; keep them small.
- Fill in the PR template: what changed, why, how you tested it.
- Link the relevant issue or roadmap milestone.
- Include/update tests. Bug fixes should add a regression test.
- Update docs when behaviour or the API contract changes (`docs/` and the OpenAPI derives).
- CI must be green; resolve review comments or reply with reasoning.

### PR checklist

- [ ] Commits follow Conventional Commits.
- [ ] `cargo fmt`, `clippy -D warnings`, and `cargo test` pass.
- [ ] `biome check`, `tsc --noEmit`, `pnpm test` pass.
- [ ] New/changed endpoints are reflected in utoipa derives and `docs/04-api-specification.md`.
- [ ] Migrations are additive and reversible where practical.
- [ ] No secrets, tokens, or `.env` files committed.

## 6. Database changes

- Add a new timestamped file under `backend/migrations/` (`sqlx migrate add <name>`); never edit a migration that has shipped.
- Hand-write SQL (we do not use a schema-diff tool).
- Add indexes deliberately; justify GIN/GiST/composite choices in the PR.
- Regenerate sqlx offline metadata if you use `SQLX_OFFLINE`: `cargo sqlx prepare`.

## 7. Coding standards

Read [`docs/05-coding-standards.md`](./docs/05-coding-standards.md) before contributing. Highlights:

- Rust: `thiserror` for domain errors, no `unwrap()`/`expect()` outside tests and startup, `tracing` for logs.
- TypeScript: `strict` mode, no `any`, TanStack Query for all server state, Zustand only for UI state.
- Accessibility: interactive elements keyboard-reachable, labels on everything.

## 8. Reporting bugs / requesting features

Open an issue with: what you expected, what happened, reproduction steps, environment (OS, Docker/Node/Rust versions), and logs where relevant. For security issues see [`docs/07-security-and-backups.md`](./docs/07-security-and-backups.md#reporting-a-vulnerability).

## 9. Release process (maintainers)

1. Ensure `main` is green and the roadmap milestone is complete.
2. `release-please` (or manual) bumps the version and updates `CHANGELOG.md` from Conventional Commits.
3. Tag `vX.Y.Z`; CI publishes images/binaries.
4. Update `docs/02-roadmap.md` "shipped" status.
