const ACCEPTANCE_CRITERION_CHECKBOX_PATTERN = /^\[( |x|X)\]\s+(.*)$/

export interface ParsedAcceptanceCriterion {
  checked: boolean
  text: string
  hasExplicitCheckbox: boolean
}

export function parseAcceptanceCriterion(criterion: string): ParsedAcceptanceCriterion {
  const match = criterion.match(ACCEPTANCE_CRITERION_CHECKBOX_PATTERN)
  if (!match) {
    return { checked: false, text: criterion, hasExplicitCheckbox: false }
  }

  return {
    checked: match[1].toLowerCase() === 'x',
    text: match[2],
    hasExplicitCheckbox: true
  }
}

function formatAcceptanceCriterion(parsed: ParsedAcceptanceCriterion, nextText: string): string {
  const text = nextText.trim()
  if (text.length === 0) {
    throw new Error('[acceptance-criteria] Criterion text cannot be empty')
  }

  if (parsed.hasExplicitCheckbox || parsed.checked) {
    return `[${parsed.checked ? 'x' : ' '}] ${text}`
  }

  return text
}

export function toggleAcceptanceCriterion(criteria: string[], criterionIndex: number): string[] {
  if (criterionIndex < 0 || criterionIndex >= criteria.length) {
    throw new Error(`[acceptance-criteria] Invalid criterion index ${criterionIndex}`)
  }

  const current = parseAcceptanceCriterion(criteria[criterionIndex])
  const nextCriteria = [...criteria]
  nextCriteria[criterionIndex] = `[${current.checked ? ' ' : 'x'}] ${current.text.trim()}`
  return nextCriteria
}

export function updateAcceptanceCriterionText(criteria: string[], criterionIndex: number, nextText: string): string[] {
  if (criterionIndex < 0 || criterionIndex >= criteria.length) {
    throw new Error(`[acceptance-criteria] Invalid criterion index ${criterionIndex}`)
  }

  const current = parseAcceptanceCriterion(criteria[criterionIndex])
  const nextCriteria = [...criteria]
  nextCriteria[criterionIndex] = formatAcceptanceCriterion(current, nextText)
  return nextCriteria
}

export function addAcceptanceCriterion(criteria: string[], nextText = ''): string[] {
  const text = nextText.trim()
  return [`[ ] ${text}`, ...criteria]
}
