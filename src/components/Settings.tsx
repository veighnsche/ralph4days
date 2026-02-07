import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Field, FieldDescription, FieldLabel } from '@/components/ui/field'
import { Switch } from '@/components/ui/switch'
import { useTheme } from '@/lib/theme-provider'

interface SettingsProps {
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function Settings({ open, onOpenChange }: SettingsProps) {
  const { theme, setTheme } = useTheme()
  const isDark = theme === 'dark'

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Settings</DialogTitle>
          <DialogDescription>Customize your Ralph4days experience</DialogDescription>
        </DialogHeader>

        <div className="space-y-4">
          <Field orientation="horizontal">
            <div className="flex-1">
              <FieldLabel>Dark Mode</FieldLabel>
              <FieldDescription>Use dark theme for the interface</FieldDescription>
            </div>
            <Switch checked={isDark} onCheckedChange={checked => setTheme(checked ? 'dark' : 'light')} />
          </Field>
        </div>
      </DialogContent>
    </Dialog>
  )
}
