import { zodResolver } from '@hookform/resolvers/zod'
import { ClipboardList } from 'lucide-react'
import { useForm } from 'react-hook-form'
import { toast } from 'sonner'
import { TaskFormFields } from '@/components/forms/TaskForm'
import { useInvokeMutation } from '@/hooks/api'
import { type TaskFormData, taskSchema } from '@/lib/schemas'
import type { WorkspaceTab } from '@/stores/useWorkspaceStore'
import { useWorkspaceStore } from '@/stores/useWorkspaceStore'
import { EntityFormTabContent } from './EntityFormTabContent'

export function TaskFormTabContent({ tab }: { tab: WorkspaceTab }) {
  const closeTab = useWorkspaceStore(s => s.closeTab)

  const form = useForm<TaskFormData>({
    resolver: zodResolver(taskSchema),
    defaultValues: {
      feature: '',
      discipline: '',
      title: '',
      description: '',
      priority: 'medium',
      tags: [],
      dependsOn: [],
      acceptanceCriteria: []
    }
  })

  const createTask = useInvokeMutation<Record<string, unknown>>('create_task', {
    invalidateKeys: [['get_tasks']],
    onSuccess: () => {
      toast.success('Task created')
      closeTab(tab.id)
    }
  })

  const handleSubmit = (data: TaskFormData) => {
    createTask.mutate({
      feature: data.feature,
      discipline: data.discipline,
      title: data.title,
      description: data.description || null,
      priority: data.priority || null,
      tags: data.tags,
      dependsOn: data.dependsOn.length > 0 ? data.dependsOn : null,
      acceptanceCriteria: data.acceptanceCriteria.length > 0 ? data.acceptanceCriteria : null
    })
  }

  return (
    <EntityFormTabContent
      tab={tab}
      icon={ClipboardList}
      entityName="Task"
      form={form}
      onSubmit={handleSubmit}
      isPending={createTask.isPending}
      error={createTask.error}
      onErrorDismiss={createTask.reset}>
      <TaskFormFields disabled={createTask.isPending} />
    </EntityFormTabContent>
  )
}
