import { useState } from 'react'
import { QUERY_KEYS } from '@/constants/cache'
import { useInvokeMutation } from '@/hooks/api'

export function useFeatureCommentMutations(featureName: string) {
  const [editingId, setEditingId] = useState<number | null>(null)
  const [editBody, setEditBody] = useState('')
  const [editSummary, setEditSummary] = useState('')
  const [editReason, setEditReason] = useState('')

  const addCommentMutation = useInvokeMutation<{
    featureName: string
    category: string
    body: string
    summary?: string
    reason?: string
  }>('add_feature_comment', {
    invalidateKeys: QUERY_KEYS.FEATURES
  })

  const editComment = useInvokeMutation<{
    featureName: string
    commentId: number
    body: string
    summary?: string
    reason?: string
  }>('update_feature_comment', {
    invalidateKeys: QUERY_KEYS.FEATURES,
    onSuccess: () => {
      setEditingId(null)
      setEditBody('')
      setEditSummary('')
      setEditReason('')
    }
  })

  const deleteComment = useInvokeMutation<{
    featureName: string
    commentId: number
  }>('delete_feature_comment', {
    invalidateKeys: QUERY_KEYS.FEATURES
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
        featureName,
        commentId: editingId,
        body: editBody.trim(),
        summary: editSummary.trim() || undefined,
        reason: editReason.trim() || undefined
      })
    },
    deleteComment: (commentId: number) => deleteComment.mutate({ featureName, commentId }),
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
