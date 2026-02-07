import { X } from 'lucide-react'
import { useState } from 'react'
import { useFormContext } from 'react-hook-form'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { FormControl, FormField, FormItem, FormLabel, FormMessage } from '@/components/ui/form'
import { Input } from '@/components/ui/input'
import { NativeSelect } from '@/components/ui/native-select'
import { Textarea } from '@/components/ui/textarea'
import { useDisciplines } from '@/hooks/useDisciplines'
import { useInvoke } from '@/hooks/useInvoke'
import type { TaskFormData } from '@/lib/schemas'
import type { Feature } from '@/types/prd'

export function TaskFormFields({ disabled }: { disabled?: boolean }) {
  const { control, getValues, setValue } = useFormContext<TaskFormData>()
  const { disciplines } = useDisciplines()
  const { data: features = [] } = useInvoke<Feature[]>('get_features')

  const [newTag, setNewTag] = useState('')
  const [newCriterion, setNewCriterion] = useState('')

  const addTag = () => {
    const tag = newTag.trim()
    if (tag && !getValues('tags').includes(tag)) {
      setValue('tags', [...getValues('tags'), tag])
      setNewTag('')
    }
  }

  const removeTag = (tag: string) => {
    setValue(
      'tags',
      getValues('tags').filter(t => t !== tag)
    )
  }

  const addCriterion = () => {
    const criterion = newCriterion.trim()
    if (criterion) {
      setValue('acceptanceCriteria', [...getValues('acceptanceCriteria'), criterion])
      setNewCriterion('')
    }
  }

  const removeCriterion = (index: number) => {
    setValue(
      'acceptanceCriteria',
      getValues('acceptanceCriteria').filter((_, i) => i !== index)
    )
  }

  return (
    <div className="space-y-3">
      <FormField
        control={control}
        name="feature"
        render={({ field }) => (
          <FormItem>
            <FormLabel>
              Feature <span className="text-destructive">*</span>
            </FormLabel>
            <FormControl>
              {features.length > 0 ? (
                <NativeSelect {...field} required disabled={disabled}>
                  <option value="">Select a feature...</option>
                  {features.map(feature => (
                    <option key={feature.name} value={feature.name}>
                      {feature.displayName}
                    </option>
                  ))}
                </NativeSelect>
              ) : (
                <Input {...field} placeholder="Enter feature name" required disabled={disabled} />
              )}
            </FormControl>
            <FormMessage />
          </FormItem>
        )}
      />

      <FormField
        control={control}
        name="discipline"
        render={({ field }) => (
          <FormItem>
            <FormLabel>
              Discipline <span className="text-destructive">*</span>
            </FormLabel>
            <FormControl>
              <NativeSelect {...field} required disabled={disabled}>
                <option value="">Select a discipline...</option>
                {disciplines.map(discipline => (
                  <option key={discipline.name} value={discipline.name}>
                    {discipline.displayName}
                  </option>
                ))}
              </NativeSelect>
            </FormControl>
            <FormMessage />
          </FormItem>
        )}
      />

      <FormField
        control={control}
        name="title"
        render={({ field }) => (
          <FormItem>
            <FormLabel>
              Title <span className="text-destructive">*</span>
            </FormLabel>
            <FormControl>
              <Input {...field} placeholder="Enter task title" required disabled={disabled} />
            </FormControl>
            <FormMessage />
          </FormItem>
        )}
      />

      <FormField
        control={control}
        name="description"
        render={({ field }) => (
          <FormItem>
            <FormLabel>Description</FormLabel>
            <FormControl>
              <Textarea {...field} placeholder="Enter task description" rows={4} disabled={disabled} />
            </FormControl>
            <FormMessage />
          </FormItem>
        )}
      />

      <FormField
        control={control}
        name="priority"
        render={({ field }) => (
          <FormItem>
            <FormLabel>Priority</FormLabel>
            <FormControl>
              <NativeSelect {...field} disabled={disabled}>
                <option value="low">Low</option>
                <option value="medium">Medium</option>
                <option value="high">High</option>
                <option value="critical">Critical</option>
              </NativeSelect>
            </FormControl>
            <FormMessage />
          </FormItem>
        )}
      />

      <FormField
        control={control}
        name="tags"
        render={({ field }) => (
          <FormItem>
            <FormLabel>Tags</FormLabel>
            <div className="flex gap-2">
              <Input
                value={newTag}
                onChange={e => setNewTag(e.target.value)}
                placeholder="Add a tag"
                disabled={disabled}
                onKeyDown={e => {
                  if (e.key === 'Enter') {
                    e.preventDefault()
                    addTag()
                  }
                }}
              />
              <Button type="button" onClick={addTag} variant="outline" disabled={disabled}>
                Add
              </Button>
            </div>
            {field.value.length > 0 && (
              <div className="flex flex-wrap gap-2 mt-2">
                {field.value.map(tag => (
                  <Badge key={tag} variant="secondary" className="gap-1">
                    {tag}
                    <button
                      type="button"
                      onClick={() => removeTag(tag)}
                      className="hover:text-destructive"
                      disabled={disabled}>
                      <X className="h-3 w-3" />
                    </button>
                  </Badge>
                ))}
              </div>
            )}
            <FormMessage />
          </FormItem>
        )}
      />

      <FormField
        control={control}
        name="acceptanceCriteria"
        render={({ field }) => (
          <FormItem>
            <FormLabel>Acceptance Criteria</FormLabel>
            <div className="flex gap-2">
              <Input
                value={newCriterion}
                onChange={e => setNewCriterion(e.target.value)}
                placeholder="Add acceptance criterion"
                disabled={disabled}
                onKeyDown={e => {
                  if (e.key === 'Enter') {
                    e.preventDefault()
                    addCriterion()
                  }
                }}
              />
              <Button type="button" onClick={addCriterion} variant="outline" disabled={disabled}>
                Add
              </Button>
            </div>
            {field.value.length > 0 && (
              <ul className="space-y-1 mt-2">
                {field.value.map((criterion, index) => (
                  <li key={criterion} className="flex items-start gap-2 text-sm">
                    <span className="flex-1">{criterion}</span>
                    <button
                      type="button"
                      onClick={() => removeCriterion(index)}
                      className="text-muted-foreground hover:text-destructive"
                      disabled={disabled}>
                      <X className="h-4 w-4" />
                    </button>
                  </li>
                ))}
              </ul>
            )}
            <FormMessage />
          </FormItem>
        )}
      />
    </div>
  )
}
