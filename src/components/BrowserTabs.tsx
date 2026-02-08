import type { LucideIcon } from 'lucide-react'
import { X } from 'lucide-react'
import { useRef, useState } from 'react'
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
  ContextMenuTrigger
} from '@/components/ui/context-menu'
import type { BrowserTabsActions } from '@/hooks/useBrowserTabsActions'
import { cn } from '@/lib/utils'

export interface BrowserTab {
  id: string
  title: string
  icon?: LucideIcon
  closeable?: boolean
}

interface BrowserTabsProps {
  tabs: BrowserTab[]
  activeTabId: string
  actions: BrowserTabsActions
  newTabButton?: React.ReactNode
  className?: string
}

// WHY: Close buttons are mouse-only (tabIndex -1); Delete key for keyboard users (desktop convention)
export function BrowserTabs({ tabs, activeTabId, actions, newTabButton, className }: BrowserTabsProps) {
  const tabRefs = useRef<Map<string, HTMLButtonElement>>(new Map())
  const [draggedIndex, setDraggedIndex] = useState<number | null>(null)
  const [dragOverIndex, setDragOverIndex] = useState<number | null>(null)

  const focusTab = (id: string) => {
    tabRefs.current.get(id)?.focus()
    actions.switchTab(id)
  }

  const handleKeyDown = (e: React.KeyboardEvent, tabId: string) => {
    const idx = tabs.findIndex(t => t.id === tabId)
    if (idx === -1) return

    let handled = true

    switch (e.key) {
      case 'ArrowRight':
      case 'ArrowDown': {
        const next = tabs[(idx + 1) % tabs.length]
        focusTab(next.id)
        break
      }
      case 'ArrowLeft':
      case 'ArrowUp': {
        const prev = tabs[(idx - 1 + tabs.length) % tabs.length]
        focusTab(prev.id)
        break
      }
      case 'Home':
        focusTab(tabs[0].id)
        break
      case 'End':
        focusTab(tabs[tabs.length - 1].id)
        break
      case 'Delete': {
        const tab = tabs[idx]
        if (tab.closeable !== false) {
          actions.closeTab(tabId)
        }
        break
      }
      default:
        handled = false
    }

    if (handled) {
      e.preventDefault()
      e.stopPropagation()
    }
  }

  const handleDragStart = (e: React.DragEvent, index: number) => {
    setDraggedIndex(index)
    e.dataTransfer.effectAllowed = 'move'
  }

  const handleDragOver = (e: React.DragEvent, index: number) => {
    e.preventDefault()
    e.dataTransfer.dropEffect = 'move'
    setDragOverIndex(index)
  }

  const handleDragEnd = () => {
    if (draggedIndex !== null && dragOverIndex !== null && draggedIndex !== dragOverIndex) {
      actions.reorderTabs(draggedIndex, dragOverIndex)
    }
    setDraggedIndex(null)
    setDragOverIndex(null)
  }

  return (
    <div className={cn('flex items-end bg-muted/50 border-b border-border', className)}>
      <div
        role="tablist"
        aria-label="Workspace tabs"
        className="flex items-end gap-px min-w-0 flex-1 overflow-x-auto px-1 pt-1 [scrollbar-width:none] [&::-webkit-scrollbar]:hidden">
        {tabs.map((tab, index) => {
          const isActive = tab.id === activeTabId
          const TabIcon = tab.icon
          const isDragging = draggedIndex === index
          const isDragOver = dragOverIndex === index

          const tabElement = (
            <div
              key={tab.id}
              role="tab"
              draggable
              onDragStart={e => handleDragStart(e, index)}
              onDragOver={e => handleDragOver(e, index)}
              onDragEnd={handleDragEnd}
              ref={el => {
                if (el) tabRefs.current.set(tab.id, el as unknown as HTMLButtonElement)
                else tabRefs.current.delete(tab.id)
              }}
              id={`tab-${tab.id}`}
              aria-selected={isActive}
              aria-controls={`tabpanel-${tab.id}`}
              tabIndex={isActive ? 0 : -1}
              onClick={() => actions.switchTab(tab.id)}
              onKeyDown={e => handleKeyDown(e, tab.id)}
              className={cn(
                'group/tab flex items-center gap-1.5 h-8 min-w-0 max-w-[220px]',
                'pl-3 text-xs font-medium',
                'rounded-t-md',
                'outline-none transition-colors duration-100',
                'select-none cursor-default',
                'focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-inset',
                tab.closeable !== false ? 'pr-1' : 'pr-3',
                isActive && ['-mb-px bg-background text-foreground', 'border border-border border-b-background'],
                !isActive && [
                  'text-muted-foreground',
                  'border border-transparent',
                  'hover:text-foreground hover:bg-accent/50'
                ],
                isDragging && 'opacity-50',
                isDragOver && 'ring-2 ring-ring ring-inset'
              )}>
              {TabIcon && <TabIcon className="h-3.5 w-3.5 shrink-0" aria-hidden="true" />}
              <span className="truncate">{tab.title}</span>

              {tab.closeable !== false && (
                <button
                  type="button"
                  tabIndex={-1}
                  aria-label={`Close ${tab.title}`}
                  onClick={e => {
                    e.stopPropagation()
                    actions.closeTab(tab.id)
                  }}
                  className="h-5 w-5 rounded-sm shrink-0 inline-flex items-center justify-center transition-colors duration-100 text-muted-foreground hover:text-foreground hover:bg-muted">
                  <X className="h-3 w-3" aria-hidden="true" />
                </button>
              )}
            </div>
          )

          const hasTabsToRight = index < tabs.length - 1
          const hasOtherTabs = tabs.length > 1

          return (
            <ContextMenu key={tab.id}>
              <ContextMenuTrigger asChild>{tabElement}</ContextMenuTrigger>
              <ContextMenuContent>
                <ContextMenuItem onClick={() => actions.newTabToRight(tab.id)}>New Tab to the Right</ContextMenuItem>
                <ContextMenuSeparator />
                {tab.closeable !== false && (
                  <ContextMenuItem onClick={() => actions.closeTab(tab.id)}>Close</ContextMenuItem>
                )}
                {hasOtherTabs && (
                  <ContextMenuItem onClick={() => actions.closeOthers(tab.id)}>Close Others</ContextMenuItem>
                )}
                {hasTabsToRight && (
                  <ContextMenuItem onClick={() => actions.closeToRight(tab.id)}>
                    Close Tabs to the Right
                  </ContextMenuItem>
                )}
                {tabs.length > 0 && (
                  <>
                    <ContextMenuSeparator />
                    <ContextMenuItem variant="destructive" onClick={() => actions.closeAll()}>
                      Close All Tabs
                    </ContextMenuItem>
                  </>
                )}
              </ContextMenuContent>
            </ContextMenu>
          )
        })}
      </div>

      {newTabButton && <div className="flex items-center shrink-0 px-1 pb-px">{newTabButton}</div>}
    </div>
  )
}
