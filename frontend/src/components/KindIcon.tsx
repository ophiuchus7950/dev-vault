import { cn } from '../lib/cn'
import { type EntryKind, KIND_META } from '../lib/kinds'

export function KindIcon({ kind, className }: { kind: string; className?: string }) {
  const meta = KIND_META[kind as EntryKind] ?? KIND_META.note
  const Icon = meta.icon
  return (
    <span className={cn('icon-tile', className)} aria-hidden>
      <Icon size={16} strokeWidth={1.75} />
    </span>
  )
}
