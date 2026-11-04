import { useQuery } from '@tanstack/react-query'
import { Command } from 'cmdk'
import {
  BookOpen,
  Code2,
  FileText,
  Folder,
  Layers,
  Link2,
  type LucideIcon,
  Monitor,
  Moon,
  Search,
  Settings2,
  Star,
  Sun,
  Tags,
  Terminal,
  Trash2,
} from 'lucide-react'
import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { downloadExport, searchEntries } from '../api/endpoints'
import { useDebounced } from '../lib/hooks'
import { useTheme } from '../stores/theme'
import { toast } from '../stores/toast'
import { useUi } from '../stores/ui'

interface Action {
  icon: LucideIcon
  label: string
  keywords?: string
  run: () => void
}

function itemClass() {
  return 'flex cursor-pointer items-center gap-2.5 rounded-lg px-3 py-2 text-sm text-mute data-[selected=true]:bg-panel2 data-[selected=true]:text-ink'
}

function groupHeading() {
  return 'px-3 pt-2 pb-1 font-mono text-[10px] tracking-[0.18em] text-faint uppercase'
}

export function CommandPalette() {
  const { paletteOpen, setPaletteOpen } = useUi()
  const navigate = useNavigate()
  const theme = useTheme()
  const [query, setQuery] = useState('')
  const debounced = useDebounced(query, 200)
  const searchQ = debounced.trim().length >= 2 ? debounced.trim() : ''

  const { data } = useQuery({
    queryKey: ['search', searchQ, 'palette'],
    queryFn: () => searchEntries(searchQ, { limit: 6, is_archived: false }),
    enabled: paletteOpen && searchQ.length > 0,
    staleTime: 10_000,
  })

  if (!paletteOpen) return null

  const close = (fn: () => void) => () => {
    setPaletteOpen(false)
    setQuery('')
    fn()
  }

  const go = (path: string) => close(() => navigate(path))

  const actions: Action[] = [
    { icon: Code2, label: 'New snippet', run: go('/entries/new?kind=snippet') },
    { icon: Terminal, label: 'New command', run: go('/entries/new?kind=command') },
    { icon: FileText, label: 'New note', run: go('/entries/new?kind=note') },
    { icon: Link2, label: 'New link', run: go('/entries/new?kind=link') },
    { icon: BookOpen, label: 'New doc', run: go('/entries/new?kind=doc') },
    { icon: Layers, label: 'Go to All entries', run: go('/') },
    { icon: Star, label: 'Go to Starred', run: go('/starred') },
    { icon: Folder, label: 'Go to Collections', run: go('/collections') },
    { icon: Tags, label: 'Go to Tags', run: go('/tags') },
    { icon: Trash2, label: 'Go to Trash', run: go('/trash') },
    { icon: Settings2, label: 'Go to Settings', run: go('/settings') },
    {
      icon: theme.resolved === 'dark' ? Sun : Moon,
      label: 'Toggle theme',
      keywords: 'light dark system appearance',
      run: close(() => theme.cycle()),
    },
    {
      icon: Monitor,
      label: 'Export vault (JSON)',
      keywords: 'download backup data',
      run: close(() =>
        downloadExport('json')
          .then(() => toast('Vault exported', 'success'))
          .catch((e: Error) => toast(e.message, 'error')),
      ),
    },
  ]

  const q = query.trim().toLowerCase()
  const visibleActions = q
    ? actions.filter((a) => `${a.label} ${a.keywords ?? ''}`.toLowerCase().includes(q))
    : actions

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-label="Command palette"
      className="fixed inset-0 z-50 flex items-start justify-center bg-black/50 px-4 pt-[14vh]"
      onClick={() => setPaletteOpen(false)}
      onKeyDown={(e) => {
        if (e.key === 'Escape') setPaletteOpen(false)
      }}
    >
      <Command
        loop
        shouldFilter={false}
        onClick={(e) => e.stopPropagation()}
        className="w-full max-w-xl overflow-hidden rounded-2xl border border-line bg-panel shadow-2xl shadow-black/40"
      >
        <div className="flex items-center gap-2.5 border-b border-line/70 px-4">
          <Search size={15} className="shrink-0 text-faint" />
          <Command.Input
            autoFocus
            value={query}
            onValueChange={setQuery}
            placeholder="Type a command or search entries…"
            className="w-full bg-transparent py-3.5 text-sm text-ink outline-none placeholder:text-faint"
          />
          <span className="kbd">Esc</span>
        </div>
        <Command.List className="max-h-80 overflow-y-auto p-2">
          <Command.Empty className="px-3 py-8 text-center text-sm text-mute">
            Nothing matches “{query}”.
          </Command.Empty>

          {visibleActions.length > 0 ? (
            <Command.Group heading="Actions" className={groupHeading()}>
              {visibleActions.map((a) => (
                <Command.Item key={a.label} onSelect={close(a.run)} className={itemClass()}>
                  <a.icon size={15} strokeWidth={1.75} />
                  {a.label}
                </Command.Item>
              ))}
            </Command.Group>
          ) : null}

          {searchQ && data?.data.data.length ? (
            <Command.Group heading="Entries" className={groupHeading()}>
              {data.data.data.map((hit) => (
                <Command.Item
                  key={hit.id}
                  value={`entry-${hit.id}`}
                  onSelect={go(`/entries/${hit.id}`)}
                  className={itemClass()}
                >
                  <Search size={15} strokeWidth={1.75} />
                  <span className="flex-1 truncate">{hit.title}</span>
                  <span className="kbd">{hit.kind}</span>
                </Command.Item>
              ))}
            </Command.Group>
          ) : null}
        </Command.List>
      </Command>
    </div>
  )
}
