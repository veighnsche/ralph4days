import { FieldLabel } from '@/components/ui/field'

export function PickerSectionHeader({ title, action }: { title: string; action?: React.ReactNode }) {
  return (
    <div className="flex min-h-9 items-center justify-between gap-2">
      <FieldLabel>{title}</FieldLabel>
      <div className="flex h-9 w-9 shrink-0 items-center justify-center">{action}</div>
    </div>
  )
}
