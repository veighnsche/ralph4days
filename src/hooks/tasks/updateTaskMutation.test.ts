import { describe, expect, it } from 'vitest'
import type { Task } from '@/types/generated'
import { buildOptimisticTaskFromUpdateTask } from './updateTaskMutation'

function createTask(id: number, overrides: Partial<Task> = {}): Task {
  return {
    id,
    subsystem: 'core-platform',
    discipline: 'frontend',
    title: `Task ${id}`,
    status: 'pending',
    tags: [],
    dependsOn: [],
    acceptanceCriteria: [],
    contextFiles: [],
    outputArtifacts: [],
    signals: [],
    subsystemDisplayName: 'Core Platform',
    subsystemAcronym: 'CP',
    disciplineDisplayName: 'Frontend',
    disciplineAcronym: 'FE',
    disciplineIcon: 'bot',
    disciplineColor: '#43a047',
    ...overrides
  }
}

describe('buildOptimisticTaskFromUpdateTask', () => {
  it('maps update_task params into task fields', () => {
    const currentTask = createTask(12, {
      title: 'Before',
      acceptanceCriteria: ['[ ] old']
    })

    const nextTask = buildOptimisticTaskFromUpdateTask(currentTask, {
      id: 12,
      subsystem: 'core-platform',
      discipline: 'frontend',
      title: 'After',
      description: 'Updated description',
      priority: 'high',
      tags: ['ui'],
      depends_on: [1, 2],
      acceptance_criteria: ['[x] first'],
      context_files: ['src/a.ts'],
      output_artifacts: ['dist/a.js'],
      hints: 'Do this',
      estimated_turns: 3,
      provenance: 'agent',
      agent: 'codex',
      model: 'gpt-5.3-codex',
      effort: 'medium',
      thinking: true
    })

    expect(nextTask).toEqual(
      createTask(12, {
        title: 'After',
        description: 'Updated description',
        priority: 'high',
        tags: ['ui'],
        dependsOn: [1, 2],
        acceptanceCriteria: ['[x] first'],
        contextFiles: ['src/a.ts'],
        outputArtifacts: ['dist/a.js'],
        hints: 'Do this',
        estimatedTurns: 3,
        provenance: 'agent',
        agent: 'codex',
        model: 'gpt-5.3-codex',
        effort: 'medium',
        thinking: true
      })
    )
  })

  it('fails loudly when ids do not match', () => {
    expect(() =>
      buildOptimisticTaskFromUpdateTask(createTask(7), {
        id: 8,
        subsystem: 'core-platform',
        discipline: 'frontend',
        title: 'Mismatch',
        tags: [],
        depends_on: [],
        acceptance_criteria: [],
        context_files: [],
        output_artifacts: []
      })
    ).toThrowError('[task-cache] Cannot build optimistic task for id 8; current task is 7')
  })
})
