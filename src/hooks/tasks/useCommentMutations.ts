import { useState } from 'react'
import { QUERY_KEYS } from '@/constants/cache'
import { useInvokeMutation } from '@/hooks/api'

export function useCommentMutations(taskId: number) {
  const [editingId, setEditingId] = useState<number | null>(null)
  const [editBody, setEditBody] = useState('')
  const [replyingToId, setReplyingToId] = useState<number | null>(null)
  const [replyBody, setReplyBody] = useState('')
  const [replyPriority, setReplyPriority] = useState<string | null>(null)

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

  const replyToCommentMutation = useInvokeMutation<{
    taskId: number
    parentCommentId: number
    priority: string | null
    body: string
  }>('add_reply_to_comment', {
    invalidateKeys: QUERY_KEYS.TASKS,
    onSuccess: () => {
      setReplyingToId(null)
      setReplyBody('')
      setReplyPriority(null)
    }
  })

  const error = addCommentMutation.error ?? editComment.error ?? deleteComment.error ?? replyToCommentMutation.error
  const resetError = () => {
    addCommentMutation.reset()
    editComment.reset()
    deleteComment.reset()
    replyToCommentMutation.reset()
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
    startReply: (commentId: number) => {
      setReplyingToId(commentId)
      setReplyBody('')
      setReplyPriority(null)
    },
    cancelReply: () => {
      setReplyingToId(null)
      setReplyBody('')
      setReplyPriority(null)
    },
    submitReply: () => {
      if (replyingToId === null || !replyBody.trim()) return
      replyToCommentMutation.mutate({
        taskId,
        parentCommentId: replyingToId,
        priority: replyPriority,
        body: replyBody.trim()
      })
    },
    editingId,
    editBody,
    setEditBody,
    replyingToId,
    replyBody,
    setReplyBody,
    replyPriority,
    setReplyPriority,
    isPending:
      addCommentMutation.isPending ||
      editComment.isPending ||
      deleteComment.isPending ||
      replyToCommentMutation.isPending,
    error,
    resetError
  }
}
