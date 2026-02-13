import { useState } from 'react'
import { QUERY_KEYS } from '@/constants/cache'
import { useInvokeMutation } from '@/hooks/api'

export function useSubsystemCommentMutations(subsystemName: string) {
  const [editingId, setEditingId] = useState<number | null>(null)
  const [editBody, setEditBody] = useState('')
  const [editSummary, setEditSummary] = useState('')
  const [editReason, setEditReason] = useState('')

  const addCommentMutation = useInvokeMutation<{
    subsystemName: string
    category: string
    body: string
    summary?: string
    reason?: string
  }>('add_subsystem_comment', {
    invalidateKeys: QUERY_KEYS.SUBSYSTEMS
  })

  const editComment = useInvokeMutation<{
    subsystemName: string
    commentId: number
    body: string
    summary?: string
    reason?: string
  }>('update_subsystem_comment', {
    invalidateKeys: QUERY_KEYS.SUBSYSTEMS,
    onSuccess: () => {
      setEditingId(null)
      setEditBody('')
      setEditSummary('')
      setEditReason('')
    }
  })

  const deleteComment = useInvokeMutation<{
    subsystemName: string
    commentId: number
  }>('delete_subsystem_comment', {
    invalidateKeys: QUERY_KEYS.SUBSYSTEMS
  })

  const error = addCommentMutation.error ?? editComment.error ?? deleteComment.error
  const resetError = () => {
    addCommentMutation.reset()
    editComment.reset()
    deleteComment.reset()
  }

  return {
    addComment: addCommentMutation,
    startEdit: (commentId: number, body: string, summary: string, reason: string) => {
      setEditingId(commentId)
      setEditBody(body)
      setEditSummary(summary)
      setEditReason(reason)
    },
    cancelEdit: () => {
      setEditingId(null)
      setEditBody('')
      setEditSummary('')
      setEditReason('')
    },
    submitEdit: () => {
      if (editingId === null || !editBody.trim()) return
      editComment.mutate({
        subsystemName,
        commentId: editingId,
        body: editBody.trim(),
        summary: editSummary.trim() || undefined,
        reason: editReason.trim() || undefined
      })
    },
    deleteComment: (commentId: number) => deleteComment.mutate({ subsystemName, commentId }),
    editingId,
    editBody,
    setEditBody,
    editSummary,
    setEditSummary,
    editReason,
    setEditReason,
    isPending: addCommentMutation.isPending || editComment.isPending || deleteComment.isPending,
    error,
    resetError
  }
}
