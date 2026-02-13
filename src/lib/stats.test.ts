import { describe, expect, it } from 'vitest'
import type { TaskListItem } from '@/types/generated'
import { computeProjectProgress, getAllTags } from './stats'

function makeTask(id: number, status: TaskListItem['status'], tags: string[]): TaskListItem {
  return {
    id,
    subsystem: 'auth',
    discipline: 'backend',
    title: `task-${id}`,
    status,
    tags,
    dependsOn: [],
    acceptanceCriteriaCount: 0,
    signalCount: 0,
    subsystemDisplayName: 'Auth',
    subsystemAcronym: 'AUTH',
    disciplineDisplayName: 'Backend',
    disciplineAcronym: 'BKND',
    disciplineIcon: 'Server',
    disciplineColor: '#123456'
  }
}

describe('stats helpers', () => {
  it('collects unique tags across tasks and sorts them', () => {
    const tasks = [makeTask(1, 'pending', ['a', 'b']), makeTask(2, 'done', ['b', 'c'])]
    expect(getAllTags(tasks)).toEqual(['a', 'b', 'c'])
  })

  it('computes project progress from actionable tasks', () => {
    const tasks = [makeTask(1, 'draft', []), makeTask(2, 'done', []), makeTask(3, 'pending', [])]
    expect(computeProjectProgress(tasks)).toEqual({
      totalTasks: 2,
      doneTasks: 1,
      progressPercent: 50
    })
  })
})
