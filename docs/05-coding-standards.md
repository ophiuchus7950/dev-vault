# DevVault — Coding Standards

- **Status:** Draft v1.0
- **Last updated:** 2026-11-04
- **Applies to:** `backend/` (Rust), `frontend/` (TypeScript/React), `cli/` (Rust)
- **Related:** [Architecture](./01-architecture.md) · [CONTRIBUTING](../CONTRIBUTING.md)

These are **requirements**, not suggestions. CI enforces most of them; reviewers enforce the rest.

---

## 1. General principles

- **Optimise for the reader.** Code is read far more than written.
- **Small, focused units.** Functions do one thing; modules have one reason to change.
- **Explicit over clever.** No magic, no hidden side effects.
- **Fail loudly at the boundary, trust internals.** Validate user input; trust internal guarantees.
- **No dead code.** Delete unused code outright — no commented-out blocks, no `_unused` shims, no "just in case".
- **Comments explain *why*, never *what*.** If the *what* isn't obvious, rename instead of commenting.
- **No new dependency without a reason** in the PR description.

---

## 2. Rust (backend + CLI)

### 2.1 Formatting & lints

- `rustfmt` with default settings: `cargo fmt --all`.
- `clippy` with warnings denied:

```toml
# Cargo.toml [lints] (or clippy.toml)
[lints.rust]
unsafe_code = "forbid"
[lints.clippy]
all = "warn"
pedantic = "warn"
unwrap_used = "warn"
expect_used = "warn"
```

CI runs: `cargo fmt --all --check` and `cargo clippy --all-targets --all-features -- -D warnings`.

- `#![deny(unsafe_code)]` at the crate root. There is no reason for `unsafe` in this project.
- `unwrap()`/`expect()` are forbidden outside tests and **startup** code (where a missing config *should* abort the process). Prefer `?`, `ok_or`, or explicit match.

### 2.2 Error handling

- **Domain errors:** a single `thiserror` enum per layer boundary; `AppError` implements `IntoResponse`.

```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{resource} not found")]
    NotFound { resource: &'static str },
    #[error("validation failed")]
    Validation(ValidationErrors),
    #[error("conflict: {reason}")]
    Conflict { reason: String },
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}
```

- `anyhow` only at the binary/edge (startup, CLI) for context-rich chains. Libraries/modules use typed errors.
- Never construct HTTP types (`StatusCode`, `Response`) outside `error.rs`/handlers.
- `From<sqlx::Error>` distinguishes `RowNotFound` (→ 404) from everything else (→ 500, logged with the request span).

### 2.3 Layering (enforced by review)

```
routes/   → parse + validate + serialise only, call service
service/  → business rules, transactions, orchestration
repo/     → ALL SQL, returns domain types; never HTTP types
domain/   → plain structs/enums; no framework imports
dto/      → request/response shapes + validator attributes + utoipa ToSchema
```

- Handlers must not contain SQL. Repos must not contain business rules.

### 2.4 Naming

| Item | Convention | Example |
| ---- | ---------- | ------- |
| Types/traits/enums | `UpperCamelCase` | `EntryKind`, `EntryService` |
| Functions/methods/vars | `snake_case` | `list_entries`, `next_cursor` |
| Constants | `SCREAMING_SNAKE_CASE` | `MAX_PAGE_SIZE` |
| Modules/files | `snake_case` | `entries.rs`, `pagination.rs` |
| Enum variants | `UpperCamelCase` | `Kind::Snippet` |

Booleans read as predicates: `is_pinned`, `has_more`, `can_purge`.

### 2.5 SQL with SQLx

- Always `sqlx::query_as!`/`query!` (compile-time checked) — never runtime string queries.
- **All values parameterised** (`$1`, `$2`) or bound via the macro. Dynamic identifiers (sort column, direction) map through a match to a fixed `&'static str` — never string-interpolated.
- Multi-step writes (create entry + tags + collections) run inside a transaction (`pool.begin()`).
- Enforce `deleted_at IS NULL` by default in the repository; expose explicit `include_deleted`/trash functions.
- Keep offline metadata fresh for CI: `cargo sqlx prepare` and commit `.sqlx/` (or run with a live DB in CI).

### 2.6 Async

- Tokio multi-thread runtime; handlers are `async fn`.
- Never block the executor: no `std::fs`, no sync DB, no CPU-heavy work in handlers. Use `tokio::task::spawn_blocking` for anything blocking.
- Bound concurrency where fan-out exists.

### 2.7 API documentation (utoipa)

- Every route has a `#[utoipa::path]`; every DTO derives `ToSchema`.
- Register in the `#[derive(OpenApi)]` aggregate so `/docs` stays complete.
- Keep `docs/04-api-specification.md` in sync whenever a route or shape changes.

### 2.8 Rust testing

- Unit tests live in `#[cfg(test)] mod tests` beside the code; integration tests in `backend/tests/`.
- Integration tests hit a **real ephemeral Postgres** (Compose service or `testcontainers`); each test runs in a transaction that is rolled back, or on a uniquely-named schema.
- Test names describe behaviour: `rejects_link_without_url`, `paginates_without_duplicates_under_inserts`.
- Prefer table-driven tests for validation edge cases.

```rust
#[tokio::test]
async fn rejects_link_without_url() {
    let app = test_app().await;
    let res = app.post("/api/v1/entries").json(&json!({"kind":"link","title":"x"})).await;
    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}
```

---

## 3. TypeScript / React (frontend)

### 3.1 Tooling

- **Biome** for lint + format (single tool, replaces ESLint + Prettier). `pnpm biome check .`.
- **pnpm** as the package manager.
- `tsconfig`:
  ```jsonc
  {
    "compilerOptions": {
      "strict": true,
      "noUncheckedIndexedAccess": true,
      "noImplicitOverride": true,
      "verbatimModuleSyntax": true,
      "noFallthroughCasesInSwitch": true
    }
  }
  ```
- No `any`. Use `unknown` + narrowing. `@ts-expect-error` requires a comment justifying it.

### 3.2 Structure

```
frontend/src/
├── main.tsx
├── App.tsx                 # router + providers
├── api/                    # generated types + typed fetch client
│   ├── client.ts
│   └── types.gen.ts        # from OpenAPI (do not hand-edit)
├── features/               # feature-first folders
│   ├── entries/
│   │   ├── components/
│   │   ├── hooks/          # TanStack Query hooks
│   │   └── routes/
│   ├── tags/
│   ├── collections/
│   ├── search/
│   └── portability/
├── components/ui/          # shadcn/ui primitives
├── stores/                 # Zustand stores
├── lib/                    # pure helpers
└── styles/
```

- **Feature-first**, not type-first. A feature owns its components, hooks and routes.
- One component per file; file name matches the component (`EntryCard.tsx`).

### 3.3 State management

- **All server data goes through TanStack Query.** Never store server data in Zustand or `useState` (beyond form drafts).
- Query keys are structured and centralised: `['entries', { filters, cursor }]`, `['entry', id]`, `['tags']`.
- Mutations invalidate the precise keys they affect (or use optimistic updates) — never blanket-invalidate everything.
- **Zustand is for UI state only**: theme, sidebar, active filters, palette open, toasts.
- Derive, don't duplicate: compute values from props/query data instead of mirroring them in state.

### 3.4 Components

- Function components + hooks only. No class components.
- Props typed with an explicit `type Props = { ... }`; no `React.FC`.
- Keep components presentational and pure; push data access into hooks (`useEntries`, `useCreateEntry`).
- Server/local state split: hooks own the data, components render it.
- Wrap expensive/lazy views (`CodeMirror`, `Shiki`, export/import dialogs) in `React.lazy` + `Suspense`.

### 3.5 Styling

- **Tailwind** utilities + **shadcn/ui** (Radix) primitives.
- No ad-hoc CSS files except for global tokens; no inline `style` except dynamic values (e.g. editor height).
- Theme via CSS custom properties (`--background`, `--foreground`, `--primary`, …) so light/dark is a class toggle.
- Conditional classes via a `cn()` helper (clsx + tailwind-merge).

### 3.6 Accessibility

- Every interactive element is keyboard reachable and focus-visible.
- Inputs have labels; icons-only buttons have `aria-label`.
- Modals/palettes trap focus and restore it on close; `Esc` closes.
- Respect `prefers-reduced-motion`; never convey state by colour alone.

### 3.7 TypeScript testing

- **Vitest** + **React Testing Library**; query by role/text, not test ids where possible.
- Test behaviour, not implementation: "clicking pin toggles optimistic state", not internal hook calls.
- **Playwright** for a smoke path: create → search → pin → export.
- Mock the API at the network layer (MSW) rather than stubbing modules.

---

## 4. Git workflow

- **Trunk-based.** `main` is always green and deployable. Work in short-lived branches, squash-merge via PR.
- **Branch names:** `feat/<slug>`, `fix/<slug>`, `docs/<slug>`, `chore/<slug>`, `refactor/<slug>`, `perf/<slug>`, `test/<slug>`.
- **Conventional Commits** (drives changelog/versioning):

```
<type>(<scope>): <subject>

[optional body]

[optional footer(s)]
```

| Type | Use |
| ---- | --- |
| `feat` | user-facing feature |
| `fix` | bug fix |
| `docs` | documentation only |
| `refactor` | no behaviour change |
| `perf` | performance |
| `test` | tests only |
| `build` | build system / deps |
| `ci` | CI config |
| `chore` | misc |
| `style` | formatting only |
| `revert` | revert |

Examples: `feat(search): add trigram fallback`, `fix(pagination): tie-break cursor on id`, `docs(api): document If-Match`.

- Scope is the module/area (`search`, `entries`, `tags`, `collections`, `ui`, `api`, `db`).
- Subject in imperative mood, lower-case, no trailing period, ≤ 72 chars.
- **Breaking changes:** add `!` after type/scope and a `BREAKING CHANGE:` footer.
- Rebase on `main` before review; keep history linear (squash-merge).

### Commit checklist

- [ ] Builds and tests pass locally (`just check`).
- [ ] No secrets, no `.env`, no large binaries.
- [ ] Conventional Commit message.
- [ ] Docs/OpenAPI updated if behaviour changed.

---

## 5. Code review guidelines

**Authors**
- Keep PRs small and single-purpose; describe *what* and *why* in the description.
- Self-review the diff before requesting review; leave context comments on tricky spots.

**Reviewers**
- Check correctness, tests, security (parameterised SQL, sanitised Markdown), and readability.
- Verify the layering rules (§2.3) and the state-management rules (§3.3).
- Distinguish blocking issues from nits; label nits clearly.
- Prefer asking a question over asserting when unsure.

**Merge criteria:** CI green, at least one approval, all conversations resolved, PR checklist complete.

## 6. Definition of Done (per change)

- [ ] Meets the relevant PRD requirement(s)/acceptance criteria.
- [ ] Tests added/updated and passing (unit + integration where relevant).
- [ ] `cargo fmt`/`clippy -D warnings`/`cargo test` pass (backend + CLI).
- [ ] `biome check`/`tsc --noEmit`/`pnpm test` pass (frontend).
- [ ] No new warnings, no dead code, no `unwrap()` in non-test code.
- [ ] API changes reflected in utoipa + `docs/04-api-specification.md`.
- [ ] Docs/CHANGELOG updated when user-visible.
- [ ] Accessibility considered for any UI change.

## 7. Dependency policy

- Prefer the standard library / existing deps over new crates/packages.
- Every new dependency is justified in the PR and must be actively maintained, permissively licensed, and reasonably small.
- Run `cargo audit` and `pnpm audit` in CI; address advisories promptly.
- Pin major versions; let Dependabot/Renovate propose updates; do not auto-merge major bumps.
