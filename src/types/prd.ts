export type TaskStatus = 'pending' | 'in_progress' | 'done' | 'blocked' | 'skipped'

export type InferredTaskStatus = 'ready' | 'waiting_on_deps' | 'externally_blocked' | 'in_progress' | 'done' | 'skipped'

export type TaskProvenance = 'agent' | 'human' | 'system'

export interface McpServerConfig {
  name: string
  command: string
  args?: string[]
  env?: Record<string, string>
}

export type CommentAuthor = 'human' | 'agent'

export interface TaskComment {
  id: number
  author: CommentAuthor
  agent_task_id?: number // WHY: snake_case matches Rust serde output (no rename_all on TaskComment struct)
  body: string
  created?: string
}

export interface Task {
  id: number
  feature: string
  discipline: string
  title: string
  description?: string
  status: TaskStatus
  inferredStatus: InferredTaskStatus
  priority?: 'low' | 'medium' | 'high' | 'critical'
  tags?: string[]
  dependsOn?: number[]
  blockedBy?: string
  created?: string
  updated?: string
  completed?: string
  acceptanceCriteria?: string[]
  contextFiles?: string[]
  outputArtifacts?: string[]
  hints?: string
  estimatedTurns?: number
  provenance?: TaskProvenance
  comments?: TaskComment[]
  featureDisplayName: string
  featureAcronym: string
  disciplineDisplayName: string
  disciplineAcronym: string
  disciplineIcon: string
  disciplineColor: string
}

export interface GroupStats {
  name: string
  displayName: string
  total: number
  done: number
  pending: number
  inProgress: number
  blocked: number
  skipped: number
}

export interface ProjectProgress {
  totalTasks: number
  doneTasks: number
  progressPercent: number
}

export interface ProjectInfo {
  title: string
  description?: string
  created?: string
}

export type StatusFilter =
  | 'all'
  | 'pending'
  | 'in_progress'
  | 'blocked'
  | 'done'
  | 'skipped'
  | 'ready'
  | 'waiting_on_deps'
export type PriorityFilter = 'all' | 'low' | 'medium' | 'high' | 'critical'

export type LearningSource = 'auto' | 'agent' | 'human' | 'opus_reviewed'

export interface FeatureLearning {
  text: string
  reason?: string
  source: LearningSource
  taskId?: number
  iteration?: number
  created: string
  hitCount: number
  reviewed: boolean
  reviewCount: number
}

export interface Feature {
  name: string
  displayName: string
  acronym?: string
  description?: string
  created?: string
  knowledgePaths?: string[]
  contextFiles?: string[]
  architecture?: string
  boundaries?: string
  learnings?: FeatureLearning[]
  dependencies?: string[]
}
