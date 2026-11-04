import { useEffect } from 'react'
import { Outlet, useNavigate } from 'react-router-dom'
import { useUi } from '../stores/ui'
import { CommandPalette } from './CommandPalette'
import { Sidebar } from './Sidebar'
import { Toaster } from './Toaster'
import { Topbar } from './Topbar'

export function AppShell() {
  const navigate = useNavigate()
  const setPaletteOpen = useUi((s) => s.setPaletteOpen)

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const target = e.target as HTMLElement
      const typing = target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'k') {
        e.preventDefault()
        setPaletteOpen(true)
        return
      }
      if (e.key === '/' && !typing) {
        e.preventDefault()
        document.getElementById('topbar-search')?.focus()
        return
      }
      if (e.key === 'n' && !typing && !e.metaKey && !e.ctrlKey && !e.altKey) {
        e.preventDefault()
        navigate('/entries/new')
      }
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [navigate, setPaletteOpen])

  return (
    <div className="flex h-full">
      <Sidebar />
      <div className="flex min-w-0 flex-1 flex-col">
        <Topbar />
        <main className="min-h-0 flex-1 overflow-y-auto">
          <Outlet />
        </main>
      </div>
      <CommandPalette />
      <Toaster />
    </div>
  )
}
