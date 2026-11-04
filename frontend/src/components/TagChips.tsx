import { cn } from '../lib/cn'

interface TagChipsProps {
  tags: string[]
  hash?: boolean
  onTagClick?: (tag: string) => void
  className?: string
}

export function TagChips({ tags, hash = false, onTagClick, className }: TagChipsProps) {
  if (tags.length === 0) return null
  return (
    <div className={cn('flex flex-wrap gap-1.5', className)}>
      {tags.map((tag) => (
        <button
          key={tag}
          type="button"
          onClick={onTagClick ? () => onTagClick(tag) : undefined}
          className={cn('chip', onTagClick && 'cursor-pointer hover:border-linestrong hover:text-ink')}
        >
          {hash ? `#${tag}` : tag}
        </button>
      ))}
    </div>
  )
}
