import { CommentAvatar, DisciplineRadial, NumberedIdDisplay } from '@/components/shared'
import { formatDate } from '@/lib/formatDate'
import type { TaskComment } from '@/types/generated'

interface ReplyCardProps {
  reply: TaskComment
}

export function ReplyCard({ reply }: ReplyCardProps) {
  return (
    <div className="ml-12 group/reply flex gap-2.5 relative overflow-hidden rounded-md px-2 py-1.5 pb-6 border-l-2 border-border/50">
      <DisciplineRadial discipline={reply.author} />
      <CommentAvatar discipline={reply.author} />
      <div className="flex-1 min-w-0 space-y-2 pr-24">
        <div className="flex items-baseline gap-2">
          <NumberedIdDisplay id={reply.id} variant="inline" />
          <span className="text-sm font-medium">{reply.author ?? 'You'}</span>
          {reply.priority && reply.priority !== 'none' && (
            <span className="text-xs text-muted-foreground uppercase">{reply.priority}</span>
          )}
        </div>
        <p className="text-sm leading-relaxed whitespace-pre-wrap text-foreground/90">{reply.body}</p>
      </div>
      {reply.created && (
        <div className="absolute bottom-1.5 right-2">
          <span className="text-xs text-muted-foreground">{formatDate(reply.created)}</span>
        </div>
      )}
    </div>
  )
}
