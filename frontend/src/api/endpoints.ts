import { API_BASE, http } from './client'
import type {
  Collection,
  Entry,
  EntryDraft,
  EntryListParams,
  HealthInfo,
  ImportSummary,
  Page,
  SearchHit,
  SearchMatch,
  Tag,
} from './types'

/* ---------------- entries ---------------- */

export function listEntries(params: EntryListParams = {}) {
  return http<Page<Entry>>('/entries', { query: { ...params } })
}

export function getEntry(id: string) {
  return http<Entry>(`/entries/${id}`)
}

export function createEntry(draft: EntryDraft) {
  return http<Entry>('/entries', { method: 'POST', body: draft })
}

export function updateEntry(id: string, patch: Partial<EntryDraft>, ifMatch: string) {
  return http<Entry>(`/entries/${id}`, { method: 'PATCH', body: patch, ifMatch })
}

export function deleteEntry(id: string) {
  return http<void>(`/entries/${id}`, { method: 'DELETE' })
}

export function restoreEntry(id: string) {
  return http<Entry>(`/entries/${id}/restore`, { method: 'POST' })
}

export function purgeEntry(id: string) {
  return http<void>(`/entries/${id}/purge`, { method: 'DELETE' })
}

export function listTrash() {
  return http<Page<Entry>>('/trash')
}

/* ---------------- search ---------------- */

export function searchEntries(
  q: string,
  params: Omit<EntryListParams, 'cursor' | 'sort' | 'relevance'> = {},
  mode?: SearchMatch,
) {
  return http<Page<SearchHit>>('/search', {
    query: { ...params, q, mode },
  })
}

/* ---------------- tags ---------------- */

export function listTags(q = '', sort?: 'name' | 'count') {
  return http<{ data: Tag[] }>('/tags', { query: { q, sort } })
}

export function renameTag(id: string, name: string) {
  return http<Tag>(`/tags/${id}`, { method: 'PATCH', body: { name } })
}

export function deleteTag(id: string) {
  return http<void>(`/tags/${id}`, { method: 'DELETE' })
}

/* ---------------- collections ---------------- */

export function listCollections() {
  return http<{ data: Collection[] }>('/collections')
}

export function createCollection(body: { name: string; description?: string }) {
  return http<Collection>('/collections', { method: 'POST', body })
}

export function getCollection(idOrSlug: string) {
  return http<Collection>(`/collections/${idOrSlug}`)
}

export function updateCollection(idOrSlug: string, patch: { name?: string; description?: string }) {
  return http<Collection>(`/collections/${idOrSlug}`, { method: 'PATCH', body: patch })
}

export function deleteCollection(idOrSlug: string) {
  return http<void>(`/collections/${idOrSlug}`, { method: 'DELETE' })
}

export function assignEntries(collectionId: string, entryIds: string[]) {
  return http<void>(`/collections/${collectionId}/entries`, {
    method: 'POST',
    body: { entry_ids: entryIds },
  })
}

export function unassignEntries(collectionId: string, entryIds: string[]) {
  return http<void>(`/collections/${collectionId}/entries`, {
    method: 'DELETE',
    body: { entry_ids: entryIds },
  })
}

/* ---------------- portability ---------------- */

export async function downloadExport(format: 'json' | 'zip'): Promise<void> {
  const res = await fetch(`${API_BASE}/export?format=${format}`)
  if (!res.ok) throw new Error(`Export failed (${res.status})`)
  const blob = await res.blob()
  const dispo = res.headers.get('content-disposition') ?? ''
  const match = /filename="?([^";]+)"?/.exec(dispo)
  const name = match?.[1] ?? `devvault-export.${format}`
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = name
  a.click()
  URL.revokeObjectURL(url)
}

export function entryExportUrl(id: string, format: 'md' | 'json'): string {
  return `${API_BASE}/entries/${id}/export?format=${format}`
}

export async function importVault(file: File, onConflict: 'skip' | 'overwrite'): Promise<ImportSummary> {
  const form = new FormData()
  form.append('file', file)
  const res = await fetch(`${API_BASE}/import?on_conflict=${onConflict}`, {
    method: 'POST',
    body: form,
  })
  if (!res.ok) {
    let message = `Import failed (${res.status})`
    try {
      const payload = (await res.json()) as { error?: { message?: string } }
      message = payload?.error?.message ?? message
    } catch {
      // ignore
    }
    throw new Error(message)
  }
  return (await res.json()) as ImportSummary
}

/* ---------------- health ---------------- */

export async function getHealth(): Promise<HealthInfo> {
  const res = await fetch('/health')
  if (!res.ok) throw new Error(`Health check failed (${res.status})`)
  return (await res.json()) as HealthInfo
}
