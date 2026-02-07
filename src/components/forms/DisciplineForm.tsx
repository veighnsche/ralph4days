import * as LucideIcons from 'lucide-react'
import { useFormContext } from 'react-hook-form'
import { FormControl, FormField, FormItem, FormLabel, FormMessage } from '@/components/ui/form'
import { Input } from '@/components/ui/input'
import { NativeSelect } from '@/components/ui/native-select'
import type { DisciplineFormData } from '@/lib/schemas'

const COMMON_ICONS = [
  'Code',
  'Palette',
  'Database',
  'Shield',
  'TestTube',
  'BookOpen',
  'Wrench',
  'Rocket',
  'Monitor',
  'Cloud',
  'Cpu',
  'Settings',
  'FileText',
  'Layers',
  'Package',
  'Target'
] as const

const COMMON_COLORS = [
  { name: 'Blue', value: '#3b82f6' },
  { name: 'Green', value: '#22c55e' },
  { name: 'Yellow', value: '#eab308' },
  { name: 'Red', value: '#ef4444' },
  { name: 'Purple', value: '#a855f7' },
  { name: 'Pink', value: '#ec4899' },
  { name: 'Orange', value: '#f97316' },
  { name: 'Teal', value: '#14b8a6' },
  { name: 'Indigo', value: '#6366f1' },
  { name: 'Cyan', value: '#06b6d4' }
] as const

export function DisciplineFormFields({ disabled, isEditing }: { disabled?: boolean; isEditing?: boolean }) {
  const { control, watch } = useFormContext<DisciplineFormData>()
  const iconName = watch('icon')
  const colorValue = watch('color')

  const IconComponent = LucideIcons[iconName as keyof typeof LucideIcons] as LucideIcons.LucideIcon

  return (
    <div className="space-y-3">
      <FormField
        control={control}
        name="displayName"
        render={({ field }) => (
          <FormItem>
            <FormLabel>
              Display Name <span className="text-destructive">*</span>
            </FormLabel>
            <FormControl>
              <Input {...field} placeholder="Enter discipline display name" required disabled={disabled} />
            </FormControl>
            <p className="text-xs text-muted-foreground">The human-readable name shown in the UI</p>
            <FormMessage />
          </FormItem>
        )}
      />

      <FormField
        control={control}
        name="acronym"
        render={({ field }) => (
          <FormItem>
            <FormLabel>
              Acronym <span className="text-destructive">*</span>
            </FormLabel>
            <FormControl>
              <Input
                {...field}
                onChange={e => field.onChange(e.target.value.toUpperCase())}
                placeholder="FRNT (3-4 uppercase letters)"
                maxLength={4}
                required
                className="font-mono"
                disabled={disabled}
              />
            </FormControl>
            <p className="text-xs text-muted-foreground">3-4 uppercase letters for task IDs (e.g., FRNT, BACK, TEST)</p>
            <FormMessage />
          </FormItem>
        )}
      />

      <FormField
        control={control}
        name="name"
        render={({ field }) => (
          <FormItem>
            <FormLabel>Internal Name</FormLabel>
            <FormControl>
              <Input {...field} placeholder="auto-generated-from-display-name" disabled={disabled || isEditing} />
            </FormControl>
            <p className="text-xs text-muted-foreground">
              {isEditing
                ? 'Internal name cannot be changed after creation'
                : 'Auto-generated from display name (lowercase with hyphens)'}
            </p>
            <FormMessage />
          </FormItem>
        )}
      />

      <FormField
        control={control}
        name="icon"
        render={({ field }) => (
          <FormItem>
            <FormLabel>
              Icon <span className="text-destructive">*</span>
            </FormLabel>
            <div className="flex gap-2 items-center">
              <FormControl>
                <NativeSelect {...field} required className="flex-1" disabled={disabled}>
                  {COMMON_ICONS.map(icon => (
                    <option key={icon} value={icon}>
                      {icon}
                    </option>
                  ))}
                </NativeSelect>
              </FormControl>
              {IconComponent && (
                <div
                  className="p-2 rounded-md shrink-0"
                  style={{
                    backgroundColor: `color-mix(in oklch, ${colorValue} 15%, transparent)`,
                    color: colorValue
                  }}>
                  <IconComponent className="h-5 w-5" />
                </div>
              )}
            </div>
            <FormMessage />
          </FormItem>
        )}
      />

      <FormField
        control={control}
        name="color"
        render={({ field }) => (
          <FormItem>
            <FormLabel>
              Color <span className="text-destructive">*</span>
            </FormLabel>
            <div className="flex gap-2 items-center">
              <FormControl>
                <NativeSelect {...field} required className="flex-1" disabled={disabled}>
                  {COMMON_COLORS.map(color => (
                    <option key={color.value} value={color.value}>
                      {color.name}
                    </option>
                  ))}
                </NativeSelect>
              </FormControl>
              <div
                className="w-8 h-8 rounded-md border shrink-0"
                style={{ backgroundColor: field.value }}
                title={field.value}
              />
            </div>
            <FormMessage />
          </FormItem>
        )}
      />
    </div>
  )
}
