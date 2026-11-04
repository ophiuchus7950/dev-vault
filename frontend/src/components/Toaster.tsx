import { X } from 'lucide-react'
import { cn } from '../lib/cn'
import { useToast } from '../stores/toast'

export function Toaster() {
  const { toasts, dismiss } = useToast()
  return (
    <div role="status" aria-live="polite" className="fixed right-4 bottom-4 z-50 flex flex-col gap-2">
      {toasts.map((t) => (
        <div
          key={t.id}
          className={cn(
            'card flex items-center gap-3 px-4 py-2.5 text-sm shadow-xl shadow-black/20',
            t.tone === 'error' && 'border-danger/50',
            t.tone === 'success' && 'border-accent/40',
          )}
        >
          <span
            className={cn(
              'size-1.5 shrink-0 rounded-full',
              t.tone === 'error' ? 'bg-danger' : t.tone === 'success' ? 'bg-accent' : 'bg-linestrong',
            )}
          />
          <span>{t.message}</span>
          <button
            type="button"
            onClick={() => dismiss(t.id)}
            aria-label="Dismiss"
            className="cursor-pointer text-faint hover:text-ink"
          >
            <X size={14} />
          </button>
        </div>
      ))}
    </div>
  )
}
