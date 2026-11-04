import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import {
  assignEntries,
  createCollection,
  deleteCollection,
  getCollection,
  listCollections,
  unassignEntries,
  updateCollection,
} from '../../api/endpoints'
import { toast } from '../../stores/toast'

export function useCollections() {
  return useQuery({
    queryKey: ['collections'],
    queryFn: listCollections,
    staleTime: 15_000,
  })
}

export function useCollection(slug: string | undefined) {
  return useQuery({
    queryKey: ['collection', slug],
    queryFn: () => getCollection(slug as string),
    enabled: Boolean(slug),
  })
}

export function useCreateCollection() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (body: { name: string; description?: string }) => createCollection(body),
    onSuccess: () => {
      toast('Collection created', 'success')
      qc.invalidateQueries({ queryKey: ['collections'] })
    },
    onError: (e: Error) => toast(e.message, 'error'),
  })
}

export function useUpdateCollection() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (vars: { slug: string; patch: { name?: string; description?: string } }) =>
      updateCollection(vars.slug, vars.patch),
    onSuccess: () => {
      toast('Collection updated', 'success')
      qc.invalidateQueries({ queryKey: ['collections'] })
      qc.invalidateQueries({ queryKey: ['collection'] })
    },
    onError: (e: Error) => toast(e.message, 'error'),
  })
}

export function useDeleteCollection() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (slug: string) => deleteCollection(slug),
    onSuccess: () => {
      toast('Collection deleted', 'success')
      qc.invalidateQueries({ queryKey: ['collections'] })
    },
    onError: (e: Error) => toast(e.message, 'error'),
  })
}

export function useAssignEntries() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (vars: { collectionId: string; entryIds: string[] }) =>
      assignEntries(vars.collectionId, vars.entryIds),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['collections'] })
      qc.invalidateQueries({ queryKey: ['collection'] })
      qc.invalidateQueries({ queryKey: ['entries'] })
      qc.invalidateQueries({ queryKey: ['entry'] })
    },
    onError: (e: Error) => toast(e.message, 'error'),
  })
}

export function useUnassignEntries() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (vars: { collectionId: string; entryIds: string[] }) =>
      unassignEntries(vars.collectionId, vars.entryIds),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['collections'] })
      qc.invalidateQueries({ queryKey: ['collection'] })
      qc.invalidateQueries({ queryKey: ['entries'] })
      qc.invalidateQueries({ queryKey: ['entry'] })
    },
    onError: (e: Error) => toast(e.message, 'error'),
  })
}
