import { Layers } from 'lucide-react'
import { DisciplineLabel } from '@/components/prd/DisciplineLabel'
import { Badge } from '@/components/ui/badge'
import { Card } from '@/components/ui/card'
import { CroppedImage } from '@/components/ui/cropped-image'
import { Separator } from '@/components/ui/separator'
import { Skeleton } from '@/components/ui/skeleton'
import { useInvoke } from '@/hooks/api'
import { useStackMetadata } from '@/hooks/disciplines'
import { useTabMeta } from '@/hooks/workspace'
import { resolveIcon } from '@/lib/iconRegistry'
import type { WorkspaceTab } from '@/stores/useWorkspaceStore'
import type { DisciplineConfig } from '@/types/generated'
import { DetailPageLayout } from '../../DetailPageLayout'
import { PropertyRow } from '../../PropertyRow'
import {
  DISCIPLINE_DETAIL_EMPTY_STATE_DESCRIPTION,
  DISCIPLINE_DETAIL_EMPTY_STATE_TITLE,
  DISCIPLINE_DETAIL_FALLBACK_EYELINE_PERCENT,
  DISCIPLINE_DETAIL_TAB_FALLBACK_TITLE
} from './constants'
import type { DisciplineDetailTabParams } from './schema'

type DisciplineTemplate = DisciplineConfig['taskTemplates'][number]

function DisciplineContent({ discipline }: { discipline: DisciplineConfig }) {
  const Icon = resolveIcon(discipline.icon)
  const sections: React.ReactNode[] = []

  sections.push(
    <div key="header" className="px-6 flex items-start gap-3">
      {discipline.imagePath && discipline.crops?.face ? (
        <CroppedImage
          disciplineName={discipline.name}
          label="face"
          crop={discipline.crops.face}
          className="h-12 w-12 rounded-md shrink-0"
        />
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

      <Separator bleed="md" />
      <PropertyRow label="Launch Defaults">
        <div className="space-y-1.5 text-xs">
          {discipline.agent && (
            <div>
              <span className="text-muted-foreground">Agent:</span>{' '}
              <code className="font-mono bg-muted px-1.5 py-0.5 rounded">{discipline.agent}</code>
            </div>
          )}
          {discipline.model && (
            <div>
              <span className="text-muted-foreground">Model:</span>{' '}
              <code className="font-mono bg-muted px-1.5 py-0.5 rounded">{discipline.model}</code>
            </div>
          )}
          {discipline.effort && (
            <div>
              <span className="text-muted-foreground">Effort:</span>{' '}
              <code className="font-mono bg-muted px-1.5 py-0.5 rounded">{discipline.effort}</code>
            </div>
          )}
          {discipline.model && discipline.thinking !== undefined && (
            <div>
              <span className="text-muted-foreground">Thinking:</span>{' '}
              <code className="font-mono bg-muted px-1.5 py-0.5 rounded">{discipline.thinking ? 'on' : 'off'}</code>
            </div>
          )}
          {!discipline.model && <span className="text-muted-foreground">No model set</span>}
        </div>
      </PropertyRow>
    </div>
  )
}

function TaskTemplateCard({ template }: { template: DisciplineTemplate }) {
  return (
    <Card className="p-4 space-y-3">
      <div className="space-y-1">
        <h3 className="text-sm font-medium leading-tight">{template.title}</h3>
        {template.description && (
          <p className="text-xs text-muted-foreground leading-relaxed whitespace-pre-wrap">{template.description}</p>
        )}
      </div>

      <div className="flex flex-wrap gap-1.5">
        {template.priority && (
          <Badge variant="outline" className="text-[10px] font-mono uppercase">
            {template.priority}
          </Badge>
        )}
        {template.estimatedTurns != null && (
          <Badge variant="secondary" className="text-[10px] font-mono">
            {template.estimatedTurns} turns
          </Badge>
        )}
        <Badge variant="secondary" className="text-[10px] font-mono">
          pulled {template.pulledCount}x
        </Badge>
      </div>

      {(template.hints || template.agent || template.model || template.effort || template.thinking != null) && (
        <div className="text-xs text-muted-foreground space-y-1">
          {template.hints && <p className="leading-relaxed whitespace-pre-wrap">{template.hints}</p>}
          {(template.agent || template.model || template.effort || template.thinking != null) && (
            <p className="font-mono">
              {[
                template.agent,
                template.model,
                template.effort,
                template.thinking != null ? `thinking:${template.thinking ? 'on' : 'off'}` : null
              ]
                .filter(Boolean)
                .join(' · ')}
            </p>
          )}
        </div>
      )}
    </Card>
  )
}

function DisciplineTaskTemplatesSection({ discipline }: { discipline: DisciplineConfig }) {
  const templates = discipline.taskTemplates ?? []
  return (
    <Card className="shadow-sm p-0">
      <div className="px-4 py-3 border-b">
        <h2 className="text-sm font-medium">Task Templates</h2>
        <p className="text-xs text-muted-foreground mt-1">Reusable definitions this discipline can pull into tasks.</p>
      </div>

      {templates.length === 0 ? (
        <div className="px-4 py-6 text-sm text-muted-foreground">No active templates for this discipline yet.</div>
      ) : (
        <div className="p-3 grid grid-cols-1 gap-3">
          {templates.map(template => (
            <TaskTemplateCard key={template.id} template={template} />
          ))}
        </div>
      )}
    </Card>
  )
}

export function DisciplineDetailTabContent({ tab, params }: { tab: WorkspaceTab; params: DisciplineDetailTabParams }) {
  const { entityId: disciplineId } = params

  const { data: disciplines, isLoading } = useInvoke<DisciplineConfig[]>('get_disciplines_config', undefined, {
    staleTime: 5 * 60 * 1000
  })
  const { stacks } = useStackMetadata()

  const discipline = disciplines?.find(d => d.id === disciplineId)
  const stackName = discipline?.stackId != null ? stacks.find(s => s.stackId === discipline.stackId)?.name : undefined

  useTabMeta(tab.id, discipline?.displayName ?? DISCIPLINE_DETAIL_TAB_FALLBACK_TITLE, Layers)

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
          <h2 className="text-lg font-semibold mb-2">{DISCIPLINE_DETAIL_EMPTY_STATE_TITLE}</h2>
          <p className="text-sm text-muted-foreground">{DISCIPLINE_DETAIL_EMPTY_STATE_DESCRIPTION}</p>
        </div>
      </div>
    )
  }

  const borderColor = `${discipline.color}40`
  const faceCrop = discipline.crops?.face
  const eyelinePercent = faceCrop
    ? Math.round((faceCrop.y + faceCrop.h / 2) * 100)
    : DISCIPLINE_DETAIL_FALLBACK_EYELINE_PERCENT

  return (
    <DetailPageLayout
      accentColor={discipline.color}
      cardBorderColor={borderColor}
      sidebarImage={
        discipline.imagePath &&
        discipline.crops?.strip && (
          <CroppedImage
            disciplineName={discipline.name}
            label="strip"
            crop={discipline.crops.strip}
            className="absolute inset-0 w-full h-full object-cover opacity-15"
            style={{ objectPosition: `center ${eyelinePercent}%` }}
          />
        )
      }
      mainContent={<DisciplineContent discipline={discipline} />}
      sidebar={<DisciplineSidebar discipline={discipline} stackName={stackName} />}>
      <DisciplineTaskTemplatesSection discipline={discipline} />
    </DetailPageLayout>
  )
}
