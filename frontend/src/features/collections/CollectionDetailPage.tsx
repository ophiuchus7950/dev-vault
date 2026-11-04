import { ArrowLeft, Check, Folder, Pencil, Trash2, X } from 'lucide-react'
import { useState } from 'react'
import { Link, useNavigate, useParams } from 'react-router-dom'
import { EmptyState } from '../../components/EmptyState'
import { RowSkeleton } from '../../components/Skeleton'
import { EntryCard } from '../entries/EntryCard'
import { useEntries } from '../entries/hooks'
import { useCollection, useDeleteCollection, useUpdateCollection } from './hooks'

export function CollectionDetailPage() {
  const { slug } = useParams()
  const navigate = useNavigate()
  const { data, isPending, isError } = useCollection(slug)
  const entriesQuery = useEntries({ collection: slug, is_archived: false, limit: 50 })
  const update = useUpdateCollection()
  const remove = useDeleteCollection()
  const [editing, setEditing] = useState(false)
  const [name, setName] = useState('')
  const [description, setDescription] = useState('')

  if (isPending) {
    return (
      <div className="mx-auto max-w-4xl px-8 py-8">
        <div className="card divide-y divide-line/70">
          <RowSkeleton />
          <RowSkeleton />
        </div>
      </div>
    )
  }

  if (isError || !data) {
    return (
      <div className="mx-auto max-w-4xl px-8 py-8">
        <EmptyState
          icon={Folder}
          title="Collection not found"
          action={
            <Link to="/collections" className="btn btn-secondary">
              Back to collections
            </Link>
          }
        />
      </div>
    )
  }

  const collection = data.data
  const entries = entriesQuery.data?.pages.flatMap((p) => p.data.data) ?? []

  const startEdit = () => {
    setName(collection.name)
    setDescription(collection.description)
    setEditing(true)
  }

  return (
    <div className="mx-auto max-w-4xl px-8 py-8">
      <Link to="/collections" className="btn btn-ghost -ml-2 px-2">
        <ArrowLeft size={15} />
        Collections
      </Link>

      {editing ? (
        <div className="mt-4 space-y-2">
          <input
            className="input text-lg font-semibold"
            value={name}
            aria-label="Collection name"
            onChange={(e) => setName(e.target.value)}
          />
          <input
            className="input"
            value={description}
            placeholder="Description"
            aria-label="Collection description"
            onChange={(e) => setDescription(e.target.value)}
          />
          <div className="flex gap-2">
            <button
              type="button"
              className="btn btn-accent"
              onClick={() => {
                update.mutate({
                  slug: collection.slug,
                  patch: { name: name.trim(), description: description.trim() },
                })
                setEditing(false)
              }}
              disabled={!name.trim()}
            >
              <Check size={14} />
              Save
            </button>
            <button type="button" className="btn btn-secondary" onClick={() => setEditing(false)}>
              <X size={14} />
              Cancel
            </button>
          </div>
        </div>
      ) : (
        <div className="mt-3 flex items-start gap-3">
          <div className="min-w-0 flex-1">
            <h1 className="font-display text-3xl font-semibold tracking-tight">{collection.name}</h1>
            {collection.description ? <p className="mt-1 text-mute">{collection.description}</p> : null}
            <p className="mt-1 font-mono text-[11px] text-faint">
              /{collection.slug} · {collection.entry_count}{' '}
              {collection.entry_count === 1 ? 'entry' : 'entries'}
            </p>
          </div>
          <div className="flex gap-1.5">
            <button
              type="button"
              className="btn btn-secondary size-8 p-0"
              aria-label="Edit collection"
              onClick={startEdit}
            >
              <Pencil size={14} />
            </button>
            <button
              type="button"
              className="btn btn-ghost size-8 p-0 text-danger"
              aria-label="Delete collection"
              onClick={() => {
                if (window.confirm(`Delete collection “${collection.name}”? Entries are kept.`)) {
                  remove.mutate(collection.slug, { onSuccess: () => navigate('/collections') })
                }
              }}
            >
              <Trash2 size={14} />
            </button>
          </div>
        </div>
      )}

      <div className="card mt-6 overflow-hidden">
        {entriesQuery.isPending ? (
          <div className="divide-y divide-line/70">
            <RowSkeleton />
            <RowSkeleton />
          </div>
        ) : entries.length === 0 ? (
          <EmptyState
            icon={Folder}
            title="Nothing assigned yet"
            hint="Open an entry, hit Edit, and pick this collection to file entries here."
          />
        ) : (
          entries.map((e) => <EntryCard key={e.id} entry={e} onSelect={() => navigate(`/entries/${e.id}`)} />)
        )}
      </div>
    </div>
  )
}
