import type { LucideIcon } from 'lucide-react'
import type { ReactNode } from 'react'
import { cn } from '../lib/cn'

interface EmptyStateProps {
  icon: LucideIcon
  title: string
  hint?: string
  action?: ReactNode
  className?: string
}

export function EmptyState({ icon: Icon, title, hint, action, className }: EmptyStateProps) {
  return (
    <div className={cn('flex flex-col items-center justify-center gap-3 px-6 py-16 text-center', className)}>
      <span className="icon-tile size-12 text-faint">
        <Icon size={22} strokeWidth={1.5} />
      </span>
      <div>
        <p className="font-medium text-ink">{title}</p>
        {hint ? <p className="mt-1 max-w-sm text-sm text-mute">{hint}</p> : null}
      </div>
      {action}
    </div>
  )
}
