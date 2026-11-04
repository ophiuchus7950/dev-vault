export const entryKeys = {
  all: ['entries'] as const,
  list: (filters: unknown) => ['entries', filters] as const,
  detail: (id: string) => ['entry', id] as const,
}

export const tagKeys = {
  all: ['tags'] as const,
}

export const collectionKeys = {
  all: ['collections'] as const,
  detail: (slug: string) => ['collection', slug] as const,
}

export const trashKeys = {
  all: ['trash'] as const,
}
