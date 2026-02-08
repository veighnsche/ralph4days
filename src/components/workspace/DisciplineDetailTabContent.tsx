import { Layers } from 'lucide-react'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Separator } from '@/components/ui/separator'
import { Skeleton } from '@/components/ui/skeleton'
import { useInvoke } from '@/hooks/useInvoke'
import { useTabMeta } from '@/hooks/useTabMeta'
import { resolveIcon } from '@/lib/iconRegistry'
import type { WorkspaceTab } from '@/stores/useWorkspaceStore'
import { useWorkspaceStore } from '@/stores/useWorkspaceStore'
import type { DisciplineConfig } from '@/types/generated'
import { DisciplineFormTabContent } from './DisciplineFormTabContent'

function PropertyRow({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="flex items-center justify-between gap-4 py-2">
      <span className="text-xs font-medium text-muted-foreground">{label}</span>
      <div className="text-sm text-right">{children}</div>
    </div>
  )
}

export function DisciplineDetailTabContent({ tab }: { tab: WorkspaceTab }) {
  const openTab = useWorkspaceStore(state => state.openTab)
  const disciplineName = tab.data?.entityId as string

  const { data: disciplines, isLoading } = useInvoke<DisciplineConfig[]>('get_disciplines_config', undefined, {
    staleTime: 5 * 60 * 1000
  })

  const discipline = disciplines?.find(d => d.name === disciplineName)

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

  return (
    <div className="h-full flex flex-col">
      <div className="border-b p-4">
        <div className="flex items-start justify-between gap-4">
          <div className="flex items-start gap-3">
            <div className="p-3 rounded-md" style={{ backgroundColor: bgColor, color: discipline.color }}>
              <Icon className="h-6 w-6" />
            </div>
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

      <ScrollArea className="flex-1">
        <div className="p-6 space-y-6">
          <Card>
            <CardHeader className="pb-3">
              <CardTitle className="text-sm">Properties</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-0">
                <PropertyRow label="Display Name">{discipline.displayName}</PropertyRow>
                <Separator />
                <PropertyRow label="Acronym">
                  <span className="font-mono">{discipline.acronym}</span>
                </PropertyRow>
                <Separator />
                <PropertyRow label="Internal Name">
                  <span className="font-mono text-xs">{discipline.name}</span>
                </PropertyRow>
                <Separator />
                <PropertyRow label="Icon">
                  <div className="flex items-center gap-2">
                    <Icon className="h-4 w-4" style={{ color: discipline.color }} />
                    <span className="text-xs">{discipline.icon}</span>
                  </div>
                </PropertyRow>
                <Separator />
                <PropertyRow label="Color">
                  <div className="flex items-center gap-2">
                    <div className="w-4 h-4 rounded border" style={{ backgroundColor: discipline.color }} />
                    <span className="text-xs font-mono">{discipline.color}</span>
                  </div>
                </PropertyRow>
              </div>
            </CardContent>
          </Card>

          {discipline.systemPrompt && (
            <Card>
              <CardHeader className="pb-3">
                <CardTitle className="text-sm">System Prompt</CardTitle>
                <CardDescription className="text-xs">
                  The persona Claude Code adopts when executing tasks in this discipline
                </CardDescription>
              </CardHeader>
              <CardContent>
                <pre className="text-xs whitespace-pre-wrap font-mono bg-muted p-3 rounded-md">
                  {discipline.systemPrompt}
                </pre>
              </CardContent>
            </Card>
          )}

          {discipline.skills && discipline.skills.length > 0 && (
            <Card>
              <CardHeader className="pb-3">
                <CardTitle className="text-sm">Skills</CardTitle>
                <CardDescription className="text-xs">Technologies and capabilities for this discipline</CardDescription>
              </CardHeader>
              <CardContent>
                <div className="flex flex-wrap gap-1.5">
                  {discipline.skills.map((skill, index) => (
                    <Badge key={index} variant="secondary" className="text-xs">
                      {skill}
                    </Badge>
                  ))}
                </div>
              </CardContent>
            </Card>
          )}

          {discipline.conventions && (
            <Card>
              <CardHeader className="pb-3">
                <CardTitle className="text-sm">Conventions</CardTitle>
                <CardDescription className="text-xs">
                  Naming patterns, code structure, and quality standards
                </CardDescription>
              </CardHeader>
              <CardContent>
                <pre className="text-xs whitespace-pre-wrap font-mono bg-muted p-3 rounded-md">
                  {discipline.conventions}
                </pre>
              </CardContent>
            </Card>
          )}

          {discipline.mcpServers && discipline.mcpServers.length > 0 && (
            <Card>
              <CardHeader className="pb-3">
                <CardTitle className="text-sm">MCP Servers</CardTitle>
                <CardDescription className="text-xs">
                  Additional tools and resources provided via Model Context Protocol
                </CardDescription>
              </CardHeader>
              <CardContent>
                <div className="space-y-3">
                  {discipline.mcpServers.map((server, index) => (
                    <div key={index}>
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
              </CardContent>
            </Card>
          )}
        </div>
      </ScrollArea>
    </div>
  )
}
