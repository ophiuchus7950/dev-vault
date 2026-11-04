import { Check, Pencil, Tags as TagsIcon, Trash2, X } from 'lucide-react'
import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { EmptyState } from '../../components/EmptyState'
import { RowSkeleton } from '../../components/Skeleton'
import { cn } from '../../lib/cn'
import { useDeleteTag, useRenameTag, useTags } from './hooks'

function sizeForCount(count: number): string {
  if (count >= 10) return 'text-2xl'
  if (count >= 5) return 'text-xl'
  if (count >= 3) return 'text-lg'
  return 'text-sm'
}

export function TagsPage() {
  const navigate = useNavigate()
  const { data, isPending } = useTags()
  const rename = useRenameTag()
  const remove = useDeleteTag()
  const [renamingId, setRenamingId] = useState<string | null>(null)
  const [draft, setDraft] = useState('')

  const tags = data?.data.data ?? []

  return (
    <div className="mx-auto max-w-3xl px-8 py-8">
      <p className="eyebrow">Organise</p>
      <h1 className="mt-2 font-display text-4xl font-semibold tracking-tight">Tags</h1>
      <p className="mt-2 text-mute">Click a tag to filter the vault; rename or delete below.</p>

      {isPending ? (
        <div className="card mt-6 divide-y divide-line/70">
          <RowSkeleton />
          <RowSkeleton />
        </div>
      ) : tags.length === 0 ? (
        <EmptyState
          icon={TagsIcon}
          title="No tags yet"
          hint="Tags appear here as soon as you add them to entries."
          className="card mt-6"
        />
      ) : (
        <>
          <div className="card mt-6 flex flex-wrap items-baseline gap-x-4 gap-y-2 p-5">
            {tags.map((t) => (
              <button
                key={t.id}
                type="button"
                onClick={() => navigate(`/?tag=${encodeURIComponent(t.name)}`)}
                className={cn(
                  'cursor-pointer font-medium text-mute hover:text-accent',
                  sizeForCount(t.count),
                )}
              >
                #{t.name}
                <span className="ml-1 align-middle font-mono text-[10px] text-faint">{t.count}</span>
              </button>
            ))}
          </div>

          <div className="card mt-5 overflow-hidden">
            <p className="border-b border-line/70 px-4 py-2.5 font-mono text-[10px] tracking-[0.18em] text-faint uppercase">
              Manage
            </p>
            {tags.map((t) => (
              <div
                key={t.id}
                className="flex items-center gap-3 border-b border-line/70 px-4 py-2.5 last:border-b-0"
              >
                {renamingId === t.id ? (
                  <>
                    <input
                      className="input max-w-xs py-1.5"
                      value={draft}
                      aria-label="New tag name"
                      onChange={(e) => setDraft(e.target.value)}
                      onKeyDown={(e) => {
                        if (e.key === 'Enter' && draft.trim()) {
                          rename.mutate({ id: t.id, name: draft.trim() })
                          setRenamingId(null)
                        }
                        if (e.key === 'Escape') setRenamingId(null)
                      }}
                    />
                    <button
                      type="button"
                      className="btn btn-secondary size-8 p-0"
                      aria-label="Save tag name"
                      onClick={() => {
                        if (draft.trim()) rename.mutate({ id: t.id, name: draft.trim() })
                        setRenamingId(null)
                      }}
                    >
                      <Check size={14} />
                    </button>
                    <button
                      type="button"
                      className="btn btn-ghost size-8 p-0"
                      aria-label="Cancel rename"
                      onClick={() => setRenamingId(null)}
                    >
                      <X size={14} />
                    </button>
                  </>
                ) : (
                  <>
                    <span className="font-mono text-sm text-ink">#{t.name}</span>
                    <span className="text-xs text-faint">
                      {t.count} {t.count === 1 ? 'entry' : 'entries'}
                    </span>
                    <span className="ml-auto flex gap-1">
                      <button
                        type="button"
                        className="btn btn-ghost size-8 p-0"
                        aria-label={`Rename tag ${t.name}`}
                        onClick={() => {
                          setRenamingId(t.id)
                          setDraft(t.name)
                        }}
                      >
                        <Pencil size={14} />
                      </button>
                      <button
                        type="button"
                        className="btn btn-ghost size-8 p-0 text-danger"
                        aria-label={`Delete tag ${t.name}`}
                        onClick={() => {
                          if (
                            window.confirm(`Delete tag “${t.name}”? It is removed from ${t.count} entries.`)
                          ) {
                            remove.mutate(t.id)
                          }
                        }}
                      >
                        <Trash2 size={14} />
                      </button>
                    </span>
                  </>
                )}
              </div>
            ))}
          </div>
        </>
      )}
    </div>
  )
}
