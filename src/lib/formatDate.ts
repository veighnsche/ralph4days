export function formatDate(value: unknown): string {
  if (typeof value === 'string') {
    const d = new Date(value)
    if (!Number.isNaN(d.getTime())) {
      return d.toLocaleDateString(undefined, { month: 'short', day: 'numeric', year: 'numeric' })
    }
    return value
  }
  if (value instanceof Date) {
    return value.toLocaleDateString(undefined, { month: 'short', day: 'numeric', year: 'numeric' })
  }
  return String(value)
}
