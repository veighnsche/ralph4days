import { zodResolver } from '@hookform/resolvers/zod'
import { Layers } from 'lucide-react'
import { useForm } from 'react-hook-form'
import { toast } from 'sonner'
import { DisciplineFormFields } from '@/components/forms/DisciplineForm'
import { useDisciplineMutations } from '@/hooks/useDisciplineMutations'
import { type DisciplineFormData, disciplineSchema } from '@/lib/schemas'
import type { WorkspaceTab } from '@/stores/useWorkspaceStore'
import { useWorkspaceStore } from '@/stores/useWorkspaceStore'
import { EntityFormTabContent } from './EntityFormTabContent'

export function DisciplineFormTabContent({ tab }: { tab: WorkspaceTab }) {
  const closeTab = useWorkspaceStore(s => s.closeTab)
  const { createDiscipline, isCreating, createError } = useDisciplineMutations()

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

  const handleSubmit = async (data: DisciplineFormData) => {
    try {
      await createDiscipline(data)
      toast.success('Discipline created')
      closeTab(tab.id)
    } catch (err) {
      toast.error(String(err))
    }
  }

  return (
    <EntityFormTabContent
      tab={tab}
      icon={Layers}
      entityName="Discipline"
      form={form}
      onSubmit={handleSubmit}
      isPending={isCreating}
      error={createError}
      onErrorDismiss={() => {}}>
      <DisciplineFormFields disabled={isCreating} />
    </EntityFormTabContent>
  )
}
