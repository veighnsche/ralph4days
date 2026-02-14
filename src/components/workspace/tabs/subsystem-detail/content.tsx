import { Puzzle } from 'lucide-react'
import { Badge } from '@/components/ui/badge'
import { Separator } from '@/components/ui/separator'
import { Skeleton } from '@/components/ui/skeleton'
import { useInvoke } from '@/hooks/api'
import { useSubsystemStats } from '@/hooks/subsystems'
import { useTabMeta } from '@/hooks/workspace'
import { formatDate } from '@/lib/formatDate'
import type { WorkspaceTab } from '@/stores/useWorkspaceStore'
import type { SubsystemData } from '@/types/generated'
import { DetailPageLayout } from '../../DetailPageLayout'
import { PropertyRow } from '../../PropertyRow'
import { SubsystemCommentsSection } from '../../subsystem-detail/SubsystemCommentsSection'
import {
  SUBSYSTEM_DETAIL_EMPTY_DETAILS_MESSAGE,
  SUBSYSTEM_DETAIL_EMPTY_STATE_DESCRIPTION,
  SUBSYSTEM_DETAIL_EMPTY_STATE_TITLE,
  SUBSYSTEM_DETAIL_TAB_FALLBACK_TITLE
} from './constants'
import type { SubsystemDetailTabParams } from './schema'

function SubsystemContent({ subsystem }: { subsystem: SubsystemData }) {
  const sections: React.ReactNode[] = []

  sections.push(
    <div key="header" className="px-6 flex items-start justify-between gap-4">
      <div className="flex items-start gap-3">
        <div className="p-3 rounded-md bg-muted">
          <Puzzle className="h-6 w-6" />
        </div>
        <div>
          <h1 className="text-xl font-semibold">{subsystem.displayName}</h1>
          <div className="flex items-center gap-2 mt-1">
            <Badge variant="outline" className="font-mono">
              {subsystem.acronym}
            </Badge>
            <span className="text-xs text-muted-foreground">·</span>
            <span className="text-xs text-muted-foreground font-mono">{subsystem.name}</span>
          </div>
        </div>
      </div>
    </div>
  )

  if (subsystem.description) {
    sections.push(
      <div key="description" className="px-6 space-y-2">
        <h2 className="text-sm font-medium text-muted-foreground">Description</h2>
        <p className="text-sm leading-relaxed whitespace-pre-wrap">{subsystem.description}</p>
      </div>
    )
  }

  if (sections.length <= 1) {
    return (
      <div className="space-y-6">
        {sections}
        {sections.length <= 1 && (
          <p className="px-6 text-sm text-muted-foreground">{SUBSYSTEM_DETAIL_EMPTY_DETAILS_MESSAGE}</p>
        )}
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {sections.flatMap((section, i) =>
        i === 0 ? [section] : [<Separator key={`sep-${(section as React.ReactElement).key}`} />, section]
      )}
    </div>
  )
}

function SubsystemSidebar({
  subsystem,
  stats,
  featureProgress
}: {
  subsystem: SubsystemData
  stats: { total: number; done: number; pending: number; inProgress: number; blocked: number; skipped: number }
  featureProgress: number
}) {
  return (
    <div className="px-4 py-4 space-y-0.5 overflow-y-auto h-full">
      <PropertyRow label="Status">
        <Badge variant={subsystem.status === 'active' ? 'default' : 'secondary'} className="text-xs px-1.5 py-0 h-5">
          {subsystem.status}
        </Badge>
      </PropertyRow>
      <Separator bleed="md" />
      <PropertyRow label="Acronym">
        <span className="font-mono text-sm">{subsystem.acronym}</span>
      </PropertyRow>
      <Separator bleed="md" />
      <PropertyRow label="Internal Name">
        <span className="font-mono text-xs">{subsystem.name}</span>
      </PropertyRow>
      {subsystem.created && (
        <>
          <Separator bleed="md" />
          <PropertyRow label="Created">
            <span className="text-xs text-muted-foreground">{formatDate(subsystem.created)}</span>
          </PropertyRow>
        </>
      )}
      <Separator bleed="md" />
      <PropertyRow label="Tasks">
        <div className="flex items-center gap-1.5">
          <span className="text-sm">
            {stats.done}/{stats.total}
          </span>
          <span className="text-xs text-muted-foreground">({featureProgress}%)</span>
        </div>
      </PropertyRow>
      {subsystem.comments.length > 0 && (
        <>
          <Separator bleed="md" />
          <PropertyRow label="Comments">
            <span className="text-sm">{subsystem.comments.length}</span>
          </PropertyRow>
        </>
      )}
    </div>
  )
}

export function SubsystemDetailTabContent({ tab, params }: { tab: WorkspaceTab; params: SubsystemDetailTabParams }) {
  const { entityId: subsystemId } = params

  const { data: subsystems, isLoading } = useInvoke<SubsystemData[]>('subsystems_list', undefined, {
    queryDomain: 'workspace',
    staleTime: 5 * 60 * 1000
  })
  const { statsMap } = useSubsystemStats('workspace')

  const subsystem = subsystems?.find(f => f.id === subsystemId)
  const stats = statsMap.get(subsystem?.name ?? '') || {
    total: 0,
    done: 0,
    pending: 0,
    inProgress: 0,
    blocked: 0,
    skipped: 0
  }
  const featureProgress = stats.total > 0 ? Math.round((stats.done / stats.total) * 100) : 0

  useTabMeta(tab.id, subsystem?.displayName ?? SUBSYSTEM_DETAIL_TAB_FALLBACK_TITLE, Puzzle)

  if (isLoading) {
    return (
      <div className="h-full flex flex-col">
        <div className="flex-1 p-6">
          <Skeleton className="h-[200px]" />
        </div>
      </div>
    )
  }

  if (!subsystem) {
    return (
      <div className="h-full flex items-center justify-center">
        <div className="text-center">
          <Puzzle className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
          <h2 className="text-lg font-semibold mb-2">{SUBSYSTEM_DETAIL_EMPTY_STATE_TITLE}</h2>
          <p className="text-sm text-muted-foreground">{SUBSYSTEM_DETAIL_EMPTY_STATE_DESCRIPTION}</p>
        </div>
      </div>
    )
  }

  return (
    <DetailPageLayout
      accentColor="hsl(var(--muted-foreground))"
      mainContent={<SubsystemContent subsystem={subsystem} />}
      sidebar={<SubsystemSidebar subsystem={subsystem} stats={stats} featureProgress={featureProgress} />}>
      <SubsystemCommentsSection subsystem={subsystem} />
    </DetailPageLayout>
  )
}
