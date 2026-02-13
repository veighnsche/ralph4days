import { CheckCircle2, Send, X } from 'lucide-react'
import { useEffect, useRef, useState } from 'react'
import { STATUS_CONFIG } from '@/constants/prd'
import { cn } from '@/lib/utils'
import { parseAcceptanceCriterion } from './acceptanceCriteria'

const SINGLE_CLICK_DELAY_MS = 180

interface AcceptanceCriterionItemProps {
  criterion: string
  criterionIndex: number
  isTaskDone: boolean
  isPending: boolean
  onToggle: (criterionIndex: number) => void
  onSaveText: (criterionIndex: number, nextText: string) => void
}

export function AcceptanceCriterionItem({
  criterion,
  criterionIndex,
  isTaskDone,
  isPending,
  onToggle,
  onSaveText
}: AcceptanceCriterionItemProps) {
  const parsedCriterion = parseAcceptanceCriterion(criterion)
  const [isEditing, setIsEditing] = useState(false)
  const [draftText, setDraftText] = useState(parsedCriterion.text)
  const singleClickTimeoutRef = useRef<number | null>(null)
  const isChecked = isTaskDone || parsedCriterion.checked
  const interactionDisabled = isTaskDone || isPending
  const canSave = draftText.trim().length > 0 && !isPending

  useEffect(() => {
    if (!isEditing) {
      setDraftText(parsedCriterion.text)
    }
  }, [isEditing, parsedCriterion.text])

  useEffect(
    () => () => {
      if (singleClickTimeoutRef.current !== null) {
        window.clearTimeout(singleClickTimeoutRef.current)
      }
    },
    []
  )

  const clearPendingSingleClick = () => {
    if (singleClickTimeoutRef.current !== null) {
      window.clearTimeout(singleClickTimeoutRef.current)
      singleClickTimeoutRef.current = null
    }
  }

  const queueSingleClickToggle = () => {
    clearPendingSingleClick()
    singleClickTimeoutRef.current = window.setTimeout(() => {
      onToggle(criterionIndex)
      singleClickTimeoutRef.current = null
    }, SINGLE_CLICK_DELAY_MS)
  }

  const handleTextSingleClick = () => {
    if (interactionDisabled || isEditing) return
    queueSingleClickToggle()
  }

  const handleTextDoubleClick = () => {
    if (interactionDisabled || isEditing) return
    clearPendingSingleClick()
    setDraftText(parsedCriterion.text)
    setIsEditing(true)
  }

  const handleCancelEdit = () => {
    setDraftText(parsedCriterion.text)
    setIsEditing(false)
  }

  const handleSaveEdit = () => {
    if (!canSave) return
    onSaveText(criterionIndex, draftText.trim())
    setIsEditing(false)
  }

  return (
    <li className="flex items-start gap-2.5 text-sm">
      <button
        type="button"
        onClick={() => onToggle(criterionIndex)}
        aria-label={`Toggle acceptance criterion ${(criterionIndex + 1).toString()}`}
        disabled={interactionDisabled}
        className="mt-1 w-4 h-4 rounded-sm border flex items-center justify-center flex-shrink-0 disabled:cursor-not-allowed"
        style={{
          borderColor: isChecked ? STATUS_CONFIG.done.color : 'hsl(var(--border))',
          backgroundColor: isChecked ? STATUS_CONFIG.done.bgColor : 'transparent'
        }}>
        {isChecked && <CheckCircle2 className="w-3 h-3" style={{ color: STATUS_CONFIG.done.color }} />}
      </button>

      <div className="flex-1 min-w-0">
        {isEditing ? (
          <div className="flex items-center gap-1.5 border-b border-border/60 pb-0.5">
            <input
              value={draftText}
              onChange={event => setDraftText(event.target.value)}
              onKeyDown={event => {
                if (event.key === 'Enter') {
                  event.preventDefault()
                  handleSaveEdit()
                }
                if (event.key === 'Escape') {
                  event.preventDefault()
                  handleCancelEdit()
                }
              }}
              className={cn(
                'flex-1 bg-transparent border-0 p-0 text-sm leading-relaxed focus:outline-none',
                isChecked ? 'line-through text-muted-foreground' : ''
              )}
            />
            <button
              type="button"
              onClick={handleCancelEdit}
              aria-label={`Cancel acceptance criterion ${(criterionIndex + 1).toString()} edit`}
              className="h-5 w-5 inline-flex items-center justify-center text-muted-foreground hover:text-foreground">
              <X className="h-3 w-3" />
            </button>
            <button
              type="button"
              onClick={handleSaveEdit}
              aria-label={`Save acceptance criterion ${(criterionIndex + 1).toString()} edit`}
              disabled={!canSave}
              className="h-5 w-5 inline-flex items-center justify-center text-muted-foreground hover:text-foreground disabled:opacity-50 disabled:cursor-not-allowed">
              <Send className="h-3 w-3" />
            </button>
          </div>
        ) : (
          <input
            value={parsedCriterion.text}
            readOnly
            aria-readonly="true"
            onClick={handleTextSingleClick}
            onDoubleClick={handleTextDoubleClick}
            className={cn(
              'w-full bg-transparent border-0 p-0 text-sm leading-relaxed focus:outline-none',
              interactionDisabled ? 'cursor-not-allowed' : 'cursor-default',
              isChecked ? 'line-through text-muted-foreground' : ''
            )}
          />
        )}
      </div>
    </li>
  )
}
