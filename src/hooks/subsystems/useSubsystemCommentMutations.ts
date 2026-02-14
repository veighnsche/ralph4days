import { useState } from 'react'
import { type InvokeQueryDomain, useInvokeMutation } from '@/hooks/api'
import type { SubsystemData } from '@/types/generated'
import { patchSubsystemInCache } from './subsystemCache'

export function useSubsystemCommentMutations(subsystemName: string, queryDomain: InvokeQueryDomain = 'app') {
  const [editingId, setEditingId] = useState<number | null>(null)
  const [editBody, setEditBody] = useState('')
  const [editSummary, setEditSummary] = useState('')
  const [editReason, setEditReason] = useState('')

  const addCommentMutation = useInvokeMutation<
    {
      subsystemName: string
      category: string
      body: string
      summary?: string
      reason?: string
    },
    SubsystemData
  >('subsystems_comment_add', {
    queryDomain,
    updateCache: ({ queryClient, data, queryDomain }) => patchSubsystemInCache(queryClient, data, queryDomain)
  })

  const editComment = useInvokeMutation<
    {
      subsystemName: string
      commentId: number
      body: string
      summary?: string
      reason?: string
    },
    SubsystemData
  >('subsystems_comment_update', {
    queryDomain,
    updateCache: ({ queryClient, data, queryDomain }) => patchSubsystemInCache(queryClient, data, queryDomain),
    onSuccess: () => {
      setEditingId(null)
      setEditBody('')
      setEditSummary('')
      setEditReason('')
    }
  })

  const deleteComment = useInvokeMutation<
    {
      subsystemName: string
      commentId: number
    },
    SubsystemData
  >('subsystems_comment_delete', {
    queryDomain,
    updateCache: ({ queryClient, data, queryDomain }) => patchSubsystemInCache(queryClient, data, queryDomain)
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
