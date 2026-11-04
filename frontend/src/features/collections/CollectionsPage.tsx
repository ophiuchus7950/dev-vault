import { Folder, Loader2, Plus } from 'lucide-react'
import { type FormEvent, useState } from 'react'
import { Link } from 'react-router-dom'
import { EmptyState } from '../../components/EmptyState'
import { Skeleton } from '../../components/Skeleton'
import { useCollections, useCreateCollection } from './hooks'

export function CollectionsPage() {
  const { data, isPending } = useCollections()
  const create = useCreateCollection()
  const [name, setName] = useState('')
  const [description, setDescription] = useState('')

  const submit = (e: FormEvent) => {
    e.preventDefault()
    const trimmed = name.trim()
    if (!trimmed) return
    create.mutate({ name: trimmed, description: description.trim() || undefined })
    setName('')
    setDescription('')
  }

  const collections = data?.data.data ?? []

  return (
    <div className="mx-auto max-w-4xl px-8 py-8">
      <p className="eyebrow">Organise</p>
      <h1 className="mt-2 font-display text-4xl font-semibold tracking-tight">Collections</h1>
      <p className="mt-2 text-mute">Group related entries into named sets.</p>

      <form onSubmit={submit} className="card mt-6 flex flex-wrap items-center gap-2 p-3">
        <input
          className="input max-w-xs flex-1"
          value={name}
          placeholder="New collection name"
          aria-label="New collection name"
          onChange={(e) => setName(e.target.value)}
        />
        <input
          className="input max-w-sm flex-1"
          value={description}
          placeholder="Optional description"
          aria-label="New collection description"
          onChange={(e) => setDescription(e.target.value)}
        />
        <button type="submit" className="btn btn-accent" disabled={!name.trim() || create.isPending}>
          {create.isPending ? <Loader2 size={14} className="animate-spin" /> : <Plus size={15} />}
          Create
        </button>
      </form>

      {isPending ? (
        <div className="mt-5 grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
          <Skeleton className="h-36 rounded-2xl" />
          <Skeleton className="h-36 rounded-2xl" />
        </div>
      ) : collections.length === 0 ? (
        <EmptyState
          icon={Folder}
          title="No collections yet"
          hint="Create one above — e.g. “Backend”, “Interviews”, “Reading list”."
          className="card mt-5"
        />
      ) : (
        <div className="mt-5 grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {collections.map((c) => (
            <Link
              key={c.id}
              to={`/collections/${c.slug}`}
              className="card flex flex-col p-4 transition-colors hover:border-linestrong"
            >
              <div className="flex items-center gap-2.5">
                <span className="icon-tile size-8">
                  <Folder size={15} strokeWidth={1.75} />
                </span>
                <p className="truncate font-semibold">{c.name}</p>
              </div>
              {c.description ? <p className="mt-2 line-clamp-2 text-sm text-mute">{c.description}</p> : null}
              <p className="mt-auto pt-3 font-mono text-[11px] text-faint">
                {c.entry_count} {c.entry_count === 1 ? 'entry' : 'entries'} · /{c.slug}
              </p>
            </Link>
          ))}
        </div>
      )}
    </div>
  )
}
