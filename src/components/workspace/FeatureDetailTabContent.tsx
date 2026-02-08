import { FileCode, Puzzle } from 'lucide-react'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Separator } from '@/components/ui/separator'
import { Skeleton } from '@/components/ui/skeleton'
import { useInvoke } from '@/hooks/api'
import { useFeatureStats } from '@/hooks/features'
import { useTabMeta } from '@/hooks/workspace'
import { formatDate } from '@/lib/formatDate'
import type { WorkspaceTab } from '@/stores/useWorkspaceStore'
import { useWorkspaceStore } from '@/stores/useWorkspaceStore'
import type { FeatureData } from '@/types/generated'
import { FeatureFormTabContent } from './FeatureFormTabContent'
import { PropertyRow } from './PropertyRow'

function buildSections(feature: FeatureData) {
  const sections: React.ReactNode[] = []

  if (feature.description) {
    sections.push(
      <div key="description" className="space-y-2">
        <h2 className="text-sm font-medium text-muted-foreground">Description</h2>
        <p className="text-sm leading-relaxed whitespace-pre-wrap">{feature.description}</p>
      </div>
    )
  }

  if (feature.architecture) {
    sections.push(
      <div key="architecture" className="space-y-2">
        <h2 className="text-sm font-medium text-muted-foreground">Architecture</h2>
        <pre className="text-xs whitespace-pre-wrap font-mono bg-muted p-3 rounded-md">{feature.architecture}</pre>
      </div>
    )
  }

  if (feature.boundaries) {
    sections.push(
      <div key="boundaries" className="space-y-2">
        <h2 className="text-sm font-medium text-muted-foreground">Boundaries</h2>
        <pre className="text-xs whitespace-pre-wrap font-mono bg-muted p-3 rounded-md">{feature.boundaries}</pre>
      </div>
    )
  }

  if (feature.learnings.length > 0) {
    sections.push(
      <div key="learnings" className="space-y-2">
        <h2 className="text-sm font-medium text-muted-foreground">Learnings</h2>
        <ul className="space-y-2">
          {feature.learnings.map(learning => (
            <li key={learning.text} className="text-sm border rounded-md p-3 space-y-1.5">
              <p className="leading-relaxed">{learning.text}</p>
              <div className="flex flex-wrap items-center gap-1.5 text-xs text-muted-foreground">
                <Badge variant="secondary" className="text-xs px-1.5 py-0 h-5">
                  {learning.source}
                </Badge>
                {learning.hitCount > 0 && <span>{learning.hitCount} hits</span>}
                {learning.taskId != null && (
                  <span className="font-mono">#{learning.taskId.toString().padStart(3, '0')}</span>
                )}
                <span>{formatDate(learning.created)}</span>
              </div>
            </li>
          ))}
        </ul>
      </div>
    )
  }

  if (feature.contextFiles.length > 0 || feature.knowledgePaths.length > 0) {
    sections.push(
      <div key="files" className="space-y-2">
        <h2 className="text-sm font-medium text-muted-foreground">Files</h2>
        <div className="space-y-1.5">
          {feature.contextFiles.length > 0 && (
            <div className="flex flex-wrap items-center gap-1.5">
              <span className="text-xs text-muted-foreground">Context:</span>
              {feature.contextFiles.map(file => (
                <Badge key={file} variant="outline" className="text-xs font-mono px-2 py-0.5 h-5 gap-1">
                  <FileCode className="h-3 w-3 text-muted-foreground" />
                  {file}
                </Badge>
              ))}
            </div>
          )}
          {feature.knowledgePaths.length > 0 && (
            <div className="flex flex-wrap items-center gap-1.5">
              <span className="text-xs text-muted-foreground">Knowledge:</span>
              {feature.knowledgePaths.map(path => (
                <Badge key={path} variant="outline" className="text-xs font-mono px-2 py-0.5 h-5 gap-1">
                  <FileCode className="h-3 w-3 text-muted-foreground" />
                  {path}
                </Badge>
              ))}
            </div>
          )}
        </div>
      </div>
    )
  }

  return sections
}

function SectionList({ sections }: { sections: React.ReactNode[] }) {
  if (sections.length === 0) {
    return <p className="text-sm text-muted-foreground">No details available for this feature.</p>
  }
  return (
    <div className="space-y-6">
      {sections.flatMap((section, i) =>
        i === 0 ? [section] : [<Separator key={`sep-${(section as React.ReactElement).key}`} />, section]
      )}
    </div>
  )
}

export function FeatureDetailTabContent({ tab }: { tab: WorkspaceTab }) {
  const openTab = useWorkspaceStore(state => state.openTab)
  const featureName = tab.data?.entityId as string

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

  const handleEdit = () => {
    openTab({
      type: 'feature-form',
      component: FeatureFormTabContent,
      title: `Edit ${feature.displayName}`,
      closeable: true,
      data: {
        mode: 'edit',
        entityId: feature.name
      }
    })
  }

  const sections = buildSections(feature)

  return (
    <div className="h-full flex flex-col">
      <div className="border-b p-4">
        <div className="flex items-start justify-between gap-4">
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
          <Button onClick={handleEdit} size="sm">
            Edit
          </Button>
        </div>
      </div>

      <ScrollArea className="flex-1">
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
                <PropertyRow label="Acronym">
                  <span className="font-mono text-sm">{feature.acronym}</span>
                </PropertyRow>
                <Separator />
                <PropertyRow label="Internal Name">
                  <span className="font-mono text-xs">{feature.name}</span>
                </PropertyRow>
                {feature.created && (
                  <>
                    <Separator />
                    <PropertyRow label="Created">
                      <span className="text-xs text-muted-foreground">{formatDate(feature.created)}</span>
                    </PropertyRow>
                  </>
                )}
                <Separator />
                <PropertyRow label="Tasks">
                  <div className="flex items-center gap-1.5">
                    <span className="text-sm">
                      {stats.done}/{stats.total}
                    </span>
                    <span className="text-xs text-muted-foreground">({featureProgress}%)</span>
                  </div>
                </PropertyRow>
                {feature.dependencies.length > 0 && (
                  <>
                    <Separator />
                    <PropertyRow label="Dependencies">
                      <div className="flex flex-wrap gap-1">
                        {feature.dependencies.map(dep => (
                          <Badge key={dep} variant="secondary" className="text-xs px-1.5 py-0 h-5">
                            {dep}
                          </Badge>
                        ))}
                      </div>
                    </PropertyRow>
                  </>
                )}
                {feature.contextFiles.length > 0 && (
                  <>
                    <Separator />
                    <PropertyRow label="Context Files">
                      <span className="text-sm">{feature.contextFiles.length}</span>
                    </PropertyRow>
                  </>
                )}
                {feature.knowledgePaths.length > 0 && (
                  <>
                    <Separator />
                    <PropertyRow label="Knowledge Paths">
                      <span className="text-sm">{feature.knowledgePaths.length}</span>
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
