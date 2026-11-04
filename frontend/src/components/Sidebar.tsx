import { useQuery } from '@tanstack/react-query'
import { Folder, Layers, Settings2, Star, Tags, Trash2 } from 'lucide-react'
import { NavLink } from 'react-router-dom'
import { listEntries } from '../api/endpoints'
import { cn } from '../lib/cn'

function useLiveEntryCount(): number | undefined {
  const { data } = useQuery({
    queryKey: ['vault-count'],
    queryFn: async () => (await listEntries({ limit: 100, is_archived: false })).data.data.length,
    staleTime: 30_000,
  })
  return data
}

const linkClass = ({ isActive }: { isActive: boolean }) =>
  cn(
    'flex items-center gap-3 rounded-lg border-l-2 px-3 py-2 text-sm transition-colors',
    isActive
      ? 'border-accent bg-panel text-ink'
      : 'border-transparent text-mute hover:bg-panel/60 hover:text-ink',
  )

export function Sidebar() {
  const count = useLiveEntryCount()

  return (
    <aside className="flex w-60 shrink-0 flex-col bg-sidebar">
      <div className="flex items-center gap-2.5 px-5 py-5">
        <img src="/favicon.svg" alt="" className="size-8 rounded-lg" />
        <span className="text-[15px] font-semibold tracking-tight">DevVault</span>
      </div>

      <p className="px-5 pt-3 pb-2 font-mono text-[10px] tracking-[0.18em] text-faint uppercase">Workspace</p>
      <nav className="space-y-1 px-3">
        <NavLink to="/" end className={linkClass}>
          <Layers size={16} strokeWidth={1.75} />
          <span className="flex-1">All entries</span>
          {count !== undefined ? (
            <span className="rounded-full bg-panel2 px-2 py-0.5 font-mono text-[11px] text-mute">
              {count}
            </span>
          ) : null}
        </NavLink>
        <NavLink to="/starred" className={linkClass}>
          <Star size={16} strokeWidth={1.75} />
          <span>Starred</span>
        </NavLink>
        <NavLink to="/collections" className={linkClass}>
          <Folder size={16} strokeWidth={1.75} />
          <span>Collections</span>
        </NavLink>
        <NavLink to="/tags" className={linkClass}>
          <Tags size={16} strokeWidth={1.75} />
          <span>Tags</span>
        </NavLink>
        <NavLink to="/trash" className={linkClass}>
          <Trash2 size={16} strokeWidth={1.75} />
          <span>Trash</span>
        </NavLink>
      </nav>

      <div className="mt-auto">
        <p className="px-5 pt-3 pb-2 font-mono text-[10px] tracking-[0.18em] text-faint uppercase">
          Preferences
        </p>
        <nav className="space-y-1 px-3 pb-2">
          <NavLink to="/settings" className={linkClass}>
            <Settings2 size={16} strokeWidth={1.75} />
            <span>Settings</span>
          </NavLink>
        </nav>
        <p className="border-t border-line/60 px-5 py-3 font-mono text-[10px] tracking-[0.14em] text-faint uppercase">
          Local workspace · 01
        </p>
      </div>
    </aside>
  )
}
