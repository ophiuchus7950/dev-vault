import type { ENTRY_KINDS } from '../lib/kinds'

export type EntryKind = (typeof ENTRY_KINDS)[number]

export interface Entry {
  id: string
  kind: EntryKind
  title: string
  body: string
  language: string | null
  shell: string | null
  url: string | null
  tags: string[]
  collections: string[]
  is_pinned: boolean
  is_archived: boolean
  created_at: string
  updated_at: string
  deleted_at: string | null
}

export interface Tag {
  id: string
  name: string
  count: number
}

export interface Collection {
  id: string
  name: string
  slug: string
  description: string
  entry_count: number
  created_at: string
  updated_at: string
}

export interface Page<T> {
  data: T[]
  next_cursor: string | null
  has_more: boolean
}

export type SearchMatch = 'ts' | 'trgm' | 'hybrid'

export interface SearchHit extends Entry {
  score: number
  matched: SearchMatch
}

export interface ErrorDetail {
  field: string
  code: string
  message: string
}

export interface EntryListParams {
  kind?: EntryKind
  language?: string
  tag?: string[]
  collection?: string
  is_pinned?: boolean
  is_archived?: boolean
  sort?: 'updated_at' | 'created_at' | 'title' | 'relevance'
  order?: 'asc' | 'desc'
  limit?: number
  cursor?: string
}

export type EntryDraft = Partial<Omit<Entry, 'id' | 'created_at' | 'updated_at' | 'deleted_at'>> & {
  kind: EntryKind
  title: string
}

export interface ImportSummary {
  imported: number
  skipped: number
  overwritten: number
  errors: string[]
}

export interface HealthInfo {
  status: string
  version: string
  uptime_s: number
}
