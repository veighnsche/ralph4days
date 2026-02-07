import { Bot, MessageSquare, Pencil, Trash2, User } from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";
import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import { Button } from "@/components/ui/button";
import { useInvokeMutation } from "@/hooks/useInvokeMutation";
import { formatDate } from "@/lib/formatDate";
import type { Task } from "@/types/prd";
import { CommentEditor } from "./CommentEditor";

const INVALIDATE_KEYS = [["get_tasks"]];

export function CommentsSection({ task }: { task: Task }) {
  const comments = task.comments ?? [];

  const [commentInput, setCommentInput] = useState("");
  const [editingId, setEditingId] = useState<number | null>(null);
  const [editBody, setEditBody] = useState("");

  const addComment = useInvokeMutation<{ taskId: number; author: string; body: string }>("add_task_comment", {
    invalidateKeys: INVALIDATE_KEYS,
    onSuccess: () => {
      setCommentInput("");
      toast.success("Comment added");
    },
    onError: (err) => toast.error(err.message),
  });

  const editComment = useInvokeMutation<{ taskId: number; commentId: number; body: string }>("update_task_comment", {
    invalidateKeys: INVALIDATE_KEYS,
    onSuccess: () => {
      setEditingId(null);
      setEditBody("");
    },
    onError: (err) => toast.error(err.message),
  });

  const deleteComment = useInvokeMutation<{ taskId: number; commentId: number }>("delete_task_comment", {
    invalidateKeys: INVALIDATE_KEYS,
    onSuccess: () => toast.success("Comment deleted"),
    onError: (err) => toast.error(err.message),
  });

  const handleAddComment = () => {
    if (!commentInput.trim()) return;
    addComment.mutate({ taskId: task.id, author: "human", body: commentInput.trim() });
  };

  const startEdit = (commentId: number, body: string) => {
    setEditingId(commentId);
    setEditBody(body);
  };

  const cancelEdit = () => {
    setEditingId(null);
    setEditBody("");
  };

  const submitEdit = () => {
    if (editingId === null || !editBody.trim()) return;
    editComment.mutate({ taskId: task.id, commentId: editingId, body: editBody.trim() });
  };

  const isPending = addComment.isPending || editComment.isPending || deleteComment.isPending;

  return (
    <div className="px-3 pb-1">
      {/* Header */}
      <div className="flex items-center gap-1.5 mb-3">
        <MessageSquare className="h-3.5 w-3.5 text-muted-foreground" />
        <span className="text-sm font-medium text-muted-foreground">
          Comments{comments.length > 0 && ` (${comments.length})`}
        </span>
      </div>

      {/* Comment list */}
      {comments.length > 0 && (
        <div className="space-y-3 mb-4">
          {comments.map((comment) => (
            <div key={comment.id} className="group/comment flex gap-2.5">
              <Avatar size="sm" className="mt-0.5 flex-shrink-0">
                <AvatarFallback className="text-muted-foreground">
                  {comment.author === "agent" ? <Bot className="h-3 w-3" /> : <User className="h-3 w-3" />}
                </AvatarFallback>
              </Avatar>
              <div className="flex-1 min-w-0">
                <div className="flex items-baseline gap-2">
                  <span className="text-sm font-medium">
                    {comment.author === "agent" ? `Agent #${comment.agent_task_id}` : "You"}
                  </span>
                  {comment.created && (
                    <span className="text-xs text-muted-foreground">{formatDate(comment.created)}</span>
                  )}
                  <div className="ml-auto opacity-0 group-hover/comment:opacity-100 transition-opacity flex gap-0.5">
                    <Button
                      variant="ghost"
                      size="sm"
                      className="h-5 w-5 p-0"
                      onClick={() => startEdit(comment.id, comment.body)}
                    >
                      <Pencil className="h-3 w-3 text-muted-foreground" />
                    </Button>
                    <Button
                      variant="ghost"
                      size="sm"
                      className="h-5 w-5 p-0"
                      onClick={() => deleteComment.mutate({ taskId: task.id, commentId: comment.id })}
                    >
                      <Trash2 className="h-3 w-3 text-muted-foreground" />
                    </Button>
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

      {/* Input */}
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
  );
}
