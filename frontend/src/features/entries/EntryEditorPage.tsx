import { ArrowLeft, Loader2, X } from 'lucide-react'
import { lazy, Suspense, useEffect, useRef, useState } from 'react'
import { Link, useNavigate, useParams, useSearchParams } from 'react-router-dom'
import { ApiError } from '../../api/client'
import type { EntryDraft, EntryKind } from '../../api/types'
import { MarkdownView } from '../../components/MarkdownView'
import { Skeleton } from '../../components/Skeleton'
import { cn } from '../../lib/cn'
import { titleCase } from '../../lib/format'
import { ENTRY_KINDS, KIND_META, SHELLS, SNIPPET_LANGUAGES } from '../../lib/kinds'
import { useCollections } from '../collections/hooks'
import { useTags } from '../tags/hooks'
import { useCreateEntry, useEntry, useUpdateEntry } from './hooks'

const CodeEditor = lazy(() => import('./CodeEditor'))

interface FormState {
  kind: EntryKind
  title: string
  body: string
  language: string
  shell: string
  url: string
  tags: string[]
  collections: string[]
  is_pinned: boolean
  is_archived: boolean
}

const EMPTY: FormState = {
  kind: 'snippet',
  title: '',
  body: '',
  language: 'typescript',
  shell: 'bash',
  url: '',
  tags: [],
  collections: [],
  is_pinned: false,
  is_archived: false,
}

function TagField({
  value,
  onChange,
  suggestions,
}: {
  value: string[]
  onChange: (tags: string[]) => void
  suggestions: string[]
}) {
  const [text, setText] = useState('')

  const add = (raw: string) => {
    const tag = raw.trim().toLowerCase().replace(/\s+/g, '-').replace(/^[#]+/, '')
    if (tag && !value.includes(tag) && value.length < 100) onChange([...value, tag])
    setText('')
  }

  const matches = suggestions
    .filter((s) => !value.includes(s) && (text.trim() ? s.includes(text.trim().toLowerCase()) : true))
    .slice(0, 6)

  return (
    <div>
      {value.length > 0 ? (
        <div className="mb-2 flex flex-wrap gap-1.5">
          {value.map((t) => (
            <span key={t} className="chip chip-accent">
              #{t}
              <button
                type="button"
                aria-label={`Remove tag ${t}`}
                onClick={() => onChange(value.filter((x) => x !== t))}
                className="cursor-pointer hover:text-danger"
              >
                <X size={11} />
              </button>
            </span>
          ))}
        </div>
      ) : null}
      <div className="relative">
        <input
          className="input"
          value={text}
          placeholder="Add a tag — Enter or comma to add"
          aria-label="Add tag"
          onChange={(e) => {
            const next = e.target.value
            if (next.endsWith(',')) add(next.slice(0, -1))
            else setText(next)
          }}
          onKeyDown={(e) => {
            if (e.key === 'Enter') {
              e.preventDefault()
              add(text)
            }
            if (e.key === 'Backspace' && !text && value.length > 0) {
              onChange(value.slice(0, -1))
            }
          }}
        />
        {text.trim() && matches.length > 0 ? (
          <div className="card absolute top-full left-0 z-10 mt-1 w-full p-1">
            {matches.map((s) => (
              <button
                key={s}
                type="button"
                onClick={() => add(s)}
                className="flex w-full cursor-pointer items-center gap-2 rounded-lg px-3 py-1.5 text-left text-sm text-mute hover:bg-panel2 hover:text-ink"
              >
                #{s}
              </button>
            ))}
          </div>
        ) : null}
      </div>
    </div>
  )
}

export function EntryEditorPage() {
  const { id } = useParams()
  const editing = Boolean(id)
  const navigate = useNavigate()
  const [params] = useSearchParams()
  const { data: existing } = useEntry(editing ? id : undefined)
  const create = useCreateEntry()
  const update = useUpdateEntry(id ?? '')
  const { data: tagsData } = useTags()
  const { data: collectionsData } = useCollections()

  const [form, setForm] = useState<FormState>(() => ({
    ...EMPTY,
    kind: (params.get('kind') as EntryKind | null) ?? 'snippet',
  }))
  const [fieldErrors, setFieldErrors] = useState<Record<string, string>>({})
  const [bodyTab, setBodyTab] = useState<'write' | 'preview'>('write')
  const loadedRef = useRef(false)

  useEffect(() => {
    if (editing && existing && !loadedRef.current) {
      const e = existing.data
      setForm({
        kind: e.kind,
        title: e.title,
        body: e.body,
        language: e.language ?? '',
        shell: e.shell ?? 'bash',
        url: e.url ?? '',
        tags: e.tags,
        collections: e.collections,
        is_pinned: e.is_pinned,
        is_archived: e.is_archived,
      })
      loadedRef.current = true
    }
  }, [editing, existing])

  const patch = (partial: Partial<FormState>) => setForm((f) => ({ ...f, ...partial }))

  const validate = (): boolean => {
    const errors: Record<string, string> = {}
    if (!form.title.trim()) errors.title = 'Title is required.'
    if (form.kind === 'link' && !form.url.trim()) errors.url = 'A URL is required for links.'
    if (form.kind !== 'link' && !form.body.trim()) errors.body = 'Content is required.'
    setFieldErrors(errors)
    return Object.keys(errors).length === 0
  }

  const submit = async () => {
    if (!validate()) return
    const draft: EntryDraft = {
      kind: form.kind,
      title: form.title.trim(),
      body: form.kind === 'link' ? form.body : form.body,
      language: form.kind === 'snippet' ? form.language || null : null,
      shell: form.kind === 'command' ? form.shell || null : null,
      url: form.kind === 'link' ? form.url.trim() : null,
      tags: form.tags,
      collections: form.collections,
      is_pinned: form.is_pinned,
      is_archived: form.is_archived,
    }
    try {
      if (editing && id) {
        await update.mutateAsync({ patch: draft, etag: existing?.etag ?? '' })
        navigate(`/entries/${id}`)
      } else {
        const res = await create.mutateAsync(draft)
        navigate(`/entries/${res.data.id}`)
      }
    } catch (e) {
      if (e instanceof ApiError) setFieldErrors(e.fieldErrors())
    }
  }

  const submitRef = useRef(submit)
  submitRef.current = submit
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 's') {
        e.preventDefault()
        submitRef.current()
      }
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [])

  const saving = create.isPending || update.isPending
  const isMarkdownKind = form.kind === 'note' || form.kind === 'doc'

  return (
    <div className="mx-auto max-w-3xl px-8 py-8">
      <Link to={editing ? `/entries/${id}` : '/'} className="btn btn-ghost -ml-2 px-2">
        <ArrowLeft size={15} />
        Back
      </Link>
      <h1 className="mt-3 font-display text-3xl font-semibold tracking-tight">
        {editing ? 'Edit entry' : 'New entry'}
      </h1>

      <form
        className="mt-6 space-y-5"
        onSubmit={(e) => {
          e.preventDefault()
          submit()
        }}
      >
        <div>
          <p className="mb-2 text-sm font-medium text-mute">Kind</p>
          <div className="flex flex-wrap gap-1.5">
            {ENTRY_KINDS.map((k) => {
              const meta = KIND_META[k]
              const Icon = meta.icon
              return (
                <button
                  key={k}
                  type="button"
                  onClick={() => patch({ kind: k })}
                  aria-pressed={form.kind === k}
                  className={cn('btn', form.kind === k ? 'btn-accent' : 'btn-secondary')}
                >
                  <Icon size={14} />
                  {meta.label}
                </button>
              )
            })}
          </div>
        </div>

        <div>
          <label htmlFor="entry-title" className="mb-2 block text-sm font-medium text-mute">
            Title
          </label>
          <input
            id="entry-title"
            className="input text-base"
            value={form.title}
            placeholder="A short, findable name"
            onChange={(e) => patch({ title: e.target.value })}
          />
          {fieldErrors.title ? <p className="mt-1 text-xs text-danger">{fieldErrors.title}</p> : null}
        </div>

        {form.kind === 'snippet' ? (
          <div>
            <label htmlFor="entry-language" className="mb-2 block text-sm font-medium text-mute">
              Language
            </label>
            <select
              id="entry-language"
              className="input w-auto"
              value={form.language}
              onChange={(e) => patch({ language: e.target.value })}
            >
              {SNIPPET_LANGUAGES.map((l) => (
                <option key={l} value={l}>
                  {titleCase(l)}
                </option>
              ))}
            </select>
          </div>
        ) : null}

        {form.kind === 'command' ? (
          <div>
            <label htmlFor="entry-shell" className="mb-2 block text-sm font-medium text-mute">
              Shell
            </label>
            <select
              id="entry-shell"
              className="input w-auto"
              value={form.shell}
              onChange={(e) => patch({ shell: e.target.value })}
            >
              {SHELLS.map((s) => (
                <option key={s} value={s}>
                  {s}
                </option>
              ))}
            </select>
          </div>
        ) : null}

        {form.kind === 'link' ? (
          <div>
            <label htmlFor="entry-url" className="mb-2 block text-sm font-medium text-mute">
              URL
            </label>
            <input
              id="entry-url"
              className="input"
              value={form.url}
              placeholder="https://…"
              onChange={(e) => patch({ url: e.target.value })}
            />
            {fieldErrors.url ? <p className="mt-1 text-xs text-danger">{fieldErrors.url}</p> : null}
          </div>
        ) : null}

        {form.kind !== 'link' ? (
          <div>
            <div className="mb-2 flex items-center justify-between">
              <label htmlFor="entry-body" className="text-sm font-medium text-mute">
                {isMarkdownKind ? 'Body (Markdown)' : 'Content'}
              </label>
              {isMarkdownKind ? (
                <div className="flex gap-1">
                  {(['write', 'preview'] as const).map((tab) => (
                    <button
                      key={tab}
                      type="button"
                      onClick={() => setBodyTab(tab)}
                      aria-pressed={bodyTab === tab}
                      className={cn(
                        'cursor-pointer rounded-lg px-2.5 py-1 text-xs',
                        bodyTab === tab ? 'bg-panel2 text-ink' : 'text-mute hover:text-ink',
                      )}
                    >
                      {titleCase(tab)}
                    </button>
                  ))}
                </div>
              ) : null}
            </div>
            {isMarkdownKind && bodyTab === 'preview' ? (
              <div className="card min-h-[240px] p-4">
                {form.body.trim() ? (
                  <MarkdownView content={form.body} />
                ) : (
                  <p className="text-sm text-faint">Nothing to preview yet.</p>
                )}
              </div>
            ) : isMarkdownKind ? (
              <textarea
                id="entry-body"
                className="input min-h-[240px] font-mono text-[13px] leading-relaxed"
                value={form.body}
                placeholder="Write it down…"
                onChange={(e) => patch({ body: e.target.value })}
              />
            ) : (
              <Suspense
                fallback={
                  <div className="space-y-2 rounded-xl border border-line bg-codebg p-4">
                    <Skeleton className="h-3 w-2/3" />
                    <Skeleton className="h-3 w-1/2" />
                    <Loader2 size={16} className="mt-2 animate-spin text-faint" />
                  </div>
                }
              >
                <CodeEditor
                  value={form.body}
                  onChange={(body) => patch({ body })}
                  onSave={() => submitRef.current()}
                  language={form.kind === 'command' ? 'bash' : form.language}
                  placeholder={form.kind === 'command' ? '$ your command here' : 'Paste the code…'}
                />
              </Suspense>
            )}
            {fieldErrors.body ? <p className="mt-1 text-xs text-danger">{fieldErrors.body}</p> : null}
          </div>
        ) : null}

        <div>
          <p className="mb-2 text-sm font-medium text-mute">Tags</p>
          <TagField
            value={form.tags}
            onChange={(tags) => patch({ tags })}
            suggestions={tagsData?.data.data.map((t) => t.name) ?? []}
          />
        </div>

        {collectionsData && collectionsData.data.data.length > 0 ? (
          <div>
            <p className="mb-2 text-sm font-medium text-mute">Collections</p>
            <div className="flex flex-wrap gap-1.5">
              {collectionsData.data.data.map((c) => {
                const active = form.collections.includes(c.slug)
                return (
                  <button
                    key={c.id}
                    type="button"
                    aria-pressed={active}
                    onClick={() =>
                      patch({
                        collections: active
                          ? form.collections.filter((s) => s !== c.slug)
                          : [...form.collections, c.slug],
                      })
                    }
                    className={cn('chip cursor-pointer', active ? 'chip-accent' : 'hover:text-ink')}
                  >
                    {c.name}
                  </button>
                )
              })}
            </div>
          </div>
        ) : null}

        <div className="flex flex-wrap gap-2">
          {(
            [
              ['is_pinned', 'Starred'],
              ['is_archived', 'Archived'],
            ] as const
          ).map(([key, label]) => (
            <button
              key={key}
              type="button"
              aria-pressed={form[key]}
              onClick={() => patch({ [key]: !form[key] } as Partial<FormState>)}
              className={cn('chip cursor-pointer px-3 py-1.5 text-xs', form[key] ? 'chip-accent' : '')}
            >
              {label}
            </button>
          ))}
        </div>

        <div className="flex items-center gap-2 border-t border-line/60 pt-5">
          <button type="submit" className="btn btn-accent" disabled={saving}>
            {saving ? <Loader2 size={14} className="animate-spin" /> : null}
            {editing ? 'Save changes' : 'Create entry'}
          </button>
          <button type="button" className="btn btn-secondary" onClick={() => navigate(-1)}>
            Cancel
          </button>
          <span className="ml-auto text-xs text-faint">
            <span className="kbd">⌘S</span> to save
          </span>
        </div>
      </form>
    </div>
  )
}
