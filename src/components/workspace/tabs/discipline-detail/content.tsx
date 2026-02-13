import { Eye, Layers, Play, Settings2 } from 'lucide-react'
import { DisciplineLabel } from '@/components/prd/DisciplineLabel'
import { Badge } from '@/components/ui/badge'
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

function DisciplineSidebar({
  discipline,
  stackName,
  templates
}: {
  discipline: DisciplineConfig
  stackName?: string
  templates: DisciplineTemplate[]
}) {
  const Icon = resolveIcon(discipline.icon)

  return (
    <div className="px-3 pt-0 pb-3 space-y-0.5 overflow-y-auto h-full">
      <div className="-mx-3">
        {templates.length === 0 ? (
          <div className="px-3 text-xs text-muted-foreground">No active templates for this discipline yet.</div>
        ) : (
          <div className="space-y-1.5">
            {templates.map(template => (
              <TaskTemplateCard key={template.id} template={template} />
            ))}
          </div>
        )}
      </div>

      <Separator bleed="md" />

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
    <div className="rounded-md border border-border/60 bg-muted/20 overflow-hidden flex">
      <div className="flex-1 min-w-0 px-2.5 py-2 space-y-1 bg-muted/20">
        <h3 className="text-xs font-medium leading-tight">{template.title}</h3>
        {template.description && (
          <p className="text-xs text-muted-foreground leading-relaxed line-clamp-2">{template.description}</p>
        )}
      </div>

      <div className="w-10 shrink-0 border-l border-border/60 flex flex-col bg-muted/25">
        <button
          type="button"
          className="min-h-7 flex-1 px-1 flex items-center justify-center border-b border-border/60 text-muted-foreground bg-muted/25 hover:text-foreground hover:bg-muted/40 transition-colors"
          aria-label={`Play template ${template.title}`}>
          <Play className="h-3 w-3" />
        </button>
        <button
          type="button"
          className="min-h-7 flex-1 px-1 flex items-center justify-center border-b border-border/60 text-muted-foreground bg-muted/25 hover:text-foreground hover:bg-muted/40 transition-colors"
          aria-label={`View template ${template.title}`}>
          <Eye className="h-3 w-3" />
        </button>
        <button
          type="button"
          className="min-h-7 flex-1 px-1 flex items-center justify-center text-muted-foreground bg-muted/25 hover:text-foreground hover:bg-muted/40 transition-colors"
          aria-label={`Template settings for ${template.title}`}>
          <Settings2 className="h-3 w-3" />
        </button>
      </div>
    </div>
  )
}

export function DisciplineDetailTabContent({ tab, params }: { tab: WorkspaceTab; params: DisciplineDetailTabParams }) {
  const { entityId: disciplineId } = params

  const { data: disciplines, isLoading } = useInvoke<DisciplineConfig[]>('get_disciplines_config', undefined, {
    queryDomain: 'workspace',
    staleTime: 5 * 60 * 1000
  })
  const { stacks } = useStackMetadata('workspace')

  const discipline = disciplines?.find(d => d.id === disciplineId)
  const stackName = discipline?.stackId != null ? stacks.find(s => s.stackId === discipline.stackId)?.name : undefined
  const taskTemplates = discipline?.taskTemplates ?? []

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
      sidebar={<DisciplineSidebar discipline={discipline} stackName={stackName} templates={taskTemplates} />}
    />
  )
}
