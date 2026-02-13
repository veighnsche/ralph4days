import type { LucideIcon } from 'lucide-react'
import {
  BookOpen,
  Cable,
  Cloud,
  Code,
  Cpu,
  Database,
  FileText,
  FlaskConical,
  Layers,
  Monitor,
  Package,
  Palette,
  Rocket,
  Server,
  Settings,
  Shield,
  Target,
  TestTube,
  Wrench
} from 'lucide-react'
import { useFormContext } from 'react-hook-form'
import { Badge } from '@/components/ui/badge'
import { FormControl, FormField, FormItem, FormLabel, FormMessage } from '@/components/ui/form'
import { Input } from '@/components/ui/input'
import { NativeSelect } from '@/components/ui/native-select'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Textarea } from '@/components/ui/textarea'
import type { DisciplineFormData } from '@/lib/schemas'

type DisciplineIconName =
  | 'Code'
  | 'Palette'
  | 'Database'
  | 'Shield'
  | 'TestTube'
  | 'BookOpen'
  | 'Wrench'
  | 'Rocket'
  | 'Monitor'
  | 'Cloud'
  | 'Cpu'
  | 'Settings'
  | 'FileText'
  | 'Layers'
  | 'Package'
  | 'Target'
  | 'Server'
  | 'Cable'
  | 'FlaskConical'

const COMMON_ICONS: readonly DisciplineIconName[] = [
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
  'Target',
  'Server',
  'Cable',
  'FlaskConical'
] as const

const DISCIPLINE_ICONS: Record<DisciplineIconName, LucideIcon> = {
  Code,
  Palette,
  Database,
  Shield,
  TestTube,
  BookOpen,
  Wrench,
  Rocket,
  Monitor,
  Cloud,
  Cpu,
  Settings,
  FileText,
  Layers,
  Package,
  Target,
  Server,
  Cable,
  FlaskConical
}

const COMMON_COLORS = [
  { name: 'Blue', value: '#3b82f6' },
  { name: 'Green', value: '#22c55e' },
  { name: 'Emerald', value: '#10b981' },
  { name: 'Yellow', value: '#eab308' },
  { name: 'Orange', value: '#f97316' },
  { name: 'Red', value: '#ef4444' },
  { name: 'Purple', value: '#a855f7' },
  { name: 'Violet', value: '#8b5cf6' },
  { name: 'Pink', value: '#ec4899' },
  { name: 'Teal', value: '#14b8a6' },
  { name: 'Cyan', value: '#06b6d4' },
  { name: 'Indigo', value: '#6366f1' }
] as const

export function DisciplineFormFields({ disabled, isEditing }: { disabled?: boolean; isEditing?: boolean }) {
  const { control, watch } = useFormContext<DisciplineFormData>()
  const iconName = watch('icon')
  const colorValue = watch('color')
  const selectedAgent = watch('agent')
  const skills = watch('skills')
  const mcpServers = watch('mcpServers')

  const IconComponent = DISCIPLINE_ICONS[(iconName as DisciplineIconName) ?? 'Code'] ?? null

  return (
    <Tabs defaultValue="basic" className="w-full">
      <TabsList className="w-full grid grid-cols-6">
        <TabsTrigger value="basic">Basic</TabsTrigger>
        <TabsTrigger value="prompt">System Prompt</TabsTrigger>
        <TabsTrigger value="launch">Launch</TabsTrigger>
        <TabsTrigger value="skills">
          Skills
          {skills && skills.length > 0 && (
            <Badge variant="secondary" className="ml-1">
              {skills.length}
            </Badge>
          )}
        </TabsTrigger>
        <TabsTrigger value="conventions">Conventions</TabsTrigger>
        <TabsTrigger value="mcp">
          MCP Servers
          {mcpServers && mcpServers.length > 0 && (
            <Badge variant="secondary" className="ml-1">
              {mcpServers.length}
            </Badge>
          )}
        </TabsTrigger>
      </TabsList>

      <TabsContent value="basic" className="space-y-3 mt-4">
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
              <p className="text-xs text-muted-foreground">
                3-4 uppercase letters for task IDs (e.g., FRNT, BACK, TEST)
              </p>
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
      </TabsContent>

      <TabsContent value="prompt" className="mt-4">
        <FormField
          control={control}
          name="systemPrompt"
          render={({ field }) => (
            <FormItem>
              <FormLabel>System Prompt</FormLabel>
              <FormControl>
                <Textarea
                  {...field}
                  value={field.value || ''}
                  placeholder="Enter the system prompt for Claude Code when using this discipline...\n\nExample:\nYou are a frontend specialist focused on React and TypeScript.\n\n## Your Expertise\n- React 19 with hooks\n- TypeScript strict mode\n- Component composition\n\n## Your Approach\n1. Start with user needs\n2. Build incrementally\n3. Test as you go"
                  className="min-h-[400px] font-mono text-xs"
                  disabled={disabled}
                />
              </FormControl>
              <p className="text-xs text-muted-foreground">
                The system prompt that Claude Code receives when executing tasks in this discipline
              </p>
              <FormMessage />
            </FormItem>
          )}
        />
      </TabsContent>

      <TabsContent value="launch" className="space-y-3 mt-4">
        <FormField
          control={control}
          name="agent"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Agent</FormLabel>
              <FormControl>
                <NativeSelect
                  value={field.value || ''}
                  onChange={e => field.onChange(e.target.value || undefined)}
                  disabled={disabled}>
                  <option value="">Global default</option>
                  <option value="claude">Claude</option>
                  <option value="codex">Codex</option>
                </NativeSelect>
              </FormControl>
              <p className="text-xs text-muted-foreground">Optional default agent for tasks in this discipline</p>
              <FormMessage />
            </FormItem>
          )}
        />

        <FormField
          control={control}
          name="model"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Model</FormLabel>
              <FormControl>
                <Input
                  {...field}
                  value={field.value || ''}
                  onChange={e => field.onChange(e.target.value)}
                  placeholder={
                    selectedAgent === 'codex'
                      ? 'gpt-5-codex (or another codex model)'
                      : 'claude-sonnet-4 (or another claude model)'
                  }
                  disabled={disabled}
                />
              </FormControl>
              <p className="text-xs text-muted-foreground">Optional default model override for this discipline</p>
              <FormMessage />
            </FormItem>
          )}
        />

        <FormField
          control={control}
          name="effort"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Effort</FormLabel>
              <FormControl>
                <NativeSelect
                  value={field.value || ''}
                  onChange={e => field.onChange(e.target.value || undefined)}
                  disabled={disabled}>
                  <option value="">Model default</option>
                  <option value="low">Low</option>
                  <option value="medium">Medium</option>
                  <option value="high">High</option>
                </NativeSelect>
              </FormControl>
              <p className="text-xs text-muted-foreground">Optional default effort for selected model</p>
              <FormMessage />
            </FormItem>
          )}
        />

        <FormField
          control={control}
          name="thinking"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Thinking</FormLabel>
              <FormControl>
                <NativeSelect
                  value={field.value === undefined ? '' : field.value ? 'true' : 'false'}
                  onChange={e => {
                    if (!e.target.value) {
                      field.onChange(undefined)
                      return
                    }
                    field.onChange(e.target.value === 'true')
                  }}
                  disabled={disabled}>
                  <option value="">Global default</option>
                  <option value="true">On</option>
                  <option value="false">Off</option>
                </NativeSelect>
              </FormControl>
              <p className="text-xs text-muted-foreground">Optional default thinking mode for this discipline</p>
              <FormMessage />
            </FormItem>
          )}
        />
      </TabsContent>

      <TabsContent value="skills" className="mt-4">
        <FormField
          control={control}
          name="skills"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Skills (one per line)</FormLabel>
              <FormControl>
                <Textarea
                  value={field.value?.join('\n') || ''}
                  onChange={e =>
                    field.onChange(
                      e.target.value
                        .split('\n')
                        .map(s => s.trim())
                        .filter(Boolean)
                    )
                  }
                  placeholder="React 19\nTypeScript\nTailwind CSS v4\nComponent Composition\nState Management\nAccessibility\nPerformance Optimization"
                  className="min-h-[300px] font-mono text-xs"
                  disabled={disabled}
                />
              </FormControl>
              <p className="text-xs text-muted-foreground">
                Specific capabilities and technologies for this discipline
              </p>
              <FormMessage />
            </FormItem>
          )}
        />
      </TabsContent>

      <TabsContent value="conventions" className="mt-4">
        <FormField
          control={control}
          name="conventions"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Conventions</FormLabel>
              <FormControl>
                <Textarea
                  {...field}
                  value={field.value || ''}
                  placeholder={`Enter coding conventions for this discipline...\n\n## Naming\n- Components: PascalCase (UserProfile.tsx)\n- Hooks: camelCase with 'use' prefix (useUserData.ts)\n- Files: kebab-case (user-profile.tsx)\n\n## Structure\n- Colocate tests with components\n- Group by feature, not type\n\n## Patterns\n- Always use shadcn/ui components\n- Prefer composition over props\n\n## Quality Standards\n- All components must be accessible\n- Use CSS variables for theming`}
                  className="min-h-[400px] font-mono text-xs"
                  disabled={disabled}
                />
              </FormControl>
              <p className="text-xs text-muted-foreground">
                Naming conventions, code patterns, and quality standards for this discipline
              </p>
              <FormMessage />
            </FormItem>
          )}
        />
      </TabsContent>

      <TabsContent value="mcp" className="mt-4">
        <FormField
          control={control}
          name="mcpServers"
          render={({ field }) => (
            <FormItem>
              <FormLabel>MCP Servers (JSON array)</FormLabel>
              <FormControl>
                <Textarea
                  value={field.value ? JSON.stringify(field.value, null, 2) : '[]'}
                  onChange={e => {
                    try {
                      const parsed = JSON.parse(e.target.value)
                      field.onChange(parsed)
                    } catch {
                      // Invalid JSON, don't update
                    }
                  }}
                  placeholder={`[\n  {\n    "name": "shadcn-ui",\n    "command": "npx",\n    "args": ["-y", "@modelcontextprotocol/server-shadcn"],\n    "env": {}\n  }\n]`}
                  className="min-h-[300px] font-mono text-xs"
                  disabled={disabled}
                />
              </FormControl>
              <p className="text-xs text-muted-foreground">
                Model Context Protocol servers that provide additional tools/resources (must be valid JSON)
              </p>
              <FormMessage />
            </FormItem>
          )}
        />
      </TabsContent>
    </Tabs>
  )
}
