import { invoke } from '@tauri-apps/api/core'
import { Layers } from 'lucide-react'
import { useEffect, useState } from 'react'
import { DisciplineLabel } from '@/components/prd/DisciplineLabel'
import { Badge } from '@/components/ui/badge'
import { Card } from '@/components/ui/card'
import { CroppedImage } from '@/components/ui/cropped-image'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Separator } from '@/components/ui/separator'
import { Skeleton } from '@/components/ui/skeleton'
import { useInvoke } from '@/hooks/api'
import { useStackMetadata } from '@/hooks/disciplines'
import { useTabMeta } from '@/hooks/workspace'
import { resolveIcon } from '@/lib/iconRegistry'
import type { WorkspaceTab } from '@/stores/useWorkspaceStore'
import type { DisciplineConfig } from '@/types/generated'
import { PropertyRow } from './PropertyRow'

function useDisciplineImage(name: string, hasImage: boolean) {
  const [imageUrl, setImageUrl] = useState<string | null>(null)

  useEffect(() => {
    if (!hasImage) return
    let cancelled = false
    invoke<string | null>('get_discipline_image_data', { name }).then(b64 => {
      if (!cancelled && b64) setImageUrl(`data:image/png;base64,${b64}`)
    })
    return () => {
      cancelled = true
    }
  }, [name, hasImage])

  return imageUrl
}

function DisciplineContent({ discipline, imageUrl }: { discipline: DisciplineConfig; imageUrl: string | null }) {
  const Icon = resolveIcon(discipline.icon)
  const sections: React.ReactNode[] = []

  sections.push(
    <div key="header" className="px-6 flex items-start gap-3">
      {imageUrl && discipline.crops?.face ? (
        <CroppedImage src={imageUrl} crop={discipline.crops.face} className="h-12 w-12 rounded-md shrink-0" />
      ) : (
        <div
          className="h-12 w-12 rounded-md shrink-0 flex items-center justify-center"
          style={{
            backgroundColor: `color-mix(in oklch, ${discipline.color} 15%, transparent)`,
            color: discipline.color
          }}>
          <Icon className="h-6 w-6" />
        </div>
      )}
      <div className="space-y-1.5">
        <DisciplineLabel acronym={discipline.acronym} color={discipline.color} />
        <h1 className="text-xl font-semibold leading-tight">{discipline.displayName}</h1>
      </div>
    </div>
  )

  if (discipline.systemPrompt) {
    sections.push(
      <div key="system-prompt" className="px-6 space-y-2">
        <h2 className="text-sm font-medium text-muted-foreground">System Prompt</h2>
        <pre className="text-xs whitespace-pre-wrap font-mono bg-muted p-3 rounded-md">{discipline.systemPrompt}</pre>
      </div>
    )
  }

  if (discipline.skills && discipline.skills.length > 0) {
    sections.push(
      <div key="skills" className="px-6 space-y-2">
        <h2 className="text-sm font-medium text-muted-foreground">Skills</h2>
        <div className="flex flex-wrap gap-1.5">
          {discipline.skills.map(skill => (
            <Badge key={skill} variant="secondary" className="text-xs">
              {skill}
            </Badge>
          ))}
        </div>
      </div>
    )
  }

  if (discipline.conventions) {
    sections.push(
      <div key="conventions" className="px-6 space-y-2">
        <h2 className="text-sm font-medium text-muted-foreground">Conventions</h2>
        <pre className="text-xs whitespace-pre-wrap font-mono bg-muted p-3 rounded-md">{discipline.conventions}</pre>
      </div>
    )
  }

  if (discipline.mcpServers && discipline.mcpServers.length > 0) {
    sections.push(
      <div key="mcp-servers" className="px-6 space-y-2">
        <h2 className="text-sm font-medium text-muted-foreground">MCP Servers</h2>
        <div className="space-y-3">
          {discipline.mcpServers.map((server, index) => (
            <div key={server.name}>
              <div className="font-medium text-sm mb-2">{server.name}</div>
              <div className="space-y-1.5">
                <div className="text-xs">
                  <span className="text-muted-foreground">Command:</span>{' '}
                  <code className="font-mono bg-muted px-1.5 py-0.5 rounded">{server.command}</code>
                </div>
                {server.args && server.args.length > 0 && (
                  <div className="text-xs">
                    <span className="text-muted-foreground">Args:</span>{' '}
                    <code className="font-mono bg-muted px-1.5 py-0.5 rounded">{server.args.join(' ')}</code>
                  </div>
                )}
                {server.env && Object.keys(server.env).length > 0 && (
                  <div className="text-xs">
                    <span className="text-muted-foreground">Env:</span>{' '}
                    <code className="font-mono bg-muted px-1.5 py-0.5 rounded text-xs">
                      {JSON.stringify(server.env)}
                    </code>
                  </div>
                )}
              </div>
              {index < discipline.mcpServers.length - 1 && <Separator className="mt-3" />}
            </div>
          ))}
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {sections.reduce<React.ReactNode[]>((acc, section, i) => {
        if (i > 0) acc.push(<Separator key={`sep-${(section as React.ReactElement).key}`} />)
        acc.push(section)
        return acc
      }, [])}
    </div>
  )
}

function DisciplineSidebar({ discipline, stackName }: { discipline: DisciplineConfig; stackName?: string }) {
  const Icon = resolveIcon(discipline.icon)

  return (
    <div className="px-4 py-4 space-y-0.5 overflow-y-auto h-full">
      <PropertyRow label="Color">
        <div className="flex items-center gap-2">
          <div className="w-4 h-4 rounded border" style={{ backgroundColor: discipline.color }} />
          <span className="text-xs font-mono">{discipline.color}</span>
        </div>
      </PropertyRow>

      <Separator bleed="md" />

      <PropertyRow label="Icon">
        <div className="flex items-center gap-1.5">
          <Icon className="h-3.5 w-3.5" style={{ color: discipline.color }} />
          <span className="text-xs">{discipline.icon}</span>
        </div>
      </PropertyRow>

      {stackName && (
        <>
          <Separator bleed="md" />
          <PropertyRow label="Stack">
            <span className="text-sm">{stackName}</span>
          </PropertyRow>
        </>
      )}
    </div>
  )
}

export function DisciplineDetailTabContent({ tab }: { tab: WorkspaceTab }) {
  const disciplineName = tab.data?.entityId as string

  const { data: disciplines, isLoading } = useInvoke<DisciplineConfig[]>('get_disciplines_config', undefined, {
    staleTime: 5 * 60 * 1000
  })
  const { stacks } = useStackMetadata()

  const discipline = disciplines?.find(d => d.name === disciplineName)
  const imageUrl = useDisciplineImage(disciplineName, !!discipline?.imagePath)
  const stackName = discipline?.stackId != null ? stacks.find(s => s.stackId === discipline.stackId)?.name : undefined

  useTabMeta(tab.id, discipline?.displayName ?? 'Discipline', Layers)

  if (isLoading) {
    return (
      <div className="h-full flex flex-col">
        <div className="flex-1 p-6">
          <Skeleton className="h-[200px]" />
        </div>
      </div>
    )
  }

  if (!discipline) {
    return (
      <div className="h-full flex items-center justify-center">
        <div className="text-center">
          <Layers className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
          <h2 className="text-lg font-semibold mb-2">Discipline not found</h2>
          <p className="text-sm text-muted-foreground">The requested discipline could not be loaded</p>
        </div>
      </div>
    )
  }

  return (
    <div
      className="h-full px-3 relative"
      style={{
        background: `repeating-linear-gradient(
        45deg,
        transparent,
        transparent 10px,
        ${discipline.color}15 10px,
        ${discipline.color}15 20px
      )`
      }}>
      <ScrollArea className="h-full">
        <div className="py-3 space-y-3">
          <Card className="shadow-sm flex flex-row gap-0 py-0" style={{ borderColor: `${discipline.color}40` }}>
            <div className="flex-1 min-w-0 py-4">
              <DisciplineContent discipline={discipline} imageUrl={imageUrl} />
            </div>

            <div
              className="w-56 flex-shrink-0 border-l relative overflow-hidden"
              style={{ borderColor: `${discipline.color}40` }}>
              {imageUrl && discipline.crops?.strip && (
                <CroppedImage
                  src={imageUrl}
                  crop={discipline.crops.strip}
                  className="absolute inset-0 w-full h-full object-cover opacity-15"
                />
              )}
              <div className="relative">
                <DisciplineSidebar discipline={discipline} stackName={stackName} />
              </div>
            </div>
          </Card>
        </div>
      </ScrollArea>
    </div>
  )
}
