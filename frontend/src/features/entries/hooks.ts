import {
  type InfiniteData,
  useInfiniteQuery,
  useMutation,
  useQuery,
  useQueryClient,
} from '@tanstack/react-query'
import { ApiError, type HttpResult } from '../../api/client'
import {
  createEntry,
  deleteEntry,
  getEntry,
  listEntries,
  purgeEntry,
  restoreEntry,
  updateEntry,
} from '../../api/endpoints'
import type { Entry, EntryDraft, EntryListParams, Page } from '../../api/types'
import { toast } from '../../stores/toast'
import { entryKeys } from './keys'

export function useEntries(filters: EntryListParams) {
  return useInfiniteQuery({
    queryKey: entryKeys.list(filters),
    queryFn: ({ pageParam }) => listEntries({ ...filters, cursor: pageParam }),
    initialPageParam: undefined as string | undefined,
    getNextPageParam: (last) => last.data.next_cursor ?? undefined,
  })
}

export function useEntry(id: string | undefined) {
  return useQuery({
    queryKey: entryKeys.detail(id ?? ''),
    queryFn: () => getEntry(id as string),
    enabled: Boolean(id),
  })
}

export function useCreateEntry() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (draft: EntryDraft) => createEntry(draft),
    onSuccess: () => {
      toast('Entry created', 'success')
      qc.invalidateQueries({ queryKey: ['entries'] })
      qc.invalidateQueries({ queryKey: ['search'] })
      qc.invalidateQueries({ queryKey: ['collections'] })
      qc.invalidateQueries({ queryKey: ['vault-count'] })
    },
    onError: (e: Error) => toast(e.message, 'error'),
  })
}

export function useUpdateEntry(id: string) {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (vars: { patch: Partial<EntryDraft>; etag: string }) =>
      updateEntry(id, vars.patch, vars.etag),
    onSuccess: (res: HttpResult<Entry>) => {
      qc.setQueryData(entryKeys.detail(id), res)
      toast('Saved', 'success')
      qc.invalidateQueries({ queryKey: ['entries'] })
      qc.invalidateQueries({ queryKey: ['search'] })
      qc.invalidateQueries({ queryKey: ['collections'] })
    },
    onError: (e: unknown) => {
      if (e instanceof ApiError && e.status === 409) {
        qc.invalidateQueries({ queryKey: entryKeys.detail(id) })
        toast('Changed elsewhere — latest version reloaded', 'error')
      } else if (e instanceof Error) {
        toast(e.message, 'error')
      }
    },
  })
}

type EntriesSnapshot = [readonly unknown[], InfiniteData<HttpResult<Page<Entry>>> | undefined][]

function useEntriesRollback() {
  const qc = useQueryClient()
  return async () => {
    await qc.cancelQueries({ queryKey: ['entries'] })
    return qc.getQueriesData<InfiniteData<HttpResult<Page<Entry>>>>({ queryKey: ['entries'] })
  }
}

function restoreEntries(qc: ReturnType<typeof useQueryClient>, snapshots: EntriesSnapshot) {
  for (const [key, value] of snapshots) qc.setQueryData(key, value)
}

function withTransformedEntry(
  pages: InfiniteData<HttpResult<Page<Entry>>>['pages'],
  id: string,
  transform: (e: Entry) => Entry,
) {
  return pages.map((p) => ({
    ...p,
    data: {
      ...p.data,
      data: p.data.data.map((e) => (e.id === id ? transform(e) : e)),
    },
  }))
}

export function useTogglePin() {
  const qc = useQueryClient()
  const snapshot = useEntriesRollback()
  return useMutation({
    mutationFn: async (entry: Entry) => {
      const cached = qc.getQueryData<HttpResult<Entry>>(entryKeys.detail(entry.id))
      const etag = cached?.etag ?? (await getEntry(entry.id)).etag
      if (!etag) throw new Error('Missing ETag for entry')
      return updateEntry(entry.id, { is_pinned: !entry.is_pinned }, etag)
    },
    onMutate: async (entry: Entry) => {
      const snapshots = await snapshot()
      for (const [key, value] of snapshots) {
        if (!value) continue
        qc.setQueryData(key, {
          ...value,
          pages: withTransformedEntry(value.pages, entry.id, (e) => ({ ...e, is_pinned: !entry.is_pinned })),
        })
      }
      return { snapshots }
    },
    onError: (_e: unknown, _entry: Entry, ctx) => {
      if (ctx) restoreEntries(qc, ctx.snapshots)
      toast('Could not update star', 'error')
    },
    onSuccess: (_res, entry) => {
      toast(entry.is_pinned ? 'Unstarred' : 'Starred', 'success')
      qc.invalidateQueries({ queryKey: ['entries'] })
      qc.invalidateQueries({ queryKey: ['search'] })
      qc.invalidateQueries({ queryKey: ['vault-count'] })
      qc.invalidateQueries({ queryKey: entryKeys.detail(entry.id) })
    },
  })
}

export function useSoftDeleteEntry() {
  const qc = useQueryClient()
  const snapshot = useEntriesRollback()
  return useMutation({
    mutationFn: (id: string) => deleteEntry(id),
    onMutate: async (id: string) => {
      const snapshots = await snapshot()
      for (const [key, value] of snapshots) {
        if (!value) continue
        qc.setQueryData(key, {
          ...value,
          pages: value.pages.map((p) => ({
            ...p,
            data: { ...p.data, data: p.data.data.filter((e) => e.id !== id) },
          })),
        })
      }
      return { snapshots }
    },
    onError: (_e: unknown, _id: string, ctx) => {
      if (ctx) restoreEntries(qc, ctx.snapshots)
      toast('Delete failed', 'error')
    },
    onSuccess: () => {
      toast('Moved to trash', 'success')
      qc.invalidateQueries({ queryKey: ['entries'] })
      qc.invalidateQueries({ queryKey: ['search'] })
      qc.invalidateQueries({ queryKey: ['trash'] })
      qc.invalidateQueries({ queryKey: ['entry'] })
      qc.invalidateQueries({ queryKey: ['collections'] })
      qc.invalidateQueries({ queryKey: ['vault-count'] })
    },
  })
}

export function useRestoreEntry() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (id: string) => restoreEntry(id),
    onSuccess: () => {
      toast('Restored', 'success')
      qc.invalidateQueries({ queryKey: ['trash'] })
      qc.invalidateQueries({ queryKey: ['entries'] })
      qc.invalidateQueries({ queryKey: ['vault-count'] })
    },
    onError: (e: Error) => toast(e.message, 'error'),
  })
}

export function usePurgeEntry() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (id: string) => purgeEntry(id),
    onSuccess: () => {
      toast('Deleted permanently', 'success')
      qc.invalidateQueries({ queryKey: ['trash'] })
    },
    onError: (e: Error) => toast(e.message, 'error'),
  })
}
