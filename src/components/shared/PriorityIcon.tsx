import { ArrowDown, ArrowUp, Equal } from 'lucide-react'
import { PRIORITY_CONFIG } from '@/constants/prd'

const ICONS = { high: ArrowUp, medium: Equal, low: ArrowDown } as const

const SIZES = {
  sm: 'h-3 w-3',
  md: 'h-4 w-4',
  lg: 'h-5 w-5'
} as const

export function PriorityIcon({ priority, size = 'sm' }: { priority: string; size?: 'sm' | 'md' | 'lg' }) {
  const config = PRIORITY_CONFIG[priority as keyof typeof PRIORITY_CONFIG]
  if (!config) return null

  const Icon = ICONS[priority as keyof typeof ICONS] ?? Equal
  return <Icon className={SIZES[size]} style={{ color: config.color }} />
}
