import { Layers, Plus } from 'lucide-react'
import { useEffect, useRef } from 'react'
import { useNavigate, useParams, useSearchParams } from 'react-router-dom'
import type { EntryListParams } from '../../api/types'
import { EmptyState } from '../../components/EmptyState'
import { RowSkeleton } from '../../components/Skeleton'
import { cn } from '../../lib/cn'
import type { EntryKind } from '../../lib/kinds'
import { EntryCard } from './EntryCard'
import { EntryDetailPane } from './EntryDetailPane'
import { useEntries, useTogglePin } from './hooks'

const TABS: { value: EntryKind | 'all'; label: string }[] = [
  { value: 'all', label: 'All' },
  { value: 'snippet', label: 'Snippets' },
  { value: 'note', label: 'Notes' },
  { value: 'link', label: 'Links' },
  { value: 'command', label: 'Commands' },
  { value: 'doc', label: 'Docs' },
]

const SORTS = [
  { value: 'updated_at:desc', label: 'Recently updated' },
  { value: 'created_at:desc', label: 'Recently created' },
  { value: 'updated_at:asc', label: 'Oldest updated' },
  { value: 'title:asc', label: 'Title A–Z' },
  { value: 'title:desc', label: 'Title Z–A' },
] as const

interface EntryListPageProps {
  variant?: 'all' | 'starred'
}

export function EntryListPage({ variant = 'all' }: EntryListPageProps) {
  const { id: selectedId } = useParams()
  const navigate = useNavigate()
  const [params, setParams] = useSearchParams()
  const togglePin = useTogglePin()

  const kind = (params.get('kind') as EntryKind | null) ?? 'all'
  const sortPair = params.get('sort') ?? 'updated_at:desc'
  const tag = params.get('tag')

  const filters: EntryListParams = {
    kind: kind === 'all' ? undefined : kind,
    is_pinned: variant === 'starred' ? true : undefined,
    tag: tag ? [tag] : undefined,
    is_archived: false,
    sort: (sortPair.split(':')[0] as EntryListParams['sort']) ?? 'updated_at',
    order: (sortPair.split(':')[1] as 'asc' | 'desc') ?? 'desc',
    limit: 20,
  }

  const query = useEntries(filters)
  const entries = query.data?.pages.flatMap((p) => p.data.data) ?? []

  const sentinelRef = useRef<HTMLDivElement>(null)
  useEffect(() => {
    const el = sentinelRef.current
    if (!el || !query.hasNextPage || query.isFetchingNextPage) return
    const observer = new IntersectionObserver(
      (events) => {
        if (events[0]?.isIntersecting) query.fetchNextPage()
      },
      { rootMargin: '240px' },
    )
    observer.observe(el)
    return () => observer.disconnect()
  }, [query.hasNextPage, query.isFetchingNextPage, query.fetchNextPage])

  const setParam = (key: string, value: string | null) => {
    const next = new URLSearchParams(params)
    if (value === null || value === 'all') next.delete(key)
    else next.set(key, value)
    setParams(next, { replace: true })
  }

  const heading = variant === 'starred' ? 'Starred' : 'All entries'
  const sub =
    variant === 'starred'
      ? 'Everything you have starred, in one pile.'
      : 'Everything worth keeping, one place to find it.'

  return (
    <div className="mx-auto max-w-6xl px-8 py-8">
      <p className="eyebrow">Your personal knowledge base</p>
      <h1 className="mt-2 font-display text-4xl font-semibold tracking-tight">{heading}</h1>
      <p className="mt-2 text-mute">{sub}</p>

      <div className="mt-7 grid items-start gap-5 lg:grid-cols-[minmax(0,1fr)_400px]">
        <section className="card overflow-hidden">
          <div className="flex items-center justify-between gap-3 border-b border-line/70 p-3">
            <div className="flex gap-1 overflow-x-auto">
              {TABS.map((t) => (
                <button
                  key={t.value}
                  type="button"
                  onClick={() => setParam('kind', t.value)}
                  aria-pressed={kind === t.value}
                  className={cn(
                    'cursor-pointer rounded-lg px-3 py-1.5 text-sm whitespace-nowrap transition-colors',
                    kind === t.value ? 'bg-panel2 text-ink' : 'text-mute hover:text-ink',
                  )}
                >
                  {t.label}
                </button>
              ))}
            </div>
            <select
              value={sortPair}
              onChange={(e) => setParam('sort', e.target.value === 'updated_at:desc' ? null : e.target.value)}
              aria-label="Sort entries"
              className="input w-auto py-1.5 pr-8 text-xs"
            >
              {SORTS.map((s) => (
                <option key={s.value} value={s.value}>
                  {s.label}
                </option>
              ))}
            </select>
          </div>

          {tag ? (
            <div className="flex items-center gap-2 border-b border-line/70 px-4 py-2 text-sm text-mute">
              Filtering by
              <span className="chip chip-accent">#{tag}</span>
              <button
                type="button"
                onClick={() => setParam('tag', null)}
                className="cursor-pointer text-faint hover:text-ink"
              >
                clear
              </button>
            </div>
          ) : null}

          {query.isPending ? (
            <div className="divide-y divide-line/70">
              <RowSkeleton />
              <RowSkeleton />
              <RowSkeleton />
            </div>
          ) : entries.length === 0 ? (
            <EmptyState
              icon={Layers}
              title={variant === 'starred' ? 'Nothing starred yet' : 'Nothing here yet'}
              hint={
                tag
                  ? `No live entries are tagged “${tag}”.`
                  : 'Create your first entry — snippets, commands, notes, links or docs.'
              }
              action={
                <button type="button" className="btn btn-accent" onClick={() => navigate('/entries/new')}>
                  <Plus size={15} />
                  New entry
                </button>
              }
            />
          ) : (
            <div>
              {entries.map((e) => (
                <EntryCard
                  key={e.id}
                  entry={e}
                  selected={e.id === selectedId}
                  onSelect={() => navigate(`/entries/${e.id}`)}
                  onTogglePin={() => togglePin.mutate(e)}
                />
              ))}
            </div>
          )}

          <div ref={sentinelRef} aria-hidden />
          {query.hasNextPage ? (
            <button
              type="button"
              className="btn btn-ghost w-full rounded-none border-t border-line/70"
              onClick={() => query.fetchNextPage()}
              disabled={query.isFetchingNextPage}
            >
              {query.isFetchingNextPage ? 'Loading…' : 'Load more'}
            </button>
          ) : null}
        </section>

        <EntryDetailPane id={selectedId} className="lg:sticky lg:top-0" />
      </div>
    </div>
  )
}
