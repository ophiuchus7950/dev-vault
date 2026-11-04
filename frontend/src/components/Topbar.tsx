import { Plus, Search } from 'lucide-react'
import { type FormEvent, useRef, useState } from 'react'
import { useNavigate, useSearchParams } from 'react-router-dom'
import { ThemeToggle } from './ThemeToggle'

export function Topbar() {
  const navigate = useNavigate()
  const [params] = useSearchParams()
  const [q, setQ] = useState(params.get('q') ?? '')
  const inputRef = useRef<HTMLInputElement>(null)

  const submit = (e: FormEvent) => {
    e.preventDefault()
    const trimmed = q.trim()
    if (trimmed) navigate(`/search?q=${encodeURIComponent(trimmed)}`)
  }

  return (
    <header className="flex h-16 shrink-0 items-center gap-3 border-b border-line/70 bg-bg/80 px-6 backdrop-blur">
      <form onSubmit={submit} className="relative w-full max-w-md">
        <Search
          size={15}
          className="pointer-events-none absolute top-1/2 left-3 -translate-y-1/2 text-faint"
        />
        <input
          id="topbar-search"
          ref={inputRef}
          value={q}
          onChange={(e) => setQ(e.target.value)}
          placeholder="Search anything.."
          aria-label="Search entries"
          className="input py-2 pr-14 pl-9"
        />
        <span className="pointer-events-none absolute top-1/2 right-3 -translate-y-1/2">
          <span className="kbd">⌘K</span>
        </span>
      </form>

      <div className="ml-auto flex items-center gap-2">
        <ThemeToggle />
        <button type="button" onClick={() => navigate('/entries/new')} className="btn btn-accent">
          <Plus size={16} strokeWidth={2.25} />
          New entry
        </button>
      </div>
    </header>
  )
}
