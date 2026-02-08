import { cn } from '@/lib/utils'

type Token = { type: 'text' | 'heading' | 'bold' | 'code' | 'bullet' | 'checkbox' | 'numbered'; value: string }

function tokenizeLine(line: string): Token[] {
  const headingMatch = line.match(/^(#{1,4})\s+(.*)$/)
  if (headingMatch) return [{ type: 'heading', value: line }]

  const bulletMatch = line.match(/^(\s*-\s)(\[[ x]\]\s)?(.*)$/)
  if (bulletMatch) {
    const tokens: Token[] = [{ type: 'bullet', value: bulletMatch[1] }]
    if (bulletMatch[2]) tokens.push({ type: 'checkbox', value: bulletMatch[2] })
    tokens.push(...tokenizeInline(bulletMatch[3]))
    return tokens
  }

  const numberedMatch = line.match(/^(\s*\d+\.\s)(.*)$/)
  if (numberedMatch) {
    return [{ type: 'numbered', value: numberedMatch[1] }, ...tokenizeInline(numberedMatch[2])]
  }

  return tokenizeInline(line)
}

function tokenizeInline(text: string): Token[] {
  const tokens: Token[] = []
  const re = /(\*\*[^*]+\*\*|`[^`]+`)/g
  let last = 0
  for (const match of text.matchAll(re)) {
    const idx = match.index ?? 0
    if (idx > last) tokens.push({ type: 'text', value: text.slice(last, idx) })
    if (match[0].startsWith('**')) tokens.push({ type: 'bold', value: match[0] })
    else tokens.push({ type: 'code', value: match[0] })
    last = idx + match[0].length
  }
  if (last < text.length) tokens.push({ type: 'text', value: text.slice(last) })
  return tokens
}

const TOKEN_CLASSES: Record<Token['type'], string> = {
  heading: 'text-blue-600 dark:text-blue-400 font-semibold',
  bold: 'text-foreground font-semibold',
  code: 'text-pink-600 dark:text-pink-400 bg-pink-500/10 rounded px-0.5',
  bullet: 'text-emerald-600 dark:text-emerald-400 font-bold',
  checkbox: 'text-amber-600 dark:text-amber-400',
  numbered: 'text-emerald-600 dark:text-emerald-400 font-bold',
  text: ''
}

export function HighlightedPrompt({ text, className }: { text: string; className?: string }) {
  const lines = text.split('\n')
  return (
    <pre
      className={cn(
        'border-input dark:bg-input/30 w-full rounded-md border bg-transparent px-3 py-2 text-xs font-mono whitespace-pre-wrap break-words leading-relaxed shadow-xs',
        className
      )}>
      {lines.map((line, i) => (
        // biome-ignore lint/suspicious/noArrayIndexKey: using index is correct here since lines array is stable within each render
        <div key={i}>
          {tokenizeLine(line).map((tok, j) => (
            // biome-ignore lint/suspicious/noArrayIndexKey: using index is correct here since tokens array is stable within each render
            <span key={j} className={TOKEN_CLASSES[tok.type]}>
              {tok.value}
            </span>
          ))}
        </div>
      ))}
    </pre>
  )
}
