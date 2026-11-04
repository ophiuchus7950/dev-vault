import { Star } from 'lucide-react'
import type { Entry, SearchMatch } from '../../api/types'
import { KindIcon } from '../../components/KindIcon'
import { cn } from '../../lib/cn'
import { titleCase } from '../../lib/format'
import { kindLabel } from '../../lib/kinds'

interface EntryCardProps {
  entry: Entry
  selected?: boolean
  onSelect: () => void
  onTogglePin?: () => void
  match?: SearchMatch
}

function preview(entry: Entry): string {
  const text = entry.body.trim() || entry.url || ''
  return text.length > 220 ? `${text.slice(0, 220)}…` : text
}

export function EntryCard({ entry, selected, onSelect, onTogglePin, match }: EntryCardProps) {
  return (
    <div className={cn('group relative border-t border-line/70 first:border-t-0', selected && 'bg-panel2')}>
      {selected ? <span className="absolute inset-y-0 left-0 w-0.5 bg-accent" aria-hidden /> : null}
      <button
        type="button"
        onClick={onSelect}
        aria-current={selected}
        className="w-full cursor-pointer p-4 pr-12 text-left focus-visible:-outline-offset-2 focus-visible:outline-2 focus-visible:outline-accent"
      >
        <div className="flex items-start gap-3">
          <KindIcon kind={entry.kind} />
          <div className="min-w-0 flex-1">
            <p className="truncate text-[15px] font-semibold text-ink">{entry.title}</p>
            <p className="mt-1 line-clamp-2 text-sm text-mute">{preview(entry)}</p>
            <div className="mt-2.5 flex flex-wrap items-center gap-1.5">
              <span className="chip">{kindLabel(entry.kind)}</span>
              {entry.language ? <span className="chip">{titleCase(entry.language)}</span> : null}
              {entry.shell ? <span className="chip">{entry.shell}</span> : null}
              {entry.tags.slice(0, 3).map((t) => (
                <span key={t} className="chip">
                  {t}
                </span>
              ))}
              {match ? <span className="chip chip-accent font-mono">{match}</span> : null}
            </div>
          </div>
        </div>
      </button>
      {onTogglePin ? (
        <button
          type="button"
          onClick={onTogglePin}
          aria-label={entry.is_pinned ? 'Unstar entry' : 'Star entry'}
          title={entry.is_pinned ? 'Unstar' : 'Star'}
          className={cn(
            'absolute top-3.5 right-3 cursor-pointer rounded-md p-1.5 text-faint transition-opacity hover:text-accent focus-visible:outline-2 focus-visible:outline-accent',
            entry.is_pinned ? 'text-accent opacity-100' : 'opacity-0 group-hover:opacity-100',
          )}
        >
          <Star size={15} className={entry.is_pinned ? 'fill-accent' : ''} />
        </button>
      ) : null}
    </div>
  )
}
