import { Moon, Sun } from 'lucide-react'
import { useTheme } from '../stores/theme'

export function ThemeToggle() {
  const pref = useTheme((s) => s.pref)
  const resolved = useTheme((s) => s.resolved)
  const cycle = useTheme((s) => s.cycle)
  const Icon = resolved === 'dark' ? Moon : Sun
  return (
    <button
      type="button"
      onClick={cycle}
      aria-label={`Theme: ${pref}. Click to change`}
      title={`Theme: ${pref}`}
      className="btn btn-ghost size-9 p-0"
    >
      <Icon size={17} />
    </button>
  )
}
