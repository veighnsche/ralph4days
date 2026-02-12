import { defineWorkspaceTabModule } from '../contracts'
import { TaskDetailTabContent } from './content'
import { createTaskDetailTab } from './factory'
import { parseTaskDetailTabParams } from './schema'

export const taskDetailTabModule = defineWorkspaceTabModule({
  type: 'task-detail',
  component: TaskDetailTabContent,
  parseParams: parseTaskDetailTabParams,
  createTab: createTaskDetailTab
})
