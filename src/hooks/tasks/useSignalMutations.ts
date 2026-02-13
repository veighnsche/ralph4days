import { useState } from 'react'
import { type InvokeQueryDomain, useInvokeMutation } from '@/hooks/api'
import type { Task } from '@/types/generated'
import { patchTaskInTasksCache } from './taskCache'

export function useSignalMutations(taskId: number, queryDomain: InvokeQueryDomain = 'app') {
  const [editingId, setEditingId] = useState<number | null>(null)
  const [editBody, setEditBody] = useState('')
  const [replyingToId, setReplyingToId] = useState<number | null>(null)
  const [replyBody, setReplyBody] = useState('')
  const [replyPriority, setReplyPriority] = useState<string | null>(null)

  const addSignalMutation = useInvokeMutation<{ taskId: number; body: string }, Task>('add_task_signal', {
    queryDomain,
    updateCache: ({ queryClient, data, queryDomain }) => patchTaskInTasksCache(queryClient, data, queryDomain)
  })

  const editSignal = useInvokeMutation<{ taskId: number; signalId: number; body: string }, Task>('update_task_signal', {
    queryDomain,
    updateCache: ({ queryClient, data, queryDomain }) => patchTaskInTasksCache(queryClient, data, queryDomain),
    onSuccess: () => {
      setEditingId(null)
      setEditBody('')
    }
  })

  const deleteSignal = useInvokeMutation<{ taskId: number; signalId: number }, Task>('delete_task_signal', {
    queryDomain,
    updateCache: ({ queryClient, data, queryDomain }) => patchTaskInTasksCache(queryClient, data, queryDomain)
  })

  const replyToSignalMutation = useInvokeMutation<
    {
      taskId: number
      parentCommentId: number
      priority: string | null
      body: string
    },
    Task
  >('add_reply_to_comment', {
    queryDomain,
    updateCache: ({ queryClient, data, queryDomain }) => patchTaskInTasksCache(queryClient, data, queryDomain),
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
      addSignalMutation.isPending || editSignal.isPending || deleteSignal.isPending || replyToSignalMutation.isPending,
    error,
    resetError
  }
}
