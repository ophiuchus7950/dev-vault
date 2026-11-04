import { useQuery } from '@tanstack/react-query'
import { Search as SearchIcon, SearchX } from 'lucide-react'
import { useEffect, useRef, useState } from 'react'
import { useNavigate, useSearchParams } from 'react-router-dom'
import { searchEntries } from '../../api/endpoints'
import { EmptyState } from '../../components/EmptyState'
import { RowSkeleton } from '../../components/Skeleton'
import { useDebounced } from '../../lib/hooks'
import { EntryCard } from '../entries/EntryCard'

export function SearchPage() {
  const [params, setParams] = useSearchParams()
  const navigate = useNavigate()
  const qParam = params.get('q') ?? ''
  const [text, setText] = useState(qParam)
  const inputRef = useRef<HTMLInputElement>(null)
  const debounced = useDebounced(text, 250)

  useEffect(() => {
    if (document.activeElement?.id !== 'search-page-input') setText(qParam)
  }, [qParam])

  useEffect(() => {
    const trimmed = debounced.trim()
    if (trimmed === qParam) return
    setParams(trimmed ? { q: trimmed } : {}, { replace: true })
  }, [debounced, qParam, setParams])

  const ready = qParam.trim().length >= 2
  const { data, isPending, isFetching } = useQuery({
    queryKey: ['search', qParam, 'page'],
    queryFn: () => searchEntries(qParam.trim(), { is_archived: false, limit: 30 }),
    enabled: ready,
  })

  const hits = data?.data.data ?? []

  return (
    <div className="mx-auto max-w-3xl px-8 py-8">
      <p className="eyebrow">Search</p>
      <h1 className="mt-2 font-display text-4xl font-semibold tracking-tight">Find anything</h1>
      <p className="mt-2 text-mute">Full-text and fuzzy matching — typos welcome.</p>

      <div className="relative mt-6">
        <SearchIcon
          size={16}
          className="pointer-events-none absolute top-1/2 left-3.5 -translate-y-1/2 text-faint"
        />
        <input
          id="search-page-input"
          ref={inputRef}
          value={text}
          placeholder="Search entries…"
          aria-label="Search entries"
          className="input py-3 pl-10 text-base"
          onChange={(e) => setText(e.target.value)}
        />
      </div>

      {ready ? (
        <div className="mt-6">
          <p className="mb-3 text-xs text-faint">
            {isPending
              ? 'Searching…'
              : `${hits.length} ${hits.length === 1 ? 'result' : 'results'}${isFetching ? ' · updating…' : ''}`}
          </p>
          {isPending ? (
            <div className="card divide-y divide-line/70">
              <RowSkeleton />
              <RowSkeleton />
            </div>
          ) : hits.length === 0 ? (
            <EmptyState
              icon={SearchX}
              title={`No matches for “${qParam.trim()}”`}
              hint="Try fewer words, or a rough spelling — fuzzy matching does the rest."
            />
          ) : (
            <div className="card overflow-hidden">
              {hits.map((hit) => (
                <EntryCard
                  key={hit.id}
                  entry={hit}
                  match={hit.matched}
                  onSelect={() => navigate(`/entries/${hit.id}`)}
                />
              ))}
            </div>
          )}
        </div>
      ) : (
        <p className="mt-6 text-sm text-faint">Type at least 2 characters to search.</p>
      )}
    </div>
  )
}
