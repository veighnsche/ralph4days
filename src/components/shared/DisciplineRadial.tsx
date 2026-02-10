import { useDisciplines } from '@/hooks/disciplines'

export function DisciplineRadial({ discipline }: { discipline?: string | null }) {
  const { disciplines } = useDisciplines()
  if (!discipline) return null

  const disc = disciplines.find(d => d.name === discipline)
  if (!disc?.color) return null

  return (
    <div
      className="absolute bottom-0 left-0 w-32 h-32 pointer-events-none"
      style={{
        background: `radial-gradient(circle at bottom left, ${disc.color}20 0%, transparent 70%)`
      }}
    />
  )
}
