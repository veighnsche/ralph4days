import { useState } from 'react'
import { type InvokeQueryDomain, useInvokeMutation } from '@/hooks/api'
import type { Task } from '@/types/generated'
import { buildTaskListItemFromTask, patchTaskInTaskDetailCache, patchTaskListItemInTaskListCache } from './taskCache'

export function useSignalMutations(taskId: number, queryDomain: InvokeQueryDomain = 'app') {
  const [editingId, setEditingId] = useState<number | null>(null)
  const [editBody, setEditBody] = useState('')
  const [replyingToId, setReplyingToId] = useState<number | null>(null)
  const [replyBody, setReplyBody] = useState('')
  const [replyPriority, setReplyPriority] = useState<string | undefined>(undefined)

  const addSignalMutation = useInvokeMutation<{ taskId: number; body: string }, Task>('tasks_signal_add', {
    queryDomain,
    updateCache: ({ queryClient, data, queryDomain }) => {
      patchTaskInTaskDetailCache(queryClient, data, queryDomain)
      patchTaskListItemInTaskListCache(queryClient, buildTaskListItemFromTask(data), queryDomain)
    }
  })

  const editSignal = useInvokeMutation<{ taskId: number; signalId: number; body: string }, Task>(
    'tasks_signal_update',
    {
      queryDomain,
      updateCache: ({ queryClient, data, queryDomain }) => {
        patchTaskInTaskDetailCache(queryClient, data, queryDomain)
        patchTaskListItemInTaskListCache(queryClient, buildTaskListItemFromTask(data), queryDomain)
      },
      onSuccess: () => {
        setEditingId(null)
        setEditBody('')
      }
    }
  )

  const deleteSignal = useInvokeMutation<{ taskId: number; signalId: number }, Task>('tasks_signal_delete', {
    queryDomain,
    updateCache: ({ queryClient, data, queryDomain }) => {
      patchTaskInTaskDetailCache(queryClient, data, queryDomain)
      patchTaskListItemInTaskListCache(queryClient, buildTaskListItemFromTask(data), queryDomain)
    }
  })

  const replyToSignalMutation = useInvokeMutation<
    {
      taskId: number
      parentCommentId: number
      priority?: string
      body: string
    },
    Task
  >('tasks_comment_reply_add', {
    queryDomain,
    updateCache: ({ queryClient, data, queryDomain }) => {
      patchTaskInTaskDetailCache(queryClient, data, queryDomain)
      patchTaskListItemInTaskListCache(queryClient, buildTaskListItemFromTask(data), queryDomain)
    },
    onSuccess: () => {
      setReplyingToId(null)
      setReplyBody('')
      setReplyPriority(undefined)
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
      setReplyPriority(undefined)
    },
    cancelReply: () => {
      setReplyingToId(null)
      setReplyBody('')
      setReplyPriority(undefined)
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
