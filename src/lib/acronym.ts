// WHY: Mirrors backend logic in crates/yaml-db/src/acronym.rs
export function generateAcronym(name: string, displayName?: string): string {
  const source = displayName?.includes(' ') ? displayName : name
  const cleaned = source.toUpperCase().replace(/-/g, ' ').replace(/_/g, ' ')
  const words = cleaned.split(/\s+/).filter(w => w.length > 0)

  if (words.length === 0) return 'UNKN'

  if (words.length === 1) {
    const word = words[0]
    if (word.length >= 4) {
      const consonants = word
        .split('')
        .filter(c => !'AEIOU'.includes(c))
        .slice(0, 4)
        .join('')

      if (consonants.length >= 4) {
        return consonants
      }
      return word.slice(0, 4)
    }
    const lastChar = word[word.length - 1]
    return (word + lastChar.repeat(4 - word.length)).toUpperCase()
  }

  if (words.length === 2) {
    return (words[0].slice(0, 2) + words[1].slice(0, 2)).toUpperCase()
  }

  if (words.length === 3) {
    return (words[0].slice(0, 2) + words[1][0] + words[2][0]).toUpperCase()
  }

  return words
    .slice(0, 4)
    .map(w => w[0])
    .join('')
    .toUpperCase()
}

export function normalizeFeatureName(name: string): string {
  return name.toLowerCase().trim().replace(/\s+/g, '-').replace(/_+/g, '-')
}
