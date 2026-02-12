import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import type { PermissionLevel } from '@/hooks/preferences'
import { PERMISSION_LEVEL_OPTIONS } from '../constants'
import { useAgentSessionConfigStore } from '../store'

export function PermissionLevelControls() {
  const permissionLevel = useAgentSessionConfigStore(state => state.permissionLevel)
  const setPermissionLevel = useAgentSessionConfigStore(state => state.setPermissionLevel)

  return (
    <ToggleGroup
      type="single"
      value={permissionLevel}
      onValueChange={value => {
        if (value === '') return
        setPermissionLevel(value as PermissionLevel)
      }}
      variant="outline"
      aria-label="Permission Level">
      {PERMISSION_LEVEL_OPTIONS.map(option => (
        <ToggleGroupItem key={option.value} value={option.value}>
          {option.label}
        </ToggleGroupItem>
      ))}
    </ToggleGroup>
  )
}
