import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { FolderOpen } from 'lucide-react'
import { useState } from 'react'
import { InlineError } from '@/components/InlineError'
import { Button } from '@/components/ui/button'
import { Dialog, DialogContent, DialogTitle } from '@/components/ui/dialog'
import { Field, FieldDescription, FieldGroup, FieldLabel } from '@/components/ui/field'
import { InputGroup, InputGroupAddon, InputGroupButton, InputGroupInput } from '@/components/ui/input-group'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Separator } from '@/components/ui/separator'
import { useInvoke } from '@/hooks/useInvoke'
import type { RalphProject } from '@/types/generated'

interface ProjectSelectorProps {
  onProjectSelected: (path: string) => void
}

export function ProjectSelector({ onProjectSelected }: ProjectSelectorProps) {
  const [initPath, setInitPath] = useState('')
  const [initializing, setInitializing] = useState(false)
  const [initError, setInitError] = useState<string | null>(null)
  const [openError, setOpenError] = useState<string | null>(null)

  const {
    data: projects = [],
    isLoading: scanning,
    error: scanError
  } = useInvoke<RalphProject[]>('scan_for_ralph_projects')
  const [selectedProject, setSelectedProject] = useState('')

  const handleBrowseInit = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Select Directory to Initialize'
      })
      if (selected && typeof selected === 'string') {
        setInitPath(selected)
      }
    } catch (e) {
      console.error('Failed to open folder dialog:', e)
    }
  }

  const handleInitialize = async () => {
    if (!initPath) return
    setInitError(null)
    setInitializing(true)
    try {
      const title = initPath.split('/').pop() || 'Project'
      await invoke('initialize_ralph_project', { path: initPath, projectTitle: title })
      await invoke('set_locked_project', { path: initPath })
      onProjectSelected(initPath)
    } catch (err) {
      setInitError(`Failed to initialize: ${err}`)
    } finally {
      setInitializing(false)
    }
  }

  const handleOpenProject = async () => {
    if (!selectedProject) return
    setOpenError(null)
    try {
      await invoke('set_locked_project', { path: selectedProject })
      onProjectSelected(selectedProject)
    } catch (err) {
      setOpenError(`Failed to open: ${err}`)
    }
  }

  return (
    <Dialog open={true}>
      <DialogContent className="max-w-[700px]" showCloseButton={false}>
        <div className="grid grid-cols-[1fr_auto_1fr] gap-4">
          <div className="flex flex-col">
            <DialogTitle>Initialize Existing Project</DialogTitle>
            <FieldGroup className="flex-1">
              <div className="flex-1">
                <Field>
                  <FieldLabel>Project Directory</FieldLabel>
                  <InputGroup>
                    <InputGroupInput
                      value={initPath}
                      onChange={e => setInitPath(e.target.value)}
                      placeholder="/path/to/your-project"
                    />
                    <InputGroupAddon align="inline-end">
                      <InputGroupButton size="icon-xs" onClick={handleBrowseInit}>
                        <FolderOpen className="h-4 w-4" />
                      </InputGroupButton>
                    </InputGroupAddon>
                  </InputGroup>
                  <FieldDescription>Creates .ralph/ folder with template files</FieldDescription>
                </Field>
              </div>

              <InlineError error={initError} onDismiss={() => setInitError(null)} />
              <Button onClick={handleInitialize} disabled={!initPath || initializing} className="w-full">
                {initializing ? 'Initializing...' : 'Initialize Ralph'}
              </Button>
            </FieldGroup>
          </div>

          <Separator orientation="vertical" />

          <div className="flex flex-col">
            <DialogTitle>Open Existing Project</DialogTitle>
            <FieldGroup className="flex-1">
              {scanning ? (
                <FieldDescription>Scanning for Ralph projects...</FieldDescription>
              ) : (
                <>
                  <div className="flex-1">
                    <Field>
                      <FieldLabel>Discovered Projects ({projects.length})</FieldLabel>
                      <Select value={selectedProject} onValueChange={setSelectedProject}>
                        <SelectTrigger className="w-full">
                          <SelectValue placeholder="-- Select a project --" />
                        </SelectTrigger>
                        <SelectContent>
                          {projects.map(project => (
                            <SelectItem key={project.path} value={project.path}>
                              {project.name}
                            </SelectItem>
                          ))}
                        </SelectContent>
                      </Select>
                    </Field>
                  </div>

                  <InlineError error={scanError} />
                  <InlineError error={openError} onDismiss={() => setOpenError(null)} />
                  <Button onClick={handleOpenProject} disabled={!selectedProject} className="w-full">
                    Open Project
                  </Button>
                </>
              )}
            </FieldGroup>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  )
}
