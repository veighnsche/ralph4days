import { Ban, BookOpen, CheckCircle2, CircleDashed, CircleHelp, CircleX, Flag, Lightbulb } from 'lucide-react'

export const VERB_CONFIG = {
  done: { label: 'Done', icon: CheckCircle2, color: 'var(--status-done)' },
  partial: { label: 'Partial', icon: CircleDashed, color: 'var(--priority-medium)' },
  stuck: { label: 'Stuck', icon: CircleX, color: 'var(--status-blocked)' },
  ask: { label: 'Ask', icon: CircleHelp, color: 'var(--secondary)' },
  flag: { label: 'Flag', icon: Flag, color: 'var(--priority-medium)' },
  learned: { label: 'Learned', icon: BookOpen, color: 'var(--primary)' },
  suggest: { label: 'Suggest', icon: Lightbulb, color: 'var(--primary)' },
  blocked: { label: 'Blocked', icon: Ban, color: 'var(--status-blocked)' }
} as const

export const FLAG_SEVERITY_CONFIG = {
  info: { label: 'Info', color: 'var(--muted-foreground)' },
  warning: { label: 'Warning', color: 'var(--priority-medium)' },
  blocking: { label: 'Blocking', color: 'var(--status-blocked)' }
} as const

export type SignalVerb = keyof typeof VERB_CONFIG
export type FlagSeverity = keyof typeof FLAG_SEVERITY_CONFIG
