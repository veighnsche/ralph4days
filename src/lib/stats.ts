import type { GroupStats, ProjectProgress, SubsystemData, Task } from '@/types/generated'

interface GroupItem {
  name: string
  displayName: string
}

function isTaskRecord(task: unknown): task is Task {
  return Boolean(task && typeof task === 'object')
}

function getTaskTags(task: Task): string[] {
  return Array.isArray(task.tags) ? task.tags : []
}

export function computeSubsystemStats(tasks: Task[], subsystems: SubsystemData[]): Map<string, GroupStats> {
  const statsMap = new Map<string, GroupStats>()

  for (const subsystem of subsystems) {
    statsMap.set(subsystem.name, {
      name: subsystem.name,
      displayName: subsystem.displayName,
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
    if (!isTaskRecord(task)) continue
    const stats = statsMap.get(task.subsystem)
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
    if (!isTaskRecord(task)) continue
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
  const actionableTasks = tasks.filter(
    t => isTaskRecord(t) && typeof t.status === 'string' && t.status !== 'draft' && t.status !== 'skipped'
  )
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
    if (!isTaskRecord(task)) continue
    for (const tag of getTaskTags(task)) {
      tagsSet.add(tag)
    }
  }

  return Array.from(tagsSet).sort()
}
