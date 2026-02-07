import type { ComponentType } from 'react'
import { DisciplinesPage } from './DisciplinesPage'
import { FeaturesPage } from './FeaturesPage'
import { TasksPage } from './TasksPage'

export const pageRegistry: Record<string, ComponentType> = {
  tasks: TasksPage,
  features: FeaturesPage,
  disciplines: DisciplinesPage
}
