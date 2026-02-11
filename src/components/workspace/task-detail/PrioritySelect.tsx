import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'

interface PrioritySelectProps {
  value: string | null
  onChange: (value: string | null) => void
}

export function PrioritySelect({ value, onChange }: PrioritySelectProps) {
  return (
    <div className="flex items-center gap-2">
      <label className="text-xs font-medium text-muted-foreground">Priority</label>
      <Select value={value || 'none'} onValueChange={onChange}>
        <SelectTrigger className="h-6 w-28 text-xs">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="none">None</SelectItem>
          <SelectItem value="low">Low</SelectItem>
          <SelectItem value="medium">Medium</SelectItem>
          <SelectItem value="high">High</SelectItem>
          <SelectItem value="critical">Critical</SelectItem>
        </SelectContent>
      </Select>
    </div>
  )
}
