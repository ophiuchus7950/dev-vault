import { useQuery } from '@tanstack/react-query'
import { Download, HardDriveDownload, Loader2, Upload } from 'lucide-react'
import { useState } from 'react'
import { downloadExport, getHealth, importVault } from '../../api/endpoints'
import type { ImportSummary } from '../../api/types'
import { cn } from '../../lib/cn'
import { type ThemePref, useTheme } from '../../stores/theme'
import { toast } from '../../stores/toast'

const THEME_OPTIONS: { value: ThemePref; label: string; hint: string }[] = [
  { value: 'dark', label: 'Dark', hint: 'The default. Easier on the eyes at 2am.' },
  { value: 'light', label: 'Light', hint: 'For well-lit rooms and screenshots.' },
  { value: 'system', label: 'System', hint: 'Follow your OS setting.' },
]

export function SettingsPage() {
  const theme = useTheme()
  const [onConflict, setOnConflict] = useState<'skip' | 'overwrite'>('skip')
  const [importing, setImporting] = useState(false)
  const [summary, setSummary] = useState<ImportSummary | null>(null)

  const { data: health } = useQuery({ queryKey: ['health'], queryFn: getHealth, staleTime: 10_000 })

  const doExport = async (format: 'json' | 'zip') => {
    try {
      await downloadExport(format)
      toast(`Vault exported as ${format.toUpperCase()}`, 'success')
    } catch (e) {
      toast((e as Error).message, 'error')
    }
  }

  const doImport = async (file: File) => {
    setImporting(true)
    setSummary(null)
    try {
      const result = await importVault(file, onConflict)
      setSummary(result)
      toast('Import finished', 'success')
    } catch (e) {
      toast((e as Error).message, 'error')
    } finally {
      setImporting(false)
    }
  }

  return (
    <div className="mx-auto max-w-2xl px-8 py-8">
      <p className="eyebrow">Preferences</p>
      <h1 className="mt-2 font-display text-4xl font-semibold tracking-tight">Settings</h1>

      <section className="card mt-6 p-5">
        <h2 className="font-semibold">Appearance</h2>
        <div className="mt-3 grid gap-2 sm:grid-cols-3">
          {THEME_OPTIONS.map((option) => (
            <button
              key={option.value}
              type="button"
              onClick={() => theme.setPref(option.value)}
              aria-pressed={theme.pref === option.value}
              className={cn(
                'cursor-pointer rounded-xl border p-3 text-left transition-colors',
                theme.pref === option.value
                  ? 'border-accent bg-accent/10'
                  : 'border-line hover:border-linestrong',
              )}
            >
              <p className="text-sm font-medium">{option.label}</p>
              <p className="mt-0.5 text-xs text-faint">{option.hint}</p>
            </button>
          ))}
        </div>
      </section>

      <section className="card mt-4 p-5">
        <h2 className="font-semibold">Export</h2>
        <p className="mt-1 text-sm text-mute">
          Download the whole vault — portable JSON, or a ZIP with Markdown files.
        </p>
        <div className="mt-3 flex gap-2">
          <button type="button" className="btn btn-secondary" onClick={() => doExport('json')}>
            <Download size={14} />
            Vault JSON
          </button>
          <button type="button" className="btn btn-secondary" onClick={() => doExport('zip')}>
            <HardDriveDownload size={14} />
            Vault ZIP
          </button>
        </div>
      </section>

      <section className="card mt-4 p-5">
        <h2 className="font-semibold">Import</h2>
        <p className="mt-1 text-sm text-mute">Restore a previously exported vault JSON file.</p>
        <div className="mt-3 flex flex-wrap items-center gap-2">
          <select
            className="input w-auto"
            value={onConflict}
            aria-label="Conflict strategy"
            onChange={(e) => setOnConflict(e.target.value as 'skip' | 'overwrite')}
          >
            <option value="skip">Skip existing</option>
            <option value="overwrite">Overwrite existing</option>
          </select>
          <label className="btn btn-secondary cursor-pointer">
            <Upload size={14} />
            {importing ? <Loader2 size={14} className="animate-spin" /> : 'Choose file…'}
            <input
              type="file"
              accept="application/json"
              className="hidden"
              onChange={(e) => {
                const file = e.target.files?.[0]
                if (file) doImport(file)
                e.target.value = ''
              }}
            />
          </label>
        </div>
        {summary ? (
          <p className="mt-3 text-sm text-mute">
            Imported <span className="font-mono text-ink">{summary.imported}</span>, skipped{' '}
            <span className="font-mono text-ink">{summary.skipped}</span>, overwrote{' '}
            <span className="font-mono text-ink">{summary.overwritten}</span>.
            {summary.errors.length > 0 ? (
              <span className="text-danger"> {summary.errors.length} errors.</span>
            ) : null}
          </p>
        ) : null}
      </section>

      <section className="card mt-4 p-5">
        <h2 className="font-semibold">System</h2>
        <dl className="mt-3 space-y-1.5 font-mono text-[13px]">
          <div className="flex justify-between gap-4">
            <dt className="text-faint">api</dt>
            <dd>{health ? `${health.status} · v${health.version}` : 'unreachable'}</dd>
          </div>
          <div className="flex justify-between gap-4">
            <dt className="text-faint">uptime</dt>
            <dd>{health ? `${health.uptime_s}s` : '—'}</dd>
          </div>
          <div className="flex justify-between gap-4">
            <dt className="text-faint">workspace</dt>
            <dd>local · single-user · no auth</dd>
          </div>
        </dl>
      </section>
    </div>
  )
}
