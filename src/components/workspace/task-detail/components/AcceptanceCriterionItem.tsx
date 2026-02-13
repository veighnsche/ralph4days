import { Send, X } from 'lucide-react'
import { useEffect, useRef, useState } from 'react'
import { Checkbox } from '@/components/ui/checkbox'
import { STATUS_CONFIG } from '@/constants/prd'
import { cn } from '@/lib/utils'
import { parseAcceptanceCriterion } from './acceptanceCriteria'

const SINGLE_CLICK_DELAY_MS = 180
const EMPTY_CRITERION_PLACEHOLDER = 'New acceptance criterion'

interface AcceptanceCriterionItemProps {
  criterion: string
  criterionIndex: number
  isTaskDone: boolean
  isPending: boolean
  onToggle: (criterionIndex: number) => void
  onSaveText: (criterionIndex: number, nextText: string) => void
  autoStartEditing?: boolean
  onAutoStartEditConsumed?: () => void
}

export function AcceptanceCriterionItem({
  criterion,
  criterionIndex,
  isTaskDone,
  isPending,
  onToggle,
  onSaveText,
  autoStartEditing = false,
  onAutoStartEditConsumed
}: AcceptanceCriterionItemProps) {
  const parsedCriterion = parseAcceptanceCriterion(criterion)
  const [isEditing, setIsEditing] = useState(false)
  const [draftText, setDraftText] = useState(parsedCriterion.text)
  const singleClickTimeoutRef = useRef<number | null>(null)
  const editInputRef = useRef<HTMLInputElement | null>(null)
  const isChecked = isTaskDone || parsedCriterion.checked
  const toggleDisabled = isTaskDone
  const editInteractionDisabled = isTaskDone || isPending
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

  useEffect(() => {
    if (!isEditing) return
    editInputRef.current?.focus()
  }, [isEditing])

  const clearPendingSingleClick = () => {
    if (singleClickTimeoutRef.current !== null) {
      window.clearTimeout(singleClickTimeoutRef.current)
      singleClickTimeoutRef.current = null
    }
  }

  useEffect(() => {
    if (!autoStartEditing || editInteractionDisabled || isEditing) return

    if (singleClickTimeoutRef.current !== null) {
      window.clearTimeout(singleClickTimeoutRef.current)
      singleClickTimeoutRef.current = null
    }
    setDraftText(parsedCriterion.text)
    setIsEditing(true)
    onAutoStartEditConsumed?.()
  }, [autoStartEditing, editInteractionDisabled, isEditing, onAutoStartEditConsumed, parsedCriterion.text])

  const queueSingleClickToggle = () => {
    clearPendingSingleClick()
    singleClickTimeoutRef.current = window.setTimeout(() => {
      onToggle(criterionIndex)
      singleClickTimeoutRef.current = null
    }, SINGLE_CLICK_DELAY_MS)
  }

  const handleTextSingleClick = () => {
    if (toggleDisabled || isEditing) return
    queueSingleClickToggle()
  }

  const handleTextDoubleClick = () => {
    if (editInteractionDisabled || isEditing) return
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
      <Checkbox
        checked={isChecked}
        onCheckedChange={() => onToggle(criterionIndex)}
        aria-label={`Toggle acceptance criterion ${(criterionIndex + 1).toString()}`}
        disabled={toggleDisabled}
        className="mt-1 flex-shrink-0"
        style={{
          borderColor: isChecked ? STATUS_CONFIG.done.color : 'hsl(var(--border))',
          backgroundColor: isChecked ? STATUS_CONFIG.done.bgColor : 'transparent',
          color: STATUS_CONFIG.done.color
        }}
      />

      <div className="flex-1 min-w-0">
        {isEditing ? (
          <div className="flex items-center gap-1.5 border-b border-border/60 pb-0.5">
            <input
              ref={editInputRef}
              value={draftText}
              placeholder={EMPTY_CRITERION_PLACEHOLDER}
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
            placeholder={parsedCriterion.text.trim().length === 0 ? EMPTY_CRITERION_PLACEHOLDER : undefined}
            readOnly
            aria-readonly="true"
            onClick={handleTextSingleClick}
            onDoubleClick={handleTextDoubleClick}
            className={cn(
              'w-full bg-transparent border-0 p-0 text-sm leading-relaxed focus:outline-none',
              toggleDisabled ? 'cursor-not-allowed' : 'cursor-default',
              isChecked ? 'line-through text-muted-foreground' : ''
            )}
          />
        )}
      </div>
    </li>
  )
}
