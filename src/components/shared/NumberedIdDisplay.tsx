interface NumberedIdDisplayProps {
  id: number
  variant?: 'inline' | 'block'
}

export function NumberedIdDisplay({ id, variant = 'inline' }: NumberedIdDisplayProps) {
  const formattedId = id > 999 ? id.toString() : `#${id.toString().padStart(3, '0')}`

  if (variant === 'block') {
    return (
      <span className="block w-12 text-sm text-muted-foreground font-mono font-semibold text-center tracking-widest">
        {formattedId}
      </span>
    )
  }

  return <span className="text-xs text-muted-foreground font-mono">{formattedId}</span>
}
