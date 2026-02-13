import { describe, expect, it } from 'vitest'
import type { Task } from '@/types/generated'
import { computeProjectProgress, getAllTags } from './stats'

function makeTask(id: number, status: Task['status'], tags: string[]): Task {
  return {
    id,
    subsystem: 'auth',
    discipline: 'backend',
    title: `task-${id}`,
    status,
    tags,
    dependsOn: [],
    acceptanceCriteria: [],
    contextFiles: [],
    outputArtifacts: [],
    signals: [],
    subsystemDisplayName: 'Auth',
    subsystemAcronym: 'AUTH',
    disciplineDisplayName: 'Backend',
    disciplineAcronym: 'BKND',
    disciplineIcon: 'Server',
    disciplineColor: '#123456'
  }
}

describe('stats helpers', () => {
  it('ignores undefined task entries when collecting tags', () => {
    const tasks = [makeTask(1, 'pending', ['a', 'b']), undefined, makeTask(2, 'done', ['b', 'c'])] as Task[]
    expect(getAllTags(tasks)).toEqual(['a', 'b', 'c'])
  })

  it('ignores malformed task entries with missing tags', () => {
    const malformed = { ...makeTask(99, 'pending', ['x']), tags: undefined } as unknown as Task
    const tasks = [makeTask(1, 'pending', ['a']), malformed, makeTask(2, 'done', ['b'])] as Task[]
    expect(getAllTags(tasks)).toEqual(['a', 'b'])
  })

  it('ignores undefined task entries when computing project progress', () => {
    const tasks = [makeTask(1, 'done', []), undefined, makeTask(2, 'pending', [])] as Task[]
    expect(computeProjectProgress(tasks)).toEqual({
      totalTasks: 2,
      doneTasks: 1,
      progressPercent: 50
    })
  })
})
