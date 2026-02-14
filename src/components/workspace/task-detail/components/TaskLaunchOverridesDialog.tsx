import type { ReactNode } from 'react'
import { useState } from 'react'
import { InlineError } from '@/components/shared'
import { Button } from '@/components/ui/button'
import { Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle, DialogTrigger } from '@/components/ui/dialog'
import { useInvokeMutation } from '@/hooks/api'
import type { UpdateTaskVariables } from '@/hooks/tasks/updateTaskMutation'
import type { AgentSessionLaunchConfig } from '@/lib/agent-session-launch-config'
import type { Task } from '@/types/generated'
import { AgentSessionLaunchForm } from '../../tabs/agent-session-config/components/AgentSessionLaunchForm'
import { useAgentSessionConfigLaunchState } from '../../tabs/agent-session-config/hooks/useAgentSessionConfigTabState'
import { useModelConstraints } from '../../tabs/agent-session-config/hooks/useModelConstraints'
import { useModelFormTreeByAgent } from '../../tabs/agent-session-config/hooks/useModelFormTreeByAgent'
import { AgentSessionConfigStoreProvider } from '../../tabs/agent-session-config/store'

function TaskLaunchOverridesDialogBody({
  task,
  onCancel,
  onSaved
}: {
  task: Task
  onCancel: () => void
  onSaved: () => void
}) {
  const { formTreeByAgent, formTreeLoading, formTreeError } = useModelFormTreeByAgent()
  const { models, loadingModels, error, selectedModel, selectedModelEffortValid } = useModelConstraints({
    formTreeByAgent,
    formTreeLoading,
    formTreeError
  })
  const { agent, model, effort, thinking } = useAgentSessionConfigLaunchState()

  const updateTaskMutation = useInvokeMutation<UpdateTaskVariables, Task>('update_task', {
    queryDomain: 'workspace',
    invalidateKeys: [['get_task', { id: task.id }], ['get_task_list_items']],
    onSuccess: () => onSaved()
  })

  const supportsEffort = (selectedModel?.effortOptions?.length ?? 0) > 0
  const canSave =
    !loadingModels && !!model && models.length > 0 && selectedModelEffortValid && !updateTaskMutation.isPending

  const handleSave = () => {
    // Fail fast if model selection is invalid; this should be prevented by UI constraints.
    if (!selectedModelEffortValid) {
      throw new Error('Invariant violation: attempted to save with invalid effort selection for model')
    }

    updateTaskMutation.mutate({
      params: {
        id: task.id,
        subsystem: task.subsystem,
        discipline: task.discipline,
        title: task.title,
        description: task.description,
        priority: task.priority,
        tags: task.tags,
        depends_on: task.dependsOn ?? [],
        acceptance_criteria: task.acceptanceCriteria,
        context_files: task.contextFiles,
        output_artifacts: task.outputArtifacts,
        hints: task.hints,
        estimated_turns: task.estimatedTurns,
        provenance: task.provenance,
        agent,
        model,
        effort: supportsEffort ? effort : undefined,
        thinking
      }
    })
  }

  return (
    <div className="flex h-[min(78vh,720px)] w-[min(92vw,960px)] flex-col">
      <DialogHeader className="px-4 pt-3 pb-2">
        <DialogTitle className="text-sm">Launch Options</DialogTitle>
      </DialogHeader>

      <div className="flex-1 min-h-0">
        <AgentSessionLaunchForm
          layout="two_column"
          showHeader={false}
          models={models}
          loadingModels={loadingModels}
          error={error}
          footer={null}
        />
      </div>

      {updateTaskMutation.error ? (
        <div className="px-4 pb-2">
          <InlineError error={updateTaskMutation.error} onDismiss={updateTaskMutation.reset} />
        </div>
      ) : null}

      <DialogFooter className="px-4 pb-3 pt-2">
        <Button type="button" variant="outline" onClick={onCancel} disabled={updateTaskMutation.isPending}>
          Cancel
        </Button>
        <Button type="button" onClick={handleSave} disabled={!canSave}>
          Save
        </Button>
      </DialogFooter>
    </div>
  )
}

export function TaskLaunchOverridesDialog({
  task,
  trigger,
  initialConfig
}: {
  task: Task
  trigger: ReactNode
  initialConfig: AgentSessionLaunchConfig
}) {
  const [open, setOpen] = useState(false)

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>{trigger}</DialogTrigger>

      {open ? (
        <DialogContent className="p-0 max-w-none" showCloseButton>
          <AgentSessionConfigStoreProvider initialConfig={initialConfig}>
            <TaskLaunchOverridesDialogBody task={task} onCancel={() => setOpen(false)} onSaved={() => setOpen(false)} />
          </AgentSessionConfigStoreProvider>
        </DialogContent>
      ) : null}
    </Dialog>
  )
}
