import { CommentEditor } from './CommentEditor'
import { PrioritySelect } from './PrioritySelect'

interface ReplyFormProps {
  value: string
  onChange: (value: string) => void
  onSubmit: () => void
  onCancel: () => void
  priority: string | null
  onPriorityChange: (value: string | null) => void
}

export function ReplyForm({ value, onChange, onSubmit, onCancel, priority, onPriorityChange }: ReplyFormProps) {
  return (
    <div className="ml-12 space-y-2">
      <PrioritySelect value={priority} onChange={onPriorityChange} />
      <CommentEditor
        value={value}
        onChange={onChange}
        onSubmit={onSubmit}
        onCancel={onCancel}
        submitLabel="Reply"
        placeholder="Write a reply..."
        autoFocus
      />
    </div>
  )
}
