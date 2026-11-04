import { BookOpen, Code2, FileText, Link2, type LucideIcon, Terminal } from 'lucide-react'
import { titleCase } from './format'

export const ENTRY_KINDS = ['snippet', 'command', 'note', 'link', 'doc'] as const
export type EntryKind = (typeof ENTRY_KINDS)[number]

export interface KindMeta {
  label: string
  plural: string
  icon: LucideIcon
}

export const KIND_META: Record<EntryKind, KindMeta> = {
  snippet: { label: 'Snippet', plural: 'Snippets', icon: Code2 },
  command: { label: 'Command', plural: 'Commands', icon: Terminal },
  note: { label: 'Note', plural: 'Notes', icon: FileText },
  link: { label: 'Link', plural: 'Links', icon: Link2 },
  doc: { label: 'Doc', plural: 'Docs', icon: BookOpen },
}

export function kindLabel(kind: string): string {
  return KIND_META[kind as EntryKind]?.label ?? titleCase(kind)
}

export const SNIPPET_LANGUAGES = [
  'typescript',
  'javascript',
  'rust',
  'python',
  'sql',
  'json',
  'bash',
  'markdown',
  'go',
  'yaml',
  'text',
]

export const SHELLS = ['bash', 'zsh', 'sh', 'fish', 'powershell']
