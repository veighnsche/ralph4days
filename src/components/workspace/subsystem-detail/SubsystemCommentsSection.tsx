import type { LucideIcon } from 'lucide-react'
import {
  AlertTriangle,
  Blocks,
  BookOpen,
  GitFork,
  Lightbulb,
  MessageSquare,
  Pencil,
  Send,
  Shield,
  Trash2
} from 'lucide-react'
import { useRef, useState } from 'react'
import { DisciplineRadial, InlineError, SignalAvatar } from '@/components/shared'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import { SignalEditor } from '@/components/workspace/task-detail/components/SignalEditor'
import { useSubsystemCommentMutations } from '@/hooks/subsystems'
import { formatDate } from '@/lib/formatDate'
import type { SubsystemData } from '@/types/generated'

const CATEGORY_CONFIG: Record<string, { icon: LucideIcon; color: string; activeClass: string }> = {
  'design-decision': {
    icon: Lightbulb,
    color: 'text-amber-500',
    activeClass: 'data-[state=on]:bg-amber-500/15 data-[state=on]:text-amber-500'
  },
  convention: {
    icon: BookOpen,
    color: 'text-blue-500',
    activeClass: 'data-[state=on]:bg-blue-500/15 data-[state=on]:text-blue-500'
  },
  gotcha: {
    icon: AlertTriangle,
    color: 'text-orange-500',
    activeClass: 'data-[state=on]:bg-orange-500/15 data-[state=on]:text-orange-500'
  },
  architecture: {
    icon: Blocks,
    color: 'text-purple-500',
    activeClass: 'data-[state=on]:bg-purple-500/15 data-[state=on]:text-purple-500'
  },
  boundary: {
    icon: Shield,
    color: 'text-red-500',
    activeClass: 'data-[state=on]:bg-red-500/15 data-[state=on]:text-red-500'
  },
  dependency: {
    icon: GitFork,
    color: 'text-emerald-500',
    activeClass: 'data-[state=on]:bg-emerald-500/15 data-[state=on]:text-emerald-500'
  }
}

const CATEGORY_SUGGESTIONS = Object.keys(CATEGORY_CONFIG)

function CategoryBadge({ category }: { category: string }) {
  const cfg = CATEGORY_CONFIG[category]
  const Icon = cfg?.icon
  return (
    <span className={`inline-flex items-center gap-1 text-xs ${cfg?.color ?? 'text-muted-foreground'}`}>
      {Icon && <Icon className="h-3 w-3" />}
      {category}
    </span>
  )
}

function AddCommentForm({
  subsystemName,
  mutations
}: {
  subsystemName: string
  mutations: ReturnType<typeof useSubsystemCommentMutations>
}) {
  const [category, setCategory] = useState('')
  const [body, setBody] = useState('')
  const [reason, setReason] = useState('')
  const [focused, setFocused] = useState(false)
  const wrapperRef = useRef<HTMLDivElement>(null)
  const textareaRef = useRef<HTMLTextAreaElement>(null)

  const canSubmit = category.trim() !== '' && body.trim() !== '' && !mutations.isPending

  const handleSubmit = () => {
    if (!canSubmit) return
    mutations.addComment.mutate(
      {
        subsystemName,
        category: category.trim(),
        body: body.trim(),
        reason: reason.trim() || undefined
      },
      {
        onSuccess: () => {
          setCategory('')
          setBody('')
          setReason('')
        }
      }
    )
  }

  const handleBlurWithin = () => {
    // Wait for focus to move, then collapse only if nothing inside remains focused.
    requestAnimationFrame(() => {
      if (!wrapperRef.current?.matches(':focus-within')) {
        setFocused(false)
      }
    })
  }

  const collapsed = !(focused || category || body.trim())

  return (
    <div
      ref={wrapperRef}
      className={`rounded-md border bg-muted/30 overflow-hidden transition-opacity ${collapsed ? 'opacity-30' : ''}`}>
      {!collapsed && (
        <div className="border-b">
          <ToggleGroup
            type="single"
            variant="outline"
            size="sm"
            value={category}
            onValueChange={val => {
              setCategory(val)
              textareaRef.current?.focus()
            }}
            className="w-full flex-wrap gap-0">
            {CATEGORY_SUGGESTIONS.map(cat => {
              const cfg = CATEGORY_CONFIG[cat]
              const Icon = cfg.icon
              return (
                <ToggleGroupItem
                  key={cat}
                  value={cat}
                  className={`h-7 px-2.5 text-xs border-0 rounded-none gap-1 ${cfg.activeClass}`}>
                  <Icon className="h-3 w-3" />
                  {cat}
                </ToggleGroupItem>
              )
            })}
          </ToggleGroup>
        </div>
      )}
      <Textarea
        ref={textareaRef}
        value={body}
        onChange={e => setBody(e.target.value)}
        onFocus={() => setFocused(true)}
        onBlur={handleBlurWithin}
        onKeyDown={e => {
          if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) {
            e.preventDefault()
            handleSubmit()
          }
        }}
        placeholder="What should agents know?"
        className={`text-sm resize-none border-0 shadow-none bg-transparent rounded-none focus-visible:ring-0 ${collapsed ? 'min-h-0 h-8 py-1.5' : 'min-h-[60px]'}`}
      />
      {!collapsed && (
        <div className="border-t bg-muted/40 px-2.5 py-1.5 flex items-center gap-2">
          <input
            type="text"
            value={reason}
            onChange={e => setReason(e.target.value)}
            onFocus={() => setFocused(true)}
            onBlur={handleBlurWithin}
            onKeyDown={e => {
              if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) {
                e.preventDefault()
                handleSubmit()
              }
            }}
            placeholder="Why? (optional)"
            className="text-xs bg-transparent border-0 outline-none placeholder:text-muted-foreground flex-1"
          />
          <span className="text-xs text-muted-foreground whitespace-nowrap">Ctrl+Enter</span>
          <Button size="sm" className="h-6 px-2 text-xs gap-1.5" disabled={!canSubmit} onClick={handleSubmit}>
            <Send className="h-3 w-3" />
            Add
          </Button>
        </div>
      )}
    </div>
  )
}

export function SubsystemCommentsSection({ subsystem }: { subsystem: SubsystemData }) {
  const comments = subsystem.comments ?? []
  const mutations = useSubsystemCommentMutations(subsystem.name, 'workspace')

  return (
    <div className="px-3 pb-1">
      <div className="flex items-center gap-1.5 mb-3">
        <MessageSquare className="h-3.5 w-3.5 text-muted-foreground" />
        <span className="text-sm font-medium text-muted-foreground">
          Knowledge{comments.length > 0 && ` (${comments.length})`}
        </span>
      </div>

      <InlineError error={mutations.error} onDismiss={mutations.resetError} className="mb-3" />

      <div className="mb-4">
        <AddCommentForm subsystemName={subsystem.name} mutations={mutations} />
      </div>

      {comments.length > 0 && (
        <div className="space-y-1">
          {comments.map(comment => (
            <div
              key={comment.id}
              className="group/comment flex gap-2.5 relative overflow-hidden rounded-md px-2 py-1.5">
              <DisciplineRadial discipline={comment.discipline} />
              <SignalAvatar discipline={comment.discipline} />
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2">
                  <span className="text-sm font-medium leading-none">{comment.discipline ?? 'You'}</span>
                  <CategoryBadge category={comment.category} />
                  {comment.created && (
                    <span className="text-xs text-muted-foreground leading-none">{formatDate(comment.created)}</span>
                  )}
                  <div className="ml-auto flex items-center gap-1">
                    <div className="opacity-0 group-hover/comment:opacity-100 transition-opacity flex gap-0.5">
                      <Button
                        variant="ghost"
                        size="sm"
                        className="h-5 w-5 p-0"
                        onClick={() =>
                          mutations.startEdit(comment.id, comment.body, comment.summary ?? '', comment.reason ?? '')
                        }>
                        <Pencil className="h-3 w-3 text-muted-foreground" />
                      </Button>
                      <Button
                        variant="ghost"
                        size="sm"
                        className="h-5 w-5 p-0"
                        onClick={() => mutations.deleteComment(comment.id)}>
                        <Trash2 className="h-3 w-3 text-muted-foreground" />
                      </Button>
                    </div>
                  </div>
                </div>
                {mutations.editingId === comment.id ? (
                  <div className="mt-1.5 space-y-2">
                    <SignalEditor
                      value={mutations.editBody}
                      onChange={mutations.setEditBody}
                      onSubmit={mutations.submitEdit}
                      onCancel={mutations.cancelEdit}
                      submitLabel="Save"
                      autoFocus
                    />
                    <Input
                      value={mutations.editSummary}
                      onChange={e => mutations.setEditSummary(e.target.value)}
                      placeholder="Summary (optional, used in prompts)"
                      className="text-sm"
                    />
                    <Input
                      value={mutations.editReason}
                      onChange={e => mutations.setEditReason(e.target.value)}
                      placeholder="Reason (optional)"
                      className="text-sm"
                    />
                  </div>
                ) : (
                  <>
                    <p className="text-sm leading-relaxed whitespace-pre-wrap mt-0.5 text-foreground/90">
                      {comment.body}
                    </p>
                    {comment.summary && (
                      <p className="text-xs text-muted-foreground/70 mt-0.5">tl;dr: {comment.summary}</p>
                    )}
                    {comment.reason && (
                      <p className="text-xs text-muted-foreground mt-1 italic">why: {comment.reason}</p>
                    )}
                  </>
                )}
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  )
}
