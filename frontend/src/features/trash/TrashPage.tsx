import { useQuery } from '@tanstack/react-query'
import { RotateCcw, Trash2 } from 'lucide-react'
import { listTrash } from '../../api/endpoints'
import { EmptyState } from '../../components/EmptyState'
import { KindIcon } from '../../components/KindIcon'
import { RowSkeleton } from '../../components/Skeleton'
import { relativeTime } from '../../lib/format'
import { kindLabel } from '../../lib/kinds'
import { usePurgeEntry, useRestoreEntry } from '../entries/hooks'

export function TrashPage() {
  const { data, isPending } = useQuery({ queryKey: ['trash'], queryFn: listTrash })
  const restore = useRestoreEntry()
  const purge = usePurgeEntry()

  const entries = data?.data.data ?? []

  return (
    <div className="mx-auto max-w-3xl px-8 py-8">
      <p className="eyebrow">Safety net</p>
      <h1 className="mt-2 font-display text-4xl font-semibold tracking-tight">Trash</h1>
      <p className="mt-2 text-mute">Deleted entries land here first — restore them, or purge forever.</p>

      {isPending ? (
        <div className="card mt-6 divide-y divide-line/70">
          <RowSkeleton />
          <RowSkeleton />
        </div>
      ) : entries.length === 0 ? (
        <EmptyState
          icon={Trash2}
          title="Trash is empty"
          hint="Deleted entries will wait here until you decide."
          className="card mt-6"
        />
      ) : (
        <div className="card mt-6 overflow-hidden">
          {entries.map((e) => (
            <div
              key={e.id}
              className="flex items-center gap-3 border-b border-line/70 px-4 py-3 last:border-b-0"
            >
              <KindIcon kind={e.kind} />
              <div className="min-w-0 flex-1">
                <p className="truncate text-sm font-medium">{e.title}</p>
                <p className="text-xs text-faint">
                  {kindLabel(e.kind)} · deleted {relativeTime(e.deleted_at ?? e.updated_at)}
                </p>
              </div>
              <button
                type="button"
                className="btn btn-secondary"
                onClick={() => restore.mutate(e.id)}
                disabled={restore.isPending}
              >
                <RotateCcw size={13} />
                Restore
              </button>
              <button
                type="button"
                className="btn btn-danger"
                onClick={() => {
                  if (window.confirm(`Permanently delete “${e.title}”? This cannot be undone.`)) {
                    purge.mutate(e.id)
                  }
                }}
                disabled={purge.isPending}
              >
                Purge
              </button>
            </div>
          ))}
        </div>
      )}
    </div>
  )
}
