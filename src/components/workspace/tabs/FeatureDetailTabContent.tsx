import { Puzzle } from 'lucide-react'
import { Badge } from '@/components/ui/badge'
import { Separator } from '@/components/ui/separator'
import { Skeleton } from '@/components/ui/skeleton'
import { useInvoke } from '@/hooks/api'
import { useFeatureStats } from '@/hooks/features'
import { useTabMeta } from '@/hooks/workspace'
import { formatDate } from '@/lib/formatDate'
import type { WorkspaceTab } from '@/stores/useWorkspaceStore'
import type { FeatureData } from '@/types/generated'
import { DetailPageLayout } from '../DetailPageLayout'
import { FeatureCommentsSection } from '../feature-detail/FeatureCommentsSection'
import { PropertyRow } from '../PropertyRow'

function FeatureContent({ feature }: { feature: FeatureData }) {
  const sections: React.ReactNode[] = []

  sections.push(
    <div key="header" className="px-6 flex items-start justify-between gap-4">
      <div className="flex items-start gap-3">
        <div className="p-3 rounded-md bg-muted">
          <Puzzle className="h-6 w-6" />
        </div>
        <div>
          <h1 className="text-xl font-semibold">{feature.displayName}</h1>
          <div className="flex items-center gap-2 mt-1">
            <Badge variant="outline" className="font-mono">
              {feature.acronym}
            </Badge>
            <span className="text-xs text-muted-foreground">·</span>
            <span className="text-xs text-muted-foreground font-mono">{feature.name}</span>
          </div>
        </div>
      </div>
    </div>
  )

  if (feature.description) {
    sections.push(
      <div key="description" className="px-6 space-y-2">
        <h2 className="text-sm font-medium text-muted-foreground">Description</h2>
        <p className="text-sm leading-relaxed whitespace-pre-wrap">{feature.description}</p>
      </div>
    )
  }

  if (sections.length <= 1) {
    return (
      <div className="space-y-6">
        {sections}
        {sections.length <= 1 && (
          <p className="px-6 text-sm text-muted-foreground">No details available for this feature.</p>
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

function FeatureSidebar({
  feature,
  stats,
  featureProgress
}: {
  feature: FeatureData
  stats: { total: number; done: number; pending: number; inProgress: number; blocked: number; skipped: number }
  featureProgress: number
}) {
  return (
    <div className="px-4 py-4 space-y-0.5 overflow-y-auto h-full">
      <PropertyRow label="Status">
        <Badge variant={feature.status === 'active' ? 'default' : 'secondary'} className="text-xs px-1.5 py-0 h-5">
          {feature.status}
        </Badge>
      </PropertyRow>
      <Separator bleed="md" />
      <PropertyRow label="Acronym">
        <span className="font-mono text-sm">{feature.acronym}</span>
      </PropertyRow>
      <Separator bleed="md" />
      <PropertyRow label="Internal Name">
        <span className="font-mono text-xs">{feature.name}</span>
      </PropertyRow>
      {feature.created && (
        <>
          <Separator bleed="md" />
          <PropertyRow label="Created">
            <span className="text-xs text-muted-foreground">{formatDate(feature.created)}</span>
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
      {feature.comments.length > 0 && (
        <>
          <Separator bleed="md" />
          <PropertyRow label="Comments">
            <span className="text-sm">{feature.comments.length}</span>
          </PropertyRow>
        </>
      )}
    </div>
  )
}

export type FeatureDetailTabParams = {
  entityId: string
}

function parseFeatureDetailTabParams(params: unknown): FeatureDetailTabParams {
  if (typeof params !== 'object' || params == null || Array.isArray(params)) {
    throw new Error('Invalid feature detail tab params: expected object')
  }
  const candidate = params as Record<string, unknown>
  if (typeof candidate.entityId !== 'string' || candidate.entityId.trim() === '') {
    throw new Error('Invalid feature detail tab params.entityId')
  }
  return { entityId: candidate.entityId }
}

export function createFeatureDetailTab(feature: FeatureData): Omit<WorkspaceTab, 'id'> {
  return {
    type: 'feature-detail',
    title: feature.displayName,
    key: feature.name,
    closeable: true,
    params: {
      entityId: feature.name
    } satisfies FeatureDetailTabParams
  }
}

export function FeatureDetailTabContent({ tab }: { tab: WorkspaceTab }) {
  const { entityId: featureName } = parseFeatureDetailTabParams(tab.params)

  const { data: features, isLoading } = useInvoke<FeatureData[]>('get_features', undefined, {
    staleTime: 5 * 60 * 1000
  })
  const { statsMap } = useFeatureStats()

  const feature = features?.find(f => f.name === featureName)
  const stats = statsMap.get(featureName) || {
    total: 0,
    done: 0,
    pending: 0,
    inProgress: 0,
    blocked: 0,
    skipped: 0
  }
  const featureProgress = stats.total > 0 ? Math.round((stats.done / stats.total) * 100) : 0

  useTabMeta(tab.id, feature?.displayName ?? 'Feature', Puzzle)

  if (isLoading) {
    return (
      <div className="h-full flex flex-col">
        <div className="flex-1 p-6">
          <Skeleton className="h-[200px]" />
        </div>
      </div>
    )
  }

  if (!feature) {
    return (
      <div className="h-full flex items-center justify-center">
        <div className="text-center">
          <Puzzle className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
          <h2 className="text-lg font-semibold mb-2">Feature not found</h2>
          <p className="text-sm text-muted-foreground">The requested feature could not be loaded</p>
        </div>
      </div>
    )
  }

  return (
    <DetailPageLayout
      accentColor="hsl(var(--muted-foreground))"
      mainContent={<FeatureContent feature={feature} />}
      sidebar={<FeatureSidebar feature={feature} stats={stats} featureProgress={featureProgress} />}>
      <FeatureCommentsSection feature={feature} />
    </DetailPageLayout>
  )
}
