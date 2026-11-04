import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { deleteTag, listTags, renameTag } from '../../api/endpoints'
import { toast } from '../../stores/toast'

export function useTags(q = '') {
  return useQuery({
    queryKey: ['tags', q],
    queryFn: () => listTags(q),
    staleTime: 15_000,
  })
}

export function useRenameTag() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (vars: { id: string; name: string }) => renameTag(vars.id, vars.name),
    onSuccess: () => {
      toast('Tag renamed', 'success')
      qc.invalidateQueries({ queryKey: ['tags'] })
      qc.invalidateQueries({ queryKey: ['entries'] })
      qc.invalidateQueries({ queryKey: ['search'] })
      qc.invalidateQueries({ queryKey: ['entry'] })
    },
    onError: (e: Error) => toast(e.message, 'error'),
  })
}

export function useDeleteTag() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (id: string) => deleteTag(id),
    onSuccess: () => {
      toast('Tag deleted', 'success')
      qc.invalidateQueries({ queryKey: ['tags'] })
      qc.invalidateQueries({ queryKey: ['entries'] })
      qc.invalidateQueries({ queryKey: ['search'] })
      qc.invalidateQueries({ queryKey: ['entry'] })
    },
    onError: (e: Error) => toast(e.message, 'error'),
  })
}
