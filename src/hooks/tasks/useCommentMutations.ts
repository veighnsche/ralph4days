import { useState } from 'react'
import { QUERY_KEYS } from '@/constants/cache'
import { useInvokeMutation } from '@/hooks/api'

export function useCommentMutations(taskId: number) {
  const [editingId, setEditingId] = useState<number | null>(null)
  const [editBody, setEditBody] = useState('')

  const addCommentMutation = useInvokeMutation<{ taskId: number; body: string }>('add_task_comment', {
    invalidateKeys: QUERY_KEYS.TASKS
  })

  const editComment = useInvokeMutation<{ taskId: number; commentId: number; body: string }>('update_task_comment', {
    invalidateKeys: QUERY_KEYS.TASKS,
    onSuccess: () => {
      setEditingId(null)
      setEditBody('')
    }
  })

  const deleteComment = useInvokeMutation<{ taskId: number; commentId: number }>('delete_task_comment', {
    invalidateKeys: QUERY_KEYS.TASKS
  })

  const error = addCommentMutation.error ?? editComment.error ?? deleteComment.error
  const resetError = () => {
    addCommentMutation.reset()
    editComment.reset()
    deleteComment.reset()
  }

  return {
    addComment: addCommentMutation,
    startEdit: (commentId: number, body: string) => {
      setEditingId(commentId)
      setEditBody(body)
    },
    cancelEdit: () => {
      setEditingId(null)
      setEditBody('')
    },
    submitEdit: () => {
      if (editingId === null || !editBody.trim()) return
      editComment.mutate({ taskId, commentId: editingId, body: editBody.trim() })
    },
    deleteComment: (commentId: number) => deleteComment.mutate({ taskId, commentId }),
    editingId,
    editBody,
    setEditBody,
    isPending: addCommentMutation.isPending || editComment.isPending || deleteComment.isPending,
    error,
    resetError
  }
}
