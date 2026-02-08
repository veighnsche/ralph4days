import { zodResolver } from '@hookform/resolvers/zod'
import { Layers } from 'lucide-react'
import { useForm } from 'react-hook-form'
import { toast } from 'sonner'
import { DisciplineFormFields } from '@/components/forms/DisciplineForm'
import { useInvokeMutation } from '@/hooks/useInvokeMutation'
import { type DisciplineFormData, disciplineSchema } from '@/lib/schemas'
import type { WorkspaceTab } from '@/stores/useWorkspaceStore'
import { useWorkspaceStore } from '@/stores/useWorkspaceStore'
import { EntityFormTabContent } from './EntityFormTabContent'

export function DisciplineFormTabContent({ tab }: { tab: WorkspaceTab }) {
  const closeTab = useWorkspaceStore(s => s.closeTab)

  const form = useForm<DisciplineFormData>({
    resolver: zodResolver(disciplineSchema),
    defaultValues: {
      name: '',
      displayName: '',
      acronym: '',
      icon: 'Code',
      color: '#3b82f6'
    }
  })

  const createDiscipline = useInvokeMutation<Record<string, unknown>>('create_discipline', {
    invalidateKeys: [['get_disciplines_config'], ['get_tasks']],
    onSuccess: () => {
      toast.success('Discipline created')
      closeTab(tab.id)
    }
  })

  const handleSubmit = (data: DisciplineFormData) => {
    createDiscipline.mutate({
      name: data.name || data.displayName,
      displayName: data.displayName,
      acronym: data.acronym,
      icon: data.icon,
      color: data.color
    })
  }

  return (
    <EntityFormTabContent
      tab={tab}
      icon={Layers}
      entityName="Discipline"
      form={form}
      onSubmit={handleSubmit}
      isPending={createDiscipline.isPending}
      error={createDiscipline.error}
      onErrorDismiss={createDiscipline.reset}>
      <DisciplineFormFields disabled={createDiscipline.isPending} />
    </EntityFormTabContent>
  )
}
