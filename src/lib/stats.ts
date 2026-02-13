import type { GroupStats, ProjectProgress, SubsystemData, Task } from '@/types/generated'

interface GroupItem {
  name: string
  displayName: string
}

export function computeSubsystemStats(tasks: Task[], subsystems: SubsystemData[]): Map<string, GroupStats> {
  const statsMap = new Map<string, GroupStats>()

  for (const feature of subsystems) {
    statsMap.set(feature.name, {
      name: feature.name,
      displayName: feature.displayName,
      total: 0,
      draft: 0,
      done: 0,
      pending: 0,
      inProgress: 0,
      blocked: 0,
      skipped: 0
    })
  }

  for (const task of tasks) {
    const stats = statsMap.get(task.feature)
    if (!stats) continue
    stats.total++
    switch (task.status) {
      case 'draft':
        stats.draft++
        break
      case 'done':
        stats.done++
        break
      case 'pending':
        stats.pending++
        break
      case 'in_progress':
        stats.inProgress++
        break
      case 'blocked':
        stats.blocked++
        break
      case 'skipped':
        stats.skipped++
        break
    }
  }

  return statsMap
}

export function computeDisciplineStats(tasks: Task[], disciplines: GroupItem[]): Map<string, GroupStats> {
  const statsMap = new Map<string, GroupStats>()

  for (const discipline of disciplines) {
    statsMap.set(discipline.name, {
      name: discipline.name,
      displayName: discipline.displayName,
      total: 0,
      draft: 0,
      done: 0,
      pending: 0,
      inProgress: 0,
      blocked: 0,
      skipped: 0
    })
  }

  for (const task of tasks) {
    const stats = statsMap.get(task.discipline)
    if (!stats) continue
    stats.total++
    switch (task.status) {
      case 'draft':
        stats.draft++
        break
      case 'done':
        stats.done++
        break
      case 'pending':
        stats.pending++
        break
      case 'in_progress':
        stats.inProgress++
        break
      case 'blocked':
        stats.blocked++
        break
      case 'skipped':
        stats.skipped++
        break
    }
  }

  return statsMap
}

export function computeProjectProgress(tasks: Task[]): ProjectProgress {
  const actionableTasks = tasks.filter(t => t.status !== 'draft' && t.status !== 'skipped')
  const totalTasks = actionableTasks.length
  let doneTasks = 0
  for (const task of actionableTasks) {
    if (task.status === 'done') doneTasks++
  }
  const progressPercent = totalTasks > 0 ? Math.floor((doneTasks * 100) / totalTasks) : 0

  return { totalTasks, doneTasks, progressPercent }
}

export function getAllTags(tasks: Task[]): string[] {
  const tagsSet = new Set<string>()

  for (const task of tasks) {
    for (const tag of task.tags) {
      tagsSet.add(tag)
    }
  }

  return Array.from(tagsSet).sort()
}
