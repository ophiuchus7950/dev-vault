# DevVault — Frontend Build Prompt

> **How to use this file:** paste everything below the line into your frontend coding agent/session. It is self-contained — it includes the stack, setup commands, folder structure, the full API contract, component inventory, state patterns, and acceptance criteria. No other context is required.

---

## ROLE

You are building the **frontend** of **DevVault**, a local-first, single-user developer knowledge manager (snippets, commands, notes, links, docs). The backend is a Rust/Axum REST API that already exists; you consume it over HTTP. There is **no authentication** and **no user concept** — it's a private, single-user tool. Build the SPA in `frontend/`.

Deliver a polished, keyboard-friendly, fast app that feels like a professional dev tool.

## STACK (fixed — do not substitute)

- **Vite + React 18 + TypeScript** (SPA, no SSR, no Next.js).
- **React Router** for routing.
- **TanStack Query v5** for ALL server state (queries, mutations, `useInfiniteQuery`).
- **Zustand** for UI-only state (theme, sidebar, filters, palette, toasts).
- **Tailwind CSS** + **shadcn/ui** (Radix primitives) for styling/components.
- **CodeMirror 6** for the snippet/command editor; **Shiki** for read-only highlighted rendering.
- **react-markdown** + `remark-gfm` + `rehype-sanitize` for notes.
- **Biome** for lint + format. **pnpm** package manager. **Vitest** + **React Testing Library**. **Playwright** for one smoke test.
- Icons: **lucide-react**. Class helper: `cn()` (clsx + tailwind-merge).

## SETUP

```bash
cd frontend
pnpm create vite . --template react-swc-ts      # if starting empty
pnpm add react-router-dom @tanstack/react-query zustand \
  tailwindcss @tailwindcss/vite postcss autoprefixer \
  cmdk lucide-react clsx tailwind-merge \
  codemirror @codemirror/state @codemirror/view @codemirror/language \
  @codemirror/lang-javascript @codemirror/lang-rust @codemirror/lang-python \
  @codemirror/lang-sql @codemirror/lang-json @codemirror/lang-markdown \
  shiki react-markdown remark-gfm rehype-sanitize rehype-highlight
pnpm add -D @biomejs/biome vitest @testing-library/react @testing-library/user-event \
  @testing-library/jest-dom jsdom msw @playwright/test
pnpm dlx shadcn@latest init
```

> If OpenAPI-generated types are available, generate them (`openapi-typescript`) into `src/api/types.gen.ts` instead of hand-writing types; otherwise hand-write the types in §TYPES below.

## PROJECT STRUCTURE

```
frontend/src/
├── main.tsx                 # mount + providers (QueryClient, Router, Theme, Toaster)
├── App.tsx                  # routes
├── api/
│   ├── client.ts            # typed fetch wrapper: base URL, params, error envelope
│   ├── types.gen.ts         # Entry, Tag, Collection, Page<T>, ApiError
│   └── endpoints.ts         # one function per API operation
├── features/
│   ├── entries/
│   │   ├── routes/{EntryList,EntryDetail,EntryEditor}.tsx
│   │   ├── components/{EntryCard,EntryKindBadge,TagChips,CodeViewer,CodeEditor,MarkdownView,MarkdownEditor,PinToggle,DeleteMenu}.tsx
│   │   └── hooks/{useEntries,useEntry,useCreateEntry,useUpdateEntry,useDeleteEntry}.ts
│   ├── search/   { routes/SearchResults, components/SearchBar,hooks/useSearch }
│   ├── tags/     { routes/TagsPage, components/TagCloud,TagInput,hooks/useTags }
│   ├── collections/ { routes/CollectionsPage,CollectionDetail, components/CollectionList, hooks/useCollections }
│   ├── trash/    { routes/TrashPage, hooks/useTrash }
│   └── portability/ { components/{ExportMenu,ImportDialog}, hooks/usePortability }
├── components/ui/           # shadcn primitives (button, input, dialog, ...)
├── components/              # app-wide: AppShell, Sidebar, Topbar, CommandPalette, ThemeToggle, EmptyState, Skeletons, Toaster
├── stores/                  # theme.ts, ui.ts, filters.ts, toast.ts
├── lib/                     # cn.ts, format.ts, api-error.ts, hotkeys.ts
└── styles/index.css         # Tailwind + CSS variables (themes)
```

One component per file; feature-first organisation (not type-first).

## API CONTRACT (implement exactly)

- **Base URL:** `/api/v1` (same-origin in prod; Vite dev proxy forwards `/api`, `/api-docs`, `/docs` to `http://localhost:3000`).
- **JSON `snake_case`**, timestamps are RFC3339 UTC strings, IDs are UUIDv7 strings.
- **Error envelope** (all non-2xx):
  ```json
  { "error": { "code": "VALIDATION_ERROR", "message": "…",
    "details": [{ "field": "title", "code": "length", "message": "…" }] } }
  ```
  The client wrapper parses this into a typed `ApiError` and surfaces `details` on form fields.
- **Concurrency:** mutable resources return `ETag`; `PATCH` requires `If-Match`. On `409`, refetch and retry once, then show a toast.
- **Pagination:** cursor-based. Response `{ data: T[], next_cursor: string|null, has_more: boolean }`. Use `useInfiniteQuery` with `getNextPageParam: (last) => last.next_cursor ?? undefined`.

### Endpoints to wire up

| Method + path | Purpose |
| ------------- | ------- |
| `GET /health` | optional status widget |
| `GET /entries` | list + filters + sort + cursor |
| `POST /entries` | create (201 + Location) |
| `GET /entries/{id}` | detail (ETag) |
| `PATCH /entries/{id}` | update (requires `If-Match`) |
| `DELETE /entries/{id}` | soft delete → Trash (204) |
| `POST /entries/{id}/restore` | restore from Trash |
| `DELETE /entries/{id}/purge` | permanent delete from Trash |
| `GET /trash` | list trashed entries |
| `GET /tags` | tags + counts (`?q=` prefix, `?sort=name\|count`) |
| `PATCH /tags/{id}` | rename tag |
| `DELETE /tags/{id}` | delete tag |
| `GET/POST /collections` | list/create collections |
| `GET/PATCH/DELETE /collections/{id_or_slug}` | manage a collection |
| `POST /collections/{id}/entries` | assign entries |
| `DELETE /collections/{id}/entries` | unassign entries |
| `GET /search` | `?q=&mode=hybrid\|fts\|trgm` + all filters |
| `GET /entries/{id}/export?format=md\|json` | export one entry |
| `GET /export?format=json\|zip` | export whole vault |
| `POST /import?on_conflict=skip\|overwrite` | import a vault |

### Query parameters (list & search)

`kind` (snippet|command|note|link|doc) · `language` · `tag` (repeatable, AND) · `collection` · `is_pinned` · `is_archived` · `sort` (updated_at|created_at|title|relevance) · `order` (asc|desc) · `limit` (1–100, default 20) · `cursor`. Search adds `q` (required, ≥2 chars) and `mode`.

## TYPES

```ts
export type EntryKind = 'snippet' | 'command' | 'note' | 'link' | 'doc';

export interface Entry {
  id: string;
  kind: EntryKind;
  title: string;
  body: string;
  language: string | null;
  shell: string | null;
  url: string | null;
  tags: string[];
  collections: string[];      // collection slugs
  is_pinned: boolean;
  is_archived: boolean;
  created_at: string;         // RFC3339 UTC
  updated_at: string;
  deleted_at: string | null;
}

export interface Tag { id: string; name: string; count: number; }
export interface Collection {
  id: string; name: string; slug: string; description: string; entry_count: number;
}
export interface Page<T> { data: T[]; next_cursor: string | null; has_more: boolean; }
export interface SearchHit extends Entry { score: number; match: 'ts' | 'trgm' | 'hybrid'; }
export interface ApiError {
  error: { code: string; message: string;
    details?: { field: string; code: string; message: string }[] };
}
export type EntryDraft = Partial<Omit<Entry, 'id'|'created_at'|'updated_at'|'deleted_at'>> & {
  kind: EntryKind; title: string;
};
```

## ROUTES

| Path | View |
| ---- | ---- |
| `/` | Entry list (default: updated_at desc, live only) with filter bar + sidebar |
| `/entries/new` | Create editor (kind picker) |
| `/entries/:id` | Detail view (rendered) |
| `/entries/:id/edit` | Edit (editor) |
| `/search` | Search results (uses `q` in URL) |
| `/collections` | Collections index |
| `/collections/:slug` | Collection detail (filtered list) |
| `/tags` | Tag cloud + management |
| `/trash` | Trashed entries (restore/purge) |
| `/settings` | Theme, import/export, about |

Use a persistent `AppShell` (sidebar + topbar + `<Outlet/>`). 404 route → friendly empty state.

## FEATURE DETAILS

**Entry list**
- Infinite scroll (or "Load more") via `useInfiniteQuery`.
- Filter bar: kind tabs/select, tag multi-select, collection select, pinned/archived toggles, sort control.
- Filter state lives in the URL (`useSearchParams`) so views are shareable/back-button-friendly, mirrored into the Zustand `filters` store for the palette.
- Each `EntryCard`: kind badge, title, language/shell chip, tag chips, snippet preview (Shiki, clamped), pin toggle, overflow menu (edit, copy, export, delete). Optimistic pin/delete with rollback.

**Entry editor**
- Kind selector switches the body editor: CodeMirror (snippet/command) with language/theme, or Markdown editor with live preview split-pane (notes/docs).
- Fields: title, body/url/shell/language as appropriate, tags (typeahead create/select), collections (multi-select), pinned, archived.
- Create → navigate to detail on 201. Edit → send `If-Match` from the loaded `ETag`; on `409`, refetch + toast.
- `⌘S` / `Ctrl+S` submits. Warn on unsaved navigation.

**Detail view**
- Render body: Shiki for code (with copy button, language label, line numbers), react-markdown for notes (sanitised), a link card for `link` (external icon), a terminal-style block for `command`.
- Show metadata (kind, tags, collections, timestamps), edit/delete/export actions.

**Search**
- Topbar search box, debounced ~250ms, updates `/search?q=`.
- Show which branch matched (`ts`/`trgm`/`hybrid`) subtly; highlight matched terms in titles.
- Respect all filters; empty state suggests recent/pinned items.

**Tags**
- Tag cloud sized by count; click filters the list.
- Rename (inline), delete (with confirmation; cascades from entries).
- `TagInput`: combobox with existing tags + "create new".

**Collections**
- Sidebar lists collections; index page shows cards with counts.
- Create/rename/delete; assign/unassign entries (multi-select on list view).

**Trash**
- Lists `deleted_at`-ordered entries; per-item Restore and Purge (purge confirms).

**Import / Export**
- Export menu: current entry (MD/JSON), whole vault (JSON/ZIP).
- Import dialog: file picker → POST → show result summary (`imported/skipped/overwritten`) and errors; choices for `on_conflict`.

**Command palette (`Cmd/Ctrl-K`)**
- Built with `cmdk`. Actions: new snippet/command/note/link, go to collections/tags/trash/settings, toggle theme, export vault.
- Fuzzy jump to entries (search API) and collections.
- Global shortcuts: `n` new, `/` focus search, `⌘S` save, `Esc` close, `?` shortcut help.

**Theme**
- Light/dark/system; persisted in Zustand + `localStorage`-free (use a store; in sandboxed envs keep it in memory). Apply via a `dark` class on `<html>` and CSS variables.
- Respect `prefers-color-scheme` for the `system` option.

## STATE PATTERNS (enforce)

- **Never** put server data in Zustand/`useState`. Server data → TanStack Query.
- Centralise query keys: `['entries', {filters, sort}]`, `['entry', id]`, `['search', {q, filters}]`, `['tags', {q}]`, `['collections']`, `['trash']`.
- Mutations: optimistic for pin/archive/delete/tag-attach; invalidate affected keys on settle.
- Zustand stores: `theme`, `ui` (sidebar, palette open), `filters` (mirror of URL), `toast`.

## ACCESSIBILITY

- Keyboard reachable everywhere; visible focus rings (`focus-visible`).
- Modals/palette trap focus and restore on close; `Esc` closes.
- Labels on inputs; `aria-label` on icon buttons; `role="status"` on toasts.
- Respect `prefers-reduced-motion`. Don't rely on colour alone.

## PERFORMANCE

- Lazy-load `CodeMirror`, `Shiki`, and Import/Export dialogs (`React.lazy` + `Suspense`).
- Route-level code splitting. Initial JS < 300 KB gz.
- Shiki: load grammars/themes on demand; cache rendered HTML.
- Virtualise very long lists if needed (optional).

## TESTING

- **Vitest + RTL**: `EntryCard`, `TagInput`, filter bar, `EntryEditor` validation + `⌘S`, optimistic pin rollback. Mock the network with **MSW** (not module stubs).
- **Playwright** smoke: load list → create a snippet → search for it (incl. a typo) → pin it → export JSON.
- `pnpm biome check .`, `pnpm tsc --noEmit`, `pnpm test` must pass.

## VITE DEV PROXY

```ts
// vite.config.ts
server: {
  proxy: {
    '/api':      { target: 'http://localhost:3000', changeOrigin: true },
    '/api-docs': { target: 'http://localhost:3000', changeOrigin: true },
    '/docs':     { target: 'http://localhost:3000', changeOrigin: true },
  }
}
```

Use same-origin `/api/v1` paths everywhere (no absolute hosts), so dev and prod behave identically.

## ACCEPTANCE CRITERIA

- [ ] Create/edit/view/delete/restore/purge all five entry kinds end-to-end.
- [ ] Snippet/command editor has live syntax highlighting; rendered code has a copy button.
- [ ] Notes render Markdown with GFM + code highlighting, sanitised.
- [ ] Search works, including typos (hybrid mode), and shows results within ~300 ms of typing settling.
- [ ] Filters (kind/language/tag/collection/pinned/archived) + sort work and are reflected in the URL.
- [ ] Infinite scroll paginates via `next_cursor` with no duplicates.
- [ ] Pin/archive/delete feel instant (optimistic) and roll back cleanly on error.
- [ ] Import/export (entry + vault, JSON + MD) work and report result counts.
- [ ] `Cmd/Ctrl-K` palette and global shortcuts work; app is fully keyboard-navigable.
- [ ] Light/dark/system themes work; no flash of incorrect theme.
- [ ] No `any`; `tsc --noEmit` and `biome check` pass; Vitest + Playwright pass.
- [ ] Lighthouse a11y ≥ 95 on the list and detail views.

**Out of scope:** authentication/login (there is none), multi-user, public sharing, mobile-native, AI features.
