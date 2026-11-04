import { useQueryClient } from '@tanstack/react-query'
import { CircleAlert, Copy, ExternalLink, Link2, MousePointerClick, Star, Trash2 } from 'lucide-react'
import { Link, useNavigate } from 'react-router-dom'
import type { Entry } from '../../api/types'
import { CodeBlock } from '../../components/CodeBlock'
import { EmptyState } from '../../components/EmptyState'
import { MarkdownView } from '../../components/MarkdownView'
import { Skeleton } from '../../components/Skeleton'
import { TagChips } from '../../components/TagChips'
import { cn } from '../../lib/cn'
import { fullDate, relativeTime, titleCase } from '../../lib/format'
import { kindLabel } from '../../lib/kinds'
import { toast } from '../../stores/toast'
import { useEntry, useSoftDeleteEntry, useTogglePin } from './hooks'
import { entryKeys } from './keys'

function LinkCard({ entry }: { entry: Entry }) {
  return (
    <div>
      {entry.url ? (
        <a
          href={entry.url}
          target="_blank"
          rel="noreferrer"
          className="card flex items-center gap-3 p-3 hover:border-linestrong"
        >
          <Link2 size={16} className="shrink-0 text-accent" />
          <span className="min-w-0 flex-1 truncate text-sm">{entry.url}</span>
          <ExternalLink size={14} className="shrink-0 text-faint" />
        </a>
      ) : null}
      {entry.body ? (
        <div className="mt-3">
          <MarkdownView content={entry.body} />
        </div>
      ) : null}
    </div>
  )
}

interface EntryDetailPaneProps {
  id?: string
  className?: string
}

export function EntryDetailPane({ id, className }: EntryDetailPaneProps) {
  const navigate = useNavigate()
  const qc = useQueryClient()
  const { data, isPending, isError } = useEntry(id)
  const togglePin = useTogglePin()
  const softDelete = useSoftDeleteEntry()

  if (!id) {
    return (
      <aside className={cn('card hidden lg:block', className)}>
        <EmptyState
          icon={MousePointerClick}
          title="Select an entry"
          hint="Pick something from the list to see it rendered here."
          className="h-full"
        />
      </aside>
    )
  }

  if (isPending) {
    return (
      <aside className={cn('card space-y-3 p-5', className)}>
        <Skeleton className="h-3 w-28" />
        <Skeleton className="h-7 w-3/4" />
        <Skeleton className="h-3 w-20" />
        <Skeleton className="mt-4 h-44 w-full" />
      </aside>
    )
  }

  if (isError || !data) {
    return (
      <aside className={cn('card', className)}>
        <EmptyState icon={CircleAlert} title="Entry not found" hint="It may have been deleted." />
      </aside>
    )
  }

  const entry = data.data
  const context = entry.collections[0] ?? entry.tags[0] ?? 'vault'

  const copyAll = async () => {
    try {
      await navigator.clipboard.writeText(entry.url ?? entry.body)
      toast('Copied to clipboard', 'success')
    } catch {
      toast('Clipboard unavailable', 'error')
    }
  }

  const toTrash = () => {
    softDelete.mutate(entry.id, { onSuccess: () => navigate('/') })
    qc.removeQueries({ queryKey: entryKeys.detail(entry.id) })
  }

  return (
    <aside className={cn('card flex flex-col p-5', className)}>
      <p className="eyebrow">
        {kindLabel(entry.kind)} / {titleCase(context)}
      </p>
      <h2 className="mt-2 font-display text-2xl leading-snug font-semibold tracking-tight">{entry.title}</h2>
      {entry.language || entry.shell ? (
        <p className="mt-1 text-sm text-mute">{titleCase(entry.language ?? entry.shell ?? '')}</p>
      ) : null}
      <TagChips
        tags={entry.tags}
        hash
        className="mt-3"
        onTagClick={(t) => navigate(`/?tag=${encodeURIComponent(t)}`)}
      />

      <div className="mt-4 min-h-0">
        {entry.kind === 'snippet' ? (
          <CodeBlock code={entry.body} language={entry.language} />
        ) : entry.kind === 'command' ? (
          <CodeBlock code={entry.body} language={entry.shell ?? 'bash'} label="terminal" />
        ) : entry.kind === 'link' ? (
          <LinkCard entry={entry} />
        ) : (
          <MarkdownView content={entry.body} />
        )}
      </div>

      <div className="mt-5 flex flex-wrap items-center gap-2">
        <button type="button" className="btn btn-secondary" onClick={copyAll}>
          <Copy size={14} />
          Copy content
        </button>
        <button type="button" className="btn btn-secondary" onClick={() => togglePin.mutate(entry)}>
          <Star size={14} className={entry.is_pinned ? 'fill-accent text-accent' : ''} />
          {entry.is_pinned ? 'Unstar' : 'Star'}
        </button>
        <Link to={`/entries/${entry.id}/edit`} className="btn btn-secondary">
          Edit
        </Link>
        <button type="button" className="btn btn-danger ml-auto" onClick={toTrash}>
          <Trash2 size={14} />
          Move to trash
        </button>
      </div>

      <p className="mt-4 border-t border-line/60 pt-3 font-mono text-[11px] text-faint">
        Created {fullDate(entry.created_at)} · Updated {relativeTime(entry.updated_at)}
      </p>
    </aside>
  )
}
