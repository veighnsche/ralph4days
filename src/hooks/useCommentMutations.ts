import { useState } from 'react'
import { toast } from 'sonner'
import { useInvokeMutation } from '@/hooks/useInvokeMutation'

const INVALIDATE_KEYS = [['get_tasks']]

export function useCommentMutations(taskId: number) {
  const [editingId, setEditingId] = useState<number | null>(null)
  const [editBody, setEditBody] = useState('')

  const addCommentMutation = useInvokeMutation<{ taskId: number; author: string; body: string }>('add_task_comment', {
    invalidateKeys: INVALIDATE_KEYS,
    onSuccess: () => toast.success('Comment added'),
    onError: err => toast.error(err.message)
  })

  const editComment = useInvokeMutation<{ taskId: number; commentId: number; body: string }>('update_task_comment', {
    invalidateKeys: INVALIDATE_KEYS,
    onSuccess: () => {
      setEditingId(null)
      setEditBody('')
    },
    onError: err => toast.error(err.message)
  })

  const deleteComment = useInvokeMutation<{ taskId: number; commentId: number }>('delete_task_comment', {
    invalidateKeys: INVALIDATE_KEYS,
    onSuccess: () => toast.success('Comment deleted'),
    onError: err => toast.error(err.message)
  })

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
    isPending: addCommentMutation.isPending || editComment.isPending || deleteComment.isPending
  }
}
