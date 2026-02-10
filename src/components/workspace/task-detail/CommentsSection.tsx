import { Bot, MessageSquare, Pencil, Trash2, User } from 'lucide-react'
import { useState } from 'react'
import { InlineError, PriorityIcon, PriorityRadial } from '@/components/shared'
import { Avatar, AvatarFallback } from '@/components/ui/avatar'
import { Button } from '@/components/ui/button'
import { CroppedImage } from '@/components/ui/cropped-image'
import { useDisciplines } from '@/hooks/disciplines'
import { useCommentMutations } from '@/hooks/tasks'
import { formatDate } from '@/lib/formatDate'
import type { Task, TaskComment as TaskCommentType } from '@/types/generated'
import { CommentEditor } from './CommentEditor'

function CommentAvatar({ comment }: { comment: TaskCommentType }) {
  const { disciplines } = useDisciplines()

  if (comment.discipline) {
    const disc = disciplines.find(d => d.name === comment.discipline)
    if (disc?.crops?.face) {
      return (
        <div className="size-12 flex-shrink-0 rounded-md overflow-hidden self-start">
          <CroppedImage
            disciplineName={disc.name}
            label="comment-face"
            crop={disc.crops.face}
            className="size-full object-cover"
          />
        </div>
      )
    }
  }

  return (
    <Avatar size="sm" className="mt-0.5 flex-shrink-0">
      <AvatarFallback className="text-muted-foreground">
        {comment.author === 'human' ? <User className="h-3 w-3" /> : <Bot className="h-3 w-3" />}
      </AvatarFallback>
    </Avatar>
  )
}

export function CommentsSection({ task }: { task: Task }) {
  const comments = task.comments ?? []
  const [commentInput, setCommentInput] = useState('')

  const {
    addComment,
    startEdit,
    cancelEdit,
    submitEdit,
    deleteComment,
    editingId,
    editBody,
    setEditBody,
    isPending,
    error,
    resetError
  } = useCommentMutations(task.id)

  const handleAddComment = () => {
    if (!commentInput.trim()) return
    addComment.mutate(
      { taskId: task.id, author: 'human', body: commentInput.trim() },
      { onSuccess: () => setCommentInput('') }
    )
  }

  return (
    <div className="px-3 pb-1">
      <div className="flex items-center gap-1.5 mb-3">
        <MessageSquare className="h-3.5 w-3.5 text-muted-foreground" />
        <span className="text-sm font-medium text-muted-foreground">
          Comments{comments.length > 0 && ` (${comments.length})`}
        </span>
      </div>

      <InlineError error={error} onDismiss={resetError} className="mb-3" />

      {comments.length > 0 && (
        <div className="space-y-3 mb-4">
          {comments.map(comment => (
            <div
              key={comment.id}
              className="group/comment flex gap-2.5 relative overflow-hidden rounded-md px-2 py-1.5">
              {comment.priority && <PriorityRadial priority={comment.priority} />}
              <CommentAvatar comment={comment} />
              <div className="flex-1 min-w-0">
                <div className="flex items-baseline gap-2">
                  <span className="text-sm font-medium">
                    {comment.author === 'human'
                      ? 'You'
                      : comment.author === 'agent'
                        ? `Agent #${comment.agent_task_id}`
                        : comment.author.charAt(0).toUpperCase() + comment.author.slice(1)}
                  </span>
                  {comment.created && (
                    <span className="text-xs text-muted-foreground">{formatDate(comment.created)}</span>
                  )}
                  <div className="ml-auto flex items-center gap-1">
                    <div className="opacity-0 group-hover/comment:opacity-100 transition-opacity flex gap-0.5">
                      <Button
                        variant="ghost"
                        size="sm"
                        className="h-5 w-5 p-0"
                        onClick={() => startEdit(comment.id, comment.body)}>
                        <Pencil className="h-3 w-3 text-muted-foreground" />
                      </Button>
                      <Button
                        variant="ghost"
                        size="sm"
                        className="h-5 w-5 p-0"
                        onClick={() => deleteComment(comment.id)}>
                        <Trash2 className="h-3 w-3 text-muted-foreground" />
                      </Button>
                    </div>
                    {comment.priority && <PriorityIcon priority={comment.priority} size="md" />}
                  </div>
                </div>
                {editingId === comment.id ? (
                  <div className="mt-1.5">
                    <CommentEditor
                      value={editBody}
                      onChange={setEditBody}
                      onSubmit={submitEdit}
                      onCancel={cancelEdit}
                      submitLabel="Save"
                      autoFocus
                    />
                  </div>
                ) : (
                  <p className="text-sm leading-relaxed whitespace-pre-wrap mt-0.5 text-foreground/90">
                    {comment.body}
                  </p>
                )}
              </div>
            </div>
          ))}
        </div>
      )}

      <div className="flex gap-2.5 items-start">
        <Avatar size="sm" className="mt-1.5 flex-shrink-0">
          <AvatarFallback className="text-muted-foreground">
            <User className="h-3 w-3" />
          </AvatarFallback>
        </Avatar>
        <div className="flex-1 min-w-0">
          <CommentEditor
            value={commentInput}
            onChange={setCommentInput}
            onSubmit={handleAddComment}
            submitLabel="Comment"
            placeholder="Add a comment..."
            disabled={isPending}
          />
        </div>
      </div>
    </div>
  )
}
