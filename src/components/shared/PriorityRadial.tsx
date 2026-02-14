import { PRIORITY_CONFIG } from '@/constants/prd'

export function PriorityRadial({ priority }: { priority?: string | null }) {
  const config = PRIORITY_CONFIG[priority as keyof typeof PRIORITY_CONFIG]
  if (!config) return null

  return (
    <div
      className="absolute top-0 right-0 w-32 h-32 pointer-events-none"
      style={{
        background: `radial-gradient(circle at top right, ${config.bgColor} 0%, transparent 70%)`
      }}
    />
  )
}
