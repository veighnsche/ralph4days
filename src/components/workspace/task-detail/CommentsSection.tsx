import { MessageSquare, Pencil, Trash2 } from 'lucide-react'
import { useState } from 'react'
import { CommentAvatar, DisciplineRadial, InlineError, PriorityIcon, PriorityRadial } from '@/components/shared'
import { Button } from '@/components/ui/button'
import { useCommentMutations } from '@/hooks/tasks'
import { formatDate } from '@/lib/formatDate'
import type { Task } from '@/types/generated'
import { CommentEditor } from './CommentEditor'

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
    addComment.mutate({ taskId: task.id, body: commentInput.trim() }, { onSuccess: () => setCommentInput('') })
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

      <div className="mb-4">
        <CommentEditor
          value={commentInput}
          onChange={setCommentInput}
          onSubmit={handleAddComment}
          submitLabel="Comment"
          placeholder="Add a comment..."
          disabled={isPending}
        />
      </div>

      {comments.length > 0 && (
        <div className="space-y-1">
          {comments.map(comment => (
            <div
              key={comment.id}
              className="group/comment flex gap-2.5 relative overflow-hidden rounded-md px-2 py-1.5">
              {comment.priority && <PriorityRadial priority={comment.priority} />}
              <DisciplineRadial discipline={comment.discipline} />
              <CommentAvatar discipline={comment.discipline} />
              <div className="flex-1 min-w-0">
                <div className="flex items-baseline gap-2">
                  <span className="text-sm font-medium">{comment.discipline ?? 'You'}</span>
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
    </div>
  )
}
