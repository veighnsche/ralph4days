import {
  AlertTriangle,
  Ban,
  CheckCircle2,
  Flag,
  HelpCircle,
  Lightbulb,
  MessageCircle,
  MessageSquare,
  Pencil,
  Reply,
  Trash2
} from 'lucide-react'
import { useState } from 'react'
import { DisciplineRadial, InlineError, NumberedIdDisplay, SignalAvatar } from '@/components/shared'
import { Button } from '@/components/ui/button'
import { useCommentMutations as useSignalMutations } from '@/hooks/tasks'
import { formatDate } from '@/lib/formatDate'
import type { Task, TaskSignal } from '@/types/generated'
import { ReplyCard } from './ReplyCard'
import { ReplyForm } from './ReplyForm'
import { SignalEditor } from './SignalEditor'

const VERB_CONFIG: Record<
  string,
  {
    icon: typeof CheckCircle2
    color: string
    radialColor: string
  }
> = {
  done: {
    icon: CheckCircle2,
    color: 'text-emerald-600 dark:text-emerald-500',
    radialColor: 'rgba(16, 185, 129, 0.15)'
  },
  partial: {
    icon: MessageCircle,
    color: 'text-amber-600 dark:text-amber-500',
    radialColor: 'rgba(245, 158, 11, 0.15)'
  },
  stuck: {
    icon: AlertTriangle,
    color: 'text-red-600 dark:text-red-500',
    radialColor: 'rgba(239, 68, 68, 0.15)'
  },
  ask: {
    icon: HelpCircle,
    color: 'text-blue-600 dark:text-blue-500',
    radialColor: 'rgba(59, 130, 246, 0.15)'
  },
  flag: {
    icon: Flag,
    color: 'text-orange-600 dark:text-orange-500',
    radialColor: 'rgba(249, 115, 22, 0.15)'
  },
  learned: {
    icon: Lightbulb,
    color: 'text-cyan-600 dark:text-cyan-500',
    radialColor: 'rgba(6, 182, 212, 0.15)'
  },
  suggest: {
    icon: MessageCircle,
    color: 'text-purple-600 dark:text-purple-500',
    radialColor: 'rgba(168, 85, 247, 0.15)'
  },
  blocked: {
    icon: Ban,
    color: 'text-red-600 dark:text-red-500',
    radialColor: 'rgba(239, 68, 68, 0.15)'
  }
}

interface SignalsSectionProps {
  task: Task
}

export function SignalsSection({ task }: SignalsSectionProps) {
  const allSignals = task.signals ?? []
  const [signalInput, setSignalInput] = useState('')

  const topLevel: TaskSignal[] = []
  const repliesByParent = new Map<number, TaskSignal[]>()

  for (const signal of allSignals) {
    if (signal.parent_signal_id) {
      const parentId = signal.parent_signal_id
      const existing = repliesByParent.get(parentId)
      if (existing) {
        existing.push(signal)
      } else {
        repliesByParent.set(parentId, [signal])
      }
    } else {
      topLevel.push(signal)
    }
  }

  topLevel.sort((a, b) => (b.created ?? '').localeCompare(a.created ?? ''))

  for (const replies of repliesByParent.values()) {
    replies.sort((a, b) => (a.created ?? '').localeCompare(b.created ?? ''))
  }

  const {
    addSignal,
    startEdit,
    cancelEdit,
    submitEdit,
    deleteSignal,
    startReply,
    cancelReply,
    submitReply,
    editingId,
    editBody,
    setEditBody,
    replyingToId,
    replyBody,
    setReplyBody,
    replyPriority,
    setReplyPriority,
    isPending,
    error,
    resetError
  } = useSignalMutations(task.id)

  const handleAddSignal = () => {
    if (!signalInput.trim()) return
    addSignal.mutate({ taskId: task.id, body: signalInput.trim() }, { onSuccess: () => setSignalInput('') })
  }

  const totalSignals = allSignals.length

  return (
    <div className="px-3 pb-1">
      <div className="flex items-center gap-1.5 mb-3">
        <MessageSquare className="h-3.5 w-3.5 text-muted-foreground" />
        <span className="text-sm font-medium text-muted-foreground">
          Signals{totalSignals > 0 && ` (${totalSignals})`}
        </span>
      </div>

      <InlineError error={error} onDismiss={resetError} className="mb-3" />

      <div className="mb-4">
        <SignalEditor
          value={signalInput}
          onChange={setSignalInput}
          onSubmit={handleAddSignal}
          submitLabel="Signal"
          placeholder="Add a signal..."
          disabled={isPending}
        />
      </div>

      {topLevel.length > 0 && (
        <div className="space-y-1">
          {topLevel.map(signal => {
            const isSignal = signal.signal_verb != null
            const replies = repliesByParent.get(signal.id) || []

            let signalConfig = null
            let SignalIcon = null

            if (isSignal && signal.signal_verb) {
              const verb = signal.signal_verb
              signalConfig = VERB_CONFIG[verb] || {
                icon: MessageCircle,
                color: 'text-muted-foreground',
                radialColor: 'rgba(128, 128, 128, 0.15)'
              }
              SignalIcon = signalConfig.icon
            }

            return (
              <div key={`signal-${signal.id}`} className="space-y-2">
                <div className="group/signal flex gap-3 relative overflow-hidden rounded-lg px-3 py-2.5 pb-8 border border-border/30">
                  {signalConfig && (
                    <div
                      className="absolute top-0 right-0 w-32 h-32 pointer-events-none"
                      style={{
                        background: `radial-gradient(circle at top right, ${signalConfig.radialColor} 0%, transparent 70%)`
                      }}
                    />
                  )}

                  {signalConfig && SignalIcon && (
                    <div className="absolute top-2 right-2 flex items-center gap-1.5">
                      <span className={`text-xs font-bold uppercase tracking-wider ${signalConfig.color}`}>
                        {signal.signal_verb}
                      </span>
                      <SignalIcon className={`h-4 w-4 ${signalConfig.color}`} />
                    </div>
                  )}

                  {!isSignal && (
                    <div className="absolute top-2 right-2 opacity-0 group-hover/signal:opacity-100 transition-opacity flex gap-0.5">
                      <Button
                        variant="ghost"
                        size="sm"
                        className="h-5 w-5 p-0"
                        onClick={() => startEdit(signal.id, signal.body)}>
                        <Pencil className="h-3 w-3 text-muted-foreground" />
                      </Button>
                      <Button variant="ghost" size="sm" className="h-5 w-5 p-0" onClick={() => deleteSignal(signal.id)}>
                        <Trash2 className="h-3 w-3 text-muted-foreground" />
                      </Button>
                    </div>
                  )}

                  <DisciplineRadial discipline={signal.author} />
                  <SignalAvatar discipline={signal.author} />

                  <div className="flex-1 min-w-0 space-y-2.5">
                    <div className="flex items-baseline gap-2">
                      <NumberedIdDisplay id={signal.id} variant="inline" />
                      <span className="text-sm font-medium">{signal.author ?? 'You'}</span>
                    </div>

                    {editingId === signal.id && !isSignal ? (
                      <div className="mt-1.5">
                        <SignalEditor
                          value={editBody}
                          onChange={setEditBody}
                          onSubmit={submitEdit}
                          onCancel={cancelEdit}
                          submitLabel="Save"
                          autoFocus
                        />
                      </div>
                    ) : (
                      <p className="text-sm leading-relaxed whitespace-pre-wrap text-foreground/90">{signal.body}</p>
                    )}
                  </div>

                  <div className="absolute bottom-1.5 left-0 right-0 flex items-center justify-between px-3">
                    <Button
                      variant="ghost"
                      size="sm"
                      className="h-6 text-xs text-muted-foreground hover:text-foreground"
                      onClick={() => startReply(signal.id)}>
                      <Reply className="h-3 w-3 mr-1" />
                      Reply
                    </Button>
                    {signal.created && (
                      <span className="text-xs text-muted-foreground/70">{formatDate(signal.created)}</span>
                    )}
                  </div>
                </div>

                {replyingToId === signal.id && (
                  <ReplyForm
                    value={replyBody}
                    onChange={setReplyBody}
                    onSubmit={submitReply}
                    onCancel={cancelReply}
                    priority={replyPriority}
                    onPriorityChange={setReplyPriority}
                  />
                )}

                {replies.map(reply => (
                  <ReplyCard key={`reply-${reply.id}`} reply={reply} />
                ))}
              </div>
            )
          })}
        </div>
      )}
    </div>
  )
}
