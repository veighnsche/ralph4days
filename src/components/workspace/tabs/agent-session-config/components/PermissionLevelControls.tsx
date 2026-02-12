import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import { isPermissionLevel, PERMISSION_LEVEL_OPTIONS } from '../constants'
import { useAgentSessionConfigActions, useAgentSessionConfigLaunchState } from '../hooks/useAgentSessionConfigTabState'

export function PermissionLevelControls() {
  const { permissionLevel } = useAgentSessionConfigLaunchState()
  const { setPermissionLevel } = useAgentSessionConfigActions()

  return (
    <ToggleGroup
      type="single"
      value={permissionLevel}
      onValueChange={value => {
        if (value === '' || !isPermissionLevel(value)) return
        setPermissionLevel(value)
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
