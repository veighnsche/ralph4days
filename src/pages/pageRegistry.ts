import type { ComponentType } from 'react'
import { DisciplinesPage } from './DisciplinesPage'
import { SubsystemsPage } from './SubsystemsPage'
import { TasksPage } from './TasksPage'

export type Page = 'tasks' | 'subsystems' | 'disciplines'

export const pageRegistry: Record<string, ComponentType> = {
  tasks: TasksPage,
  subsystems: SubsystemsPage,
  disciplines: DisciplinesPage
}
