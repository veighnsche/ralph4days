import { ChevronDown, ChevronRight, MessageSquare, Pencil, Plus, Trash2 } from 'lucide-react'
import { useState } from 'react'
import { InlineError } from '@/components/shared'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import { useFeatureCommentMutations } from '@/hooks/features'
import { formatDate } from '@/lib/formatDate'
import type { FeatureCommentData, FeatureData } from '@/types/generated'

const CATEGORY_SUGGESTIONS = ['design-decision', 'convention', 'gotcha', 'architecture', 'boundary', 'dependency']

function CategoryGroup({
  category,
  comments,
  mutations
}: {
  category: string
  comments: FeatureCommentData[]
  mutations: ReturnType<typeof useFeatureCommentMutations>
}) {
  const [collapsed, setCollapsed] = useState(false)

  return (
    <div>
      <button
        type="button"
        className="flex items-center gap-1.5 w-full text-left py-1"
        onClick={() => setCollapsed(!collapsed)}>
        {collapsed ? (
          <ChevronRight className="h-3.5 w-3.5 text-muted-foreground" />
        ) : (
          <ChevronDown className="h-3.5 w-3.5 text-muted-foreground" />
        )}
        <Badge variant="secondary" className="text-xs px-1.5 py-0 h-5">
          {category}
        </Badge>
        <span className="text-xs text-muted-foreground">({comments.length})</span>
      </button>
      {!collapsed && (
        <div className="ml-5 space-y-2 mt-1">
          {comments.map(comment => (
            <div key={comment.id} className="group/comment relative rounded-md px-2 py-1.5 hover:bg-muted/50">
              {mutations.editingId === comment.id ? (
                <div className="space-y-2">
                  <Textarea
                    value={mutations.editBody}
                    onChange={e => mutations.setEditBody(e.target.value)}
                    className="min-h-[48px] text-sm resize-none"
                    autoFocus
                    onKeyDown={e => {
                      if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) {
                        e.preventDefault()
                        mutations.submitEdit()
                      }
                      if (e.key === 'Escape') mutations.cancelEdit()
                    }}
                  />
                  <Input
                    value={mutations.editReason}
                    onChange={e => mutations.setEditReason(e.target.value)}
                    placeholder="Reason (optional)"
                    className="text-sm"
                  />
                  <div className="flex gap-1.5">
                    <Button size="sm" className="h-6 px-2 text-xs" onClick={mutations.submitEdit}>
                      Save
                    </Button>
                    <Button variant="ghost" size="sm" className="h-6 px-2 text-xs" onClick={mutations.cancelEdit}>
                      Cancel
                    </Button>
                  </div>
                </div>
              ) : (
                <>
                  <p className="text-sm leading-relaxed whitespace-pre-wrap">{comment.body}</p>
                  {comment.reason && <p className="text-xs text-muted-foreground mt-1 italic">why: {comment.reason}</p>}
                  <div className="flex items-center gap-2 mt-1 text-xs text-muted-foreground">
                    <span>{comment.author}</span>
                    {comment.created && <span>{formatDate(comment.created)}</span>}
                    <div className="ml-auto flex gap-0.5 opacity-0 group-hover/comment:opacity-100 transition-opacity">
                      <Button
                        variant="ghost"
                        size="sm"
                        className="h-5 w-5 p-0"
                        onClick={() => mutations.startEdit(comment.id, comment.body, comment.reason ?? '')}>
                        <Pencil className="h-3 w-3 text-muted-foreground" />
                      </Button>
                      <Button
                        variant="ghost"
                        size="sm"
                        className="h-5 w-5 p-0"
                        onClick={() => mutations.deleteComment(comment.id)}>
                        <Trash2 className="h-3 w-3 text-muted-foreground" />
                      </Button>
                    </div>
                  </div>
                </>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  )
}

function AddCommentForm({
  featureName,
  mutations
}: {
  featureName: string
  mutations: ReturnType<typeof useFeatureCommentMutations>
}) {
  const [expanded, setExpanded] = useState(false)
  const [category, setCategory] = useState('')
  const [body, setBody] = useState('')
  const [reason, setReason] = useState('')
  const [showSuggestions, setShowSuggestions] = useState(false)

  const filteredSuggestions = CATEGORY_SUGGESTIONS.filter(
    s => s.includes(category.toLowerCase()) && s !== category.toLowerCase()
  )

  const handleSubmit = () => {
    if (!(category.trim() && body.trim())) return
    mutations.addComment.mutate(
      {
        featureName,
        category: category.trim(),
        author: 'human',
        body: body.trim(),
        reason: reason.trim() || undefined
      },
      {
        onSuccess: () => {
          setCategory('')
          setBody('')
          setReason('')
          setExpanded(false)
        }
      }
    )
  }

  if (!expanded) {
    return (
      <Button variant="ghost" size="sm" className="h-7 px-2 text-xs gap-1.5" onClick={() => setExpanded(true)}>
        <Plus className="h-3 w-3" />
        Add comment
      </Button>
    )
  }

  return (
    <div className="border rounded-md p-3 space-y-2">
      <div className="relative">
        <Input
          value={category}
          onChange={e => {
            setCategory(e.target.value)
            setShowSuggestions(true)
          }}
          onFocus={() => setShowSuggestions(true)}
          onBlur={() => setTimeout(() => setShowSuggestions(false), 150)}
          placeholder="Category (e.g., design-decision, gotcha)"
          className="text-sm"
          autoFocus
        />
        {showSuggestions && filteredSuggestions.length > 0 && (
          <div className="absolute z-10 top-full left-0 right-0 mt-1 border rounded-md bg-popover shadow-md py-1">
            {filteredSuggestions.map(suggestion => (
              <button
                key={suggestion}
                type="button"
                className="w-full text-left px-3 py-1.5 text-sm hover:bg-muted"
                onMouseDown={e => {
                  e.preventDefault()
                  setCategory(suggestion)
                  setShowSuggestions(false)
                }}>
                {suggestion}
              </button>
            ))}
          </div>
        )}
      </div>
      <Textarea
        value={body}
        onChange={e => setBody(e.target.value)}
        placeholder="What should agents know?"
        className="min-h-[64px] text-sm resize-none"
        onKeyDown={e => {
          if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) {
            e.preventDefault()
            handleSubmit()
          }
          if (e.key === 'Escape') setExpanded(false)
        }}
      />
      <Input
        value={reason}
        onChange={e => setReason(e.target.value)}
        placeholder="Why? (optional)"
        className="text-sm"
      />
      <div className="flex gap-1.5">
        <Button
          size="sm"
          className="h-6 px-2 text-xs"
          onClick={handleSubmit}
          disabled={!(category.trim() && body.trim())}>
          Add
        </Button>
        <Button variant="ghost" size="sm" className="h-6 px-2 text-xs" onClick={() => setExpanded(false)}>
          Cancel
        </Button>
      </div>
    </div>
  )
}

export function FeatureCommentsSection({ feature }: { feature: FeatureData }) {
  const comments = feature.comments ?? []
  const mutations = useFeatureCommentMutations(feature.name)

  const grouped = new Map<string, FeatureCommentData[]>()
  for (const c of comments) {
    const existing = grouped.get(c.category)
    if (existing) {
      existing.push(c)
    } else {
      grouped.set(c.category, [c])
    }
  }

  return (
    <div className="px-6 space-y-3">
      <div className="flex items-center gap-1.5">
        <MessageSquare className="h-3.5 w-3.5 text-muted-foreground" />
        <span className="text-sm font-medium text-muted-foreground">
          Knowledge{comments.length > 0 && ` (${comments.length})`}
        </span>
      </div>

      <InlineError error={mutations.error} onDismiss={mutations.resetError} className="mb-3" />

      {grouped.size > 0 && (
        <div className="space-y-2">
          {[...grouped.entries()].map(([category, categoryComments]) => (
            <CategoryGroup key={category} category={category} comments={categoryComments} mutations={mutations} />
          ))}
        </div>
      )}

      <AddCommentForm featureName={feature.name} mutations={mutations} />
    </div>
  )
}
