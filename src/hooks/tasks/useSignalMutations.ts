import { useState } from 'react'
import { QUERY_KEYS } from '@/constants/cache'
import { useInvokeMutation } from '@/hooks/api'

export function useSignalMutations(taskId: number) {
  const [editingId, setEditingId] = useState<number | null>(null)
  const [editBody, setEditBody] = useState('')
  const [replyingToId, setReplyingToId] = useState<number | null>(null)
  const [replyBody, setReplyBody] = useState('')
  const [replyPriority, setReplyPriority] = useState<string | null>(null)

  const addSignalMutation = useInvokeMutation<{ taskId: number; body: string }>('add_task_signal', {
    invalidateKeys: QUERY_KEYS.TASKS
  })

  const editSignal = useInvokeMutation<{ taskId: number; signalId: number; body: string }>('update_task_signal', {
    invalidateKeys: QUERY_KEYS.TASKS,
    onSuccess: () => {
      setEditingId(null)
      setEditBody('')
    }
  })

  const deleteSignal = useInvokeMutation<{ taskId: number; signalId: number }>('delete_task_signal', {
    invalidateKeys: QUERY_KEYS.TASKS
  })

  const replyToSignalMutation = useInvokeMutation<{
    taskId: number
    parentSignalId: number
    priority: string | null
    body: string
  }>('add_reply_to_signal', {
    invalidateKeys: QUERY_KEYS.TASKS,
    onSuccess: () => {
      setReplyingToId(null)
      setReplyBody('')
      setReplyPriority(null)
    }
  })

  const error = addSignalMutation.error ?? editSignal.error ?? deleteSignal.error ?? replyToSignalMutation.error
  const resetError = () => {
    addSignalMutation.reset()
    editSignal.reset()
    deleteSignal.reset()
    replyToSignalMutation.reset()
  }

  return {
    addSignal: addSignalMutation,
    startEdit: (signalId: number, body: string) => {
      setEditingId(signalId)
      setEditBody(body)
    },
    cancelEdit: () => {
      setEditingId(null)
      setEditBody('')
    },
    submitEdit: () => {
      if (editingId === null || !editBody.trim()) return
      editSignal.mutate({ taskId, signalId: editingId, body: editBody.trim() })
    },
    deleteSignal: (signalId: number) => deleteSignal.mutate({ taskId, signalId: signalId }),
    startReply: (signalId: number) => {
      setReplyingToId(signalId)
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
      replyToSignalMutation.mutate({
        taskId,
        parentSignalId: replyingToId,
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
      addSignalMutation.isPending || editSignal.isPending || deleteSignal.isPending || replyToSignalMutation.isPending,
    error,
    resetError
  }
}
