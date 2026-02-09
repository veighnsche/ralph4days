import { invoke } from '@tauri-apps/api/core'
import { Layers } from 'lucide-react'
import { useEffect, useState } from 'react'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Separator } from '@/components/ui/separator'
import { Skeleton } from '@/components/ui/skeleton'
import { useInvoke } from '@/hooks/api'
import { useStackMetadata } from '@/hooks/disciplines'
import { useTabMeta } from '@/hooks/workspace'
import { resolveIcon } from '@/lib/iconRegistry'
import type { WorkspaceTab } from '@/stores/useWorkspaceStore'
import { useWorkspaceStore } from '@/stores/useWorkspaceStore'
import type { DisciplineConfig } from '@/types/generated'
import { DisciplineFormTabContent } from './DisciplineFormTabContent'
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

// TODO: Replace useDisciplineImage above with useDisciplineImageStore once TaskIdDisplay integration is validated

function buildSections(discipline: DisciplineConfig) {
  const sections: React.ReactNode[] = []

  if (discipline.systemPrompt) {
    sections.push(
      <div key="system-prompt" className="space-y-2">
        <h2 className="text-sm font-medium text-muted-foreground">System Prompt</h2>
        <pre className="text-xs whitespace-pre-wrap font-mono bg-muted p-3 rounded-md">{discipline.systemPrompt}</pre>
      </div>
    )
  }

  if (discipline.skills && discipline.skills.length > 0) {
    sections.push(
      <div key="skills" className="space-y-2">
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
      <div key="conventions" className="space-y-2">
        <h2 className="text-sm font-medium text-muted-foreground">Conventions</h2>
        <pre className="text-xs whitespace-pre-wrap font-mono bg-muted p-3 rounded-md">{discipline.conventions}</pre>
      </div>
    )
  }

  if (discipline.mcpServers && discipline.mcpServers.length > 0) {
    sections.push(
      <div key="mcp-servers" className="space-y-2">
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

  return sections
}

function SectionList({ sections }: { sections: React.ReactNode[] }) {
  if (sections.length === 0) {
    return <p className="text-sm text-muted-foreground">No details available for this discipline.</p>
  }
  return (
    <div className="space-y-6">
      {sections.flatMap((section, i) =>
        i === 0 ? [section] : [<Separator key={`sep-${(section as React.ReactElement).key}`} />, section]
      )}
    </div>
  )
}

export function DisciplineDetailTabContent({ tab }: { tab: WorkspaceTab }) {
  const openTab = useWorkspaceStore(state => state.openTab)
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

  const Icon = resolveIcon(discipline.icon)
  const bgColor = `color-mix(in oklch, ${discipline.color} 15%, transparent)`

  const handleEdit = () => {
    openTab({
      type: 'discipline-form',
      component: DisciplineFormTabContent,
      title: `Edit ${discipline.displayName}`,
      closeable: true,
      data: {
        mode: 'edit',
        entityId: discipline.name
      }
    })
  }

  const sections = buildSections(discipline)

  return (
    <div className="h-full flex flex-col">
      <div className="border-b p-4">
        <div className="flex items-start justify-between gap-4">
          <div className="flex items-start gap-3">
            {imageUrl ? (
              <img
                src={imageUrl}
                alt={discipline.displayName}
                className="h-12 w-12 rounded-md object-cover"
                style={{
                  objectPosition: discipline.crops?.face
                    ? `${(discipline.crops.face.x + discipline.crops.face.w / 2) * 100}% ${(discipline.crops.face.y + discipline.crops.face.h / 2) * 100}%`
                    : '50% 15%'
                }}
              />
            ) : (
              <div className="p-3 rounded-md" style={{ backgroundColor: bgColor, color: discipline.color }}>
                <Icon className="h-6 w-6" />
              </div>
            )}
            <div>
              <h1 className="text-xl font-semibold">{discipline.displayName}</h1>
              <div className="flex items-center gap-2 mt-1">
                <Badge variant="outline" className="font-mono">
                  {discipline.acronym}
                </Badge>
                <span className="text-xs text-muted-foreground">·</span>
                <span className="text-xs text-muted-foreground font-mono">{discipline.name}</span>
              </div>
            </div>
          </div>
          <Button onClick={handleEdit} size="sm">
            Edit
          </Button>
        </div>
      </div>

      <ScrollArea className="flex-1 min-h-0">
        <div className="flex gap-6 p-6">
          <div className="flex-1 min-w-0">
            <Card>
              <CardContent className="py-6">
                <SectionList sections={sections} />
              </CardContent>
            </Card>
          </div>

          <div className="w-56 shrink-0">
            <Card>
              <CardHeader className="pb-3">
                <CardTitle className="text-sm">Properties</CardTitle>
              </CardHeader>
              <CardContent className="space-y-0">
                <PropertyRow label="Color">
                  <div className="flex items-center gap-2">
                    <div className="w-4 h-4 rounded border" style={{ backgroundColor: discipline.color }} />
                    <span className="text-xs font-mono">{discipline.color}</span>
                  </div>
                </PropertyRow>
                <Separator />
                <PropertyRow label="Icon">
                  <div className="flex items-center gap-2">
                    <Icon className="h-4 w-4" style={{ color: discipline.color }} />
                    <span className="text-xs">{discipline.icon}</span>
                  </div>
                </PropertyRow>
                <Separator />
                <PropertyRow label="Acronym">
                  <span className="font-mono text-sm">{discipline.acronym}</span>
                </PropertyRow>
                <Separator />
                <PropertyRow label="Internal Name">
                  <span className="font-mono text-xs">{discipline.name}</span>
                </PropertyRow>
                {stackName && (
                  <>
                    <Separator />
                    <PropertyRow label="Stack">
                      <span className="text-sm">{stackName}</span>
                    </PropertyRow>
                  </>
                )}
              </CardContent>
            </Card>
          </div>
        </div>
      </ScrollArea>
    </div>
  )
}
