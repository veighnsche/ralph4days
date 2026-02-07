import { closestCenter, DndContext, KeyboardSensor, PointerSensor, useSensor, useSensors } from '@dnd-kit/core'
import {
  SortableContext,
  sortableKeyboardCoordinates,
  useSortable,
  verticalListSortingStrategy
} from '@dnd-kit/sortable'
import { CSS } from '@dnd-kit/utilities'
import { ChevronDown, ChevronUp, ClipboardCopy, GripVertical, Save, Trash2 } from 'lucide-react'
import { useEffect, useRef, useState } from 'react'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { ResizableHandle, ResizablePanel, ResizablePanelGroup } from '@/components/ui/resizable'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Separator } from '@/components/ui/separator'
import { Switch } from '@/components/ui/switch'
import { Textarea } from '@/components/ui/textarea'
import { usePromptPreview } from '@/hooks/usePromptPreview'
import { useRecipeManagement } from '@/hooks/useRecipeManagement'
import { type SectionBlock, useSectionConfiguration } from '@/hooks/useSectionConfiguration'

const BUILT_IN_RECIPES = [
  { value: 'braindump', label: 'Braindump' },
  { value: 'yap', label: 'Yap' },
  { value: 'ramble', label: 'Ramble' },
  { value: 'discuss', label: 'Discuss' },
  { value: 'task_execution', label: 'Task Execution' },
  { value: 'opus_review', label: 'Opus Review' }
] as const

const CATEGORY_COLORS: Record<string, string> = {
  project: 'bg-blue-500/15 text-blue-700 dark:text-blue-400',
  feature: 'bg-violet-500/15 text-violet-700 dark:text-violet-400',
  task: 'bg-amber-500/15 text-amber-700 dark:text-amber-400',
  discipline: 'bg-emerald-500/15 text-emerald-700 dark:text-emerald-400',
  state: 'bg-slate-500/15 text-slate-700 dark:text-slate-400',
  user: 'bg-rose-500/15 text-rose-700 dark:text-rose-400',
  instructions: 'bg-orange-500/15 text-orange-700 dark:text-orange-400'
}

interface PromptBuilderModalProps {
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function PromptBuilderModal({ open, onOpenChange }: PromptBuilderModalProps) {
  const {
    sections,
    sectionMeta,
    enabledCount,
    loadRecipeSections,
    loadCustomSections,
    handleDragEnd,
    toggleSection,
    commitInstructionOverride
  } = useSectionConfiguration(open)

  const {
    recipeName,
    customRecipeNames,
    currentPickerValue,
    saveDialogOpen,
    setSaveDialogOpen,
    saveNameInput,
    setSaveNameInput,
    handleRecipeChange,
    handleSave,
    doSave,
    handleDelete
  } = useRecipeManagement(open, sectionMeta, sections, loadRecipeSections, loadCustomSections)

  const { preview, handleUserInputChange, handleCopy } = usePromptPreview(open, sections)

  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 5 } }),
    useSensor(KeyboardSensor, { coordinateGetter: sortableKeyboardCoordinates })
  )

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-[95vw] h-[90vh] p-0 flex flex-col !max-w-[95vw]">
        <DialogHeader className="px-4 pt-3 pb-0">
          <div className="flex items-center gap-3">
            <div className="flex-1 min-w-0">
              <DialogTitle className="text-sm">Prompt Recipe Editor</DialogTitle>
              <DialogDescription className="text-xs">
                Compose sections, reorder, override instructions, save as custom recipes.
              </DialogDescription>
            </div>
            <Select value={currentPickerValue} onValueChange={handleRecipeChange}>
              <SelectTrigger className="w-[200px] h-8">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {BUILT_IN_RECIPES.map(r => (
                  <SelectItem key={r.value} value={r.value}>
                    {r.label}
                  </SelectItem>
                ))}
                {customRecipeNames.length > 0 && <Separator className="my-1" />}
                {customRecipeNames.map(name => (
                  <SelectItem key={`custom-${name}`} value={name}>
                    {name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        </DialogHeader>

        <Separator />

        <ResizablePanelGroup orientation="horizontal" className="flex-1 min-h-0">
          <ResizablePanel defaultSize={40} minSize={25}>
            <ScrollArea className="h-full">
              <div className="p-3 space-y-1.5">
                <DebouncedUserInput onDebouncedChange={handleUserInputChange} />

                <Separator />

                <p className="text-[10px] font-medium text-muted-foreground uppercase tracking-wider px-1 pt-1">
                  Sections
                </p>

                <DndContext sensors={sensors} collisionDetection={closestCenter} onDragEnd={handleDragEnd}>
                  <SortableContext items={sections.map(s => s.name)} strategy={verticalListSortingStrategy}>
                    {sections.map(section => (
                      <SortableSectionBlock
                        key={section.name}
                        section={section}
                        onToggle={() => toggleSection(section.name)}
                        onInstructionCommit={text => commitInstructionOverride(section.name, text)}
                      />
                    ))}
                  </SortableContext>
                </DndContext>
              </div>
            </ScrollArea>
          </ResizablePanel>

          <ResizableHandle withHandle />

          <ResizablePanel defaultSize={60} minSize={35}>
            <ScrollArea className="h-full">
              <div className="p-3 space-y-2">
                {preview?.sections.map(section => (
                  <div key={section.name} className="rounded-md border">
                    <div className="bg-muted/50 px-3 py-1 border-b">
                      <span className="text-[11px] font-medium text-muted-foreground">{section.name}</span>
                    </div>
                    <pre className="p-3 text-xs font-mono whitespace-pre-wrap break-words leading-relaxed">
                      {section.content}
                    </pre>
                  </div>
                ))}
                {!preview && <p className="text-sm text-muted-foreground text-center py-8">Loading preview...</p>}
              </div>
            </ScrollArea>
          </ResizablePanel>
        </ResizablePanelGroup>

        <Separator />

        <DialogFooter className="px-4 pb-2.5 pt-1.5">
          <Badge variant="outline" className="text-[10px] mr-auto">
            {enabledCount} / {sections.length} sections
          </Badge>

          {recipeName && (
            <Button variant="outline" size="default" onClick={handleDelete}>
              <Trash2 className="size-3.5" />
              Delete Recipe
            </Button>
          )}

          <Button variant="outline" size="default" onClick={handleCopy} disabled={!preview?.fullPrompt}>
            <ClipboardCopy className="size-3.5" />
            Copy Full Prompt
          </Button>

          <Button size="default" onClick={handleSave}>
            <Save className="size-3.5" />
            {recipeName ? 'Save' : 'Save As...'}
          </Button>
        </DialogFooter>
      </DialogContent>

      <Dialog open={saveDialogOpen} onOpenChange={setSaveDialogOpen}>
        <DialogContent className="max-w-sm">
          <DialogHeader>
            <DialogTitle className="text-sm">Save Recipe</DialogTitle>
            <DialogDescription className="text-xs">Enter a name for this custom recipe.</DialogDescription>
          </DialogHeader>
          <Input
            value={saveNameInput}
            onChange={e => setSaveNameInput(e.target.value)}
            placeholder="my-custom-recipe"
            className="h-8"
            onKeyDown={e => {
              if (e.key === 'Enter' && saveNameInput.trim()) {
                doSave(saveNameInput.trim())
              }
            }}
          />
          <DialogFooter>
            <Button variant="outline" size="default" onClick={() => setSaveDialogOpen(false)}>
              Cancel
            </Button>
            <Button size="default" disabled={!saveNameInput.trim()} onClick={() => doSave(saveNameInput.trim())}>
              Save
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </Dialog>
  )
}

function DebouncedUserInput({ onDebouncedChange }: { onDebouncedChange: (value: string) => void }) {
  const [value, setValue] = useState('')
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null)

  useEffect(() => {
    return () => {
      if (debounceRef.current) clearTimeout(debounceRef.current)
    }
  }, [])

  const handleChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    const newValue = e.target.value
    setValue(newValue)
    if (debounceRef.current) clearTimeout(debounceRef.current)
    debounceRef.current = setTimeout(() => onDebouncedChange(newValue), 300)
  }

  return (
    <div className="space-y-1.5 mb-3">
      <p className="text-[10px] font-medium text-muted-foreground uppercase tracking-wider px-1">
        User Input (preview only)
      </p>
      <Textarea
        value={value}
        onChange={handleChange}
        placeholder="Simulated user input..."
        className="min-h-[60px] font-mono text-xs"
      />
    </div>
  )
}

function SortableSectionBlock({
  section,
  onToggle,
  onInstructionCommit
}: {
  section: SectionBlock
  onToggle: () => void
  onInstructionCommit: (text: string | null) => void
}) {
  const { attributes, listeners, setNodeRef, setActivatorNodeRef, transform, transition, isDragging } = useSortable({
    id: section.name
  })

  const style = {
    transform: CSS.Transform.toString(transform),
    transition
  }

  const [instructionOpen, setInstructionOpen] = useState(!!section.instructionOverride)
  const [localInstruction, setLocalInstruction] = useState(section.instructionOverride ?? '')
  const [prevOverride, setPrevOverride] = useState(section.instructionOverride)
  const categoryColor = CATEGORY_COLORS[section.category] ?? ''

  if (section.instructionOverride !== prevOverride) {
    setPrevOverride(section.instructionOverride)
    setLocalInstruction(section.instructionOverride ?? '')
  }

  const handleBlur = () => {
    const committed = localInstruction || null
    if (committed !== section.instructionOverride) {
      onInstructionCommit(committed)
    }
  }

  const handleReset = () => {
    setLocalInstruction('')
    onInstructionCommit(null)
  }

  return (
    <div
      ref={setNodeRef}
      style={style}
      className={`rounded-md border transition-opacity duration-100 ${section.enabled ? 'opacity-100' : 'opacity-50'} ${isDragging ? 'z-50 shadow-md bg-background' : ''}`}>
      <div className="flex items-center gap-2 px-2.5 py-1.5">
        <button
          type="button"
          ref={setActivatorNodeRef}
          {...attributes}
          {...listeners}
          className="cursor-grab active:cursor-grabbing touch-none text-muted-foreground hover:text-foreground transition-colors">
          <GripVertical className="size-3.5" />
        </button>

        <Switch checked={section.enabled} onCheckedChange={onToggle} className="scale-75" />

        <Badge variant="secondary" className={`text-[9px] px-1.5 py-0 font-normal ${categoryColor}`}>
          {section.category}
        </Badge>

        <div className="flex-1 min-w-0">
          <p className="text-xs font-medium truncate">{section.displayName}</p>
          <p className="text-[10px] text-muted-foreground truncate">{section.description}</p>
        </div>
      </div>

      {section.isInstruction && section.enabled && (
        <Collapsible open={instructionOpen} onOpenChange={setInstructionOpen}>
          <CollapsibleTrigger asChild>
            <button
              type="button"
              className="w-full text-left px-2.5 py-1 border-t text-[10px] text-muted-foreground hover:bg-muted/30 transition-colors flex items-center gap-1">
              {instructionOpen ? <ChevronUp className="size-2.5" /> : <ChevronDown className="size-2.5" />}
              {localInstruction ? 'Custom instructions' : 'Edit instructions'}
            </button>
          </CollapsibleTrigger>
          <CollapsibleContent>
            <div className="px-2.5 pb-2 pt-1 border-t space-y-1">
              <Textarea
                value={localInstruction}
                onChange={e => setLocalInstruction(e.target.value)}
                onBlur={handleBlur}
                placeholder="Leave empty to use default instructions..."
                className="min-h-[120px] font-mono text-[11px] leading-relaxed"
              />
              {localInstruction && (
                <Button variant="ghost" size="sm" className="h-6 text-[10px]" onClick={handleReset}>
                  Reset to default
                </Button>
              )}
            </div>
          </CollapsibleContent>
        </Collapsible>
      )}
    </div>
  )
}
