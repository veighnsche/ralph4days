import type { Task } from '@/types/generated'

export function getTaskDisplayId(task: Task): string {
  return `${task.feature}/${task.discipline}/${task.id}`
}

export function getTaskDiscipline(task: Task): Task['discipline'] {
  return task.discipline
}
