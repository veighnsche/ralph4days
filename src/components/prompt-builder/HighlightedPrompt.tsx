import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import { cn } from '@/lib/utils'

export function HighlightedPrompt({ text, className }: { text: string; className?: string }) {
  return (
    <div
      className={cn(
        'border-input dark:bg-input/30 w-full rounded-md border bg-transparent px-3 py-2 text-xs font-mono leading-relaxed shadow-xs',
        className
      )}>
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        components={{
          h1: ({ children }) => <h1 className="text-blue-600 dark:text-blue-400 font-semibold mb-2">{children}</h1>,
          h2: ({ children }) => <h2 className="text-blue-600 dark:text-blue-400 font-semibold mb-2">{children}</h2>,
          h3: ({ children }) => <h3 className="text-blue-600 dark:text-blue-400 font-semibold mb-1">{children}</h3>,
          h4: ({ children }) => <h4 className="text-blue-600 dark:text-blue-400 font-semibold mb-1">{children}</h4>,
          p: ({ children }) => <p className="whitespace-pre-wrap break-words mb-2">{children}</p>,
          ul: ({ children }) => (
            <ul className="list-disc ml-4 mb-2 marker:text-emerald-600 dark:marker:text-emerald-400">{children}</ul>
          ),
          ol: ({ children }) => (
            <ol className="list-decimal ml-4 mb-2 marker:text-emerald-600 dark:marker:text-emerald-400">{children}</ol>
          ),
          li: ({ children }) => <li className="mb-1">{children}</li>,
          strong: ({ children }) => <strong className="text-foreground font-semibold">{children}</strong>,
          code: ({ children, className: markdownClassName }) => (
            <code
              className={cn(
                'text-pink-600 dark:text-pink-400 bg-pink-500/10 rounded px-0.5 whitespace-pre-wrap break-words',
                markdownClassName
              )}>
              {children}
            </code>
          ),
          pre: ({ children }) => <pre className="bg-transparent p-0 m-0">{children}</pre>,
          input: ({ checked }) => (
            <input className="mr-1 accent-amber-600 dark:accent-amber-400" type="checkbox" disabled checked={checked} />
          )
        }}>
        {text}
      </ReactMarkdown>
    </div>
  )
}
