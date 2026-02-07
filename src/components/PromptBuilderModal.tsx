import { closestCenter, DndContext, KeyboardSensor, PointerSensor, useSensor, useSensors } from '@dnd-kit/core'
import {
  SortableContext,
  sortableKeyboardCoordinates,
  useSortable,
  verticalListSortingStrategy
} from '@dnd-kit/sortable'
import { CSS } from '@dnd-kit/utilities'
import { ClipboardCopy, GripVertical, Save, Trash2, X } from 'lucide-react'
import { useEffect, useRef, useState } from 'react'
import { HighlightedPrompt } from '@/components/HighlightedPrompt'
import { InlineError } from '@/components/InlineError'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
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
import { BUILT_IN_RECIPES, CATEGORY_COLORS, CATEGORY_GRADIENT_COLORS } from '@/lib/recipe-registry'

interface PromptBuilderModalProps {
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function PromptBuilderModal({ open, onOpenChange }: PromptBuilderModalProps) {
  const {
    sections,
    enabledCount,
    loadRecipeSections,
    loadCustomSections,
    handleDragEnd,
    toggleSection,
    commitInstructionOverride,
    loadError: sectionLoadError,
    resetLoadError: resetSectionLoadError
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
    handleDelete,
    error: recipeError,
    resetError: resetRecipeError
  } = useRecipeManagement(open, sections, loadRecipeSections, loadCustomSections)

  const { preview, handleUserInputChange, handleCopy, previewError, resetPreviewError } = usePromptPreview(
    open,
    sections
  )

  const [selectedSection, setSelectedSection] = useState<string | null>(null)
  const selectedBlock = sections.find(s => s.name === selectedSection)

  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 5 } }),
    useSensor(KeyboardSensor, { coordinateGetter: sortableKeyboardCoordinates })
  )

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent
        className="!max-w-none w-screen h-screen p-0 flex flex-col gap-0 rounded-none border-0"
        onPointerDownOutside={e => e.preventDefault()}
        showCloseButton={false}>
        <DialogHeader className="px-4 pt-3 pb-3">
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
          <ResizablePanel defaultSize={33} minSize={18}>
            <ScrollArea className="h-full">
              <div className="p-3 space-y-1.5">
                <InlineError error={sectionLoadError} onDismiss={resetSectionLoadError} />
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
                        selected={section.name === selectedSection}
                        onSelect={() => setSelectedSection(section.name === selectedSection ? null : section.name)}
                        onToggle={() => toggleSection(section.name)}
                      />
                    ))}
                  </SortableContext>
                </DndContext>
              </div>
            </ScrollArea>
          </ResizablePanel>

          <ResizableHandle withHandle />

          <ResizablePanel defaultSize={34} minSize={18}>
            <SectionSettingsPanel
              section={selectedBlock ?? null}
              onInstructionCommit={text => {
                if (selectedSection) commitInstructionOverride(selectedSection, text)
              }}
            />
          </ResizablePanel>

          <ResizableHandle withHandle />

          <ResizablePanel defaultSize={33} minSize={18}>
            <div className="h-full p-3 flex flex-col gap-2">
              <InlineError error={previewError} onDismiss={resetPreviewError} />
              {preview?.fullPrompt ? (
                <HighlightedPrompt text={preview.fullPrompt} className="flex-1 min-h-0 overflow-y-auto" />
              ) : (
                <p className="text-sm text-muted-foreground text-center py-8">Loading preview...</p>
              )}
            </div>
          </ResizablePanel>
        </ResizablePanelGroup>

        <Separator />

        <DialogFooter className="px-4 pb-2.5 pt-1.5">
          <InlineError error={recipeError} onDismiss={resetRecipeError} className="w-full" />
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

          <Button variant="outline" size="default" onClick={() => onOpenChange(false)}>
            <X className="size-3.5" />
            Close
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
  selected,
  onSelect,
  onToggle
}: {
  section: SectionBlock
  selected: boolean
  onSelect: () => void
  onToggle: () => void
}) {
  const { attributes, listeners, setNodeRef, setActivatorNodeRef, transform, transition, isDragging } = useSortable({
    id: section.name
  })

  const style = {
    transform: CSS.Transform.toString(transform),
    transition
  }

  const categoryColor = CATEGORY_COLORS[section.category] ?? ''
  const gradientColor = CATEGORY_GRADIENT_COLORS[section.category] ?? 'rgba(100, 116, 139, 0.12)'

  return (
    <button
      ref={setNodeRef}
      style={style}
      type="button"
      className={`w-full rounded-md border transition-all duration-100 cursor-pointer relative overflow-hidden ${section.enabled ? 'opacity-100' : 'opacity-50'} ${isDragging ? 'z-50 shadow-md bg-background' : ''} ${selected ? 'ring-2 ring-ring' : 'hover:bg-muted/30'}`}
      onClick={onSelect}>
      <div
        className="absolute top-0 right-0 w-32 h-32 pointer-events-none"
        style={{
          background: `radial-gradient(circle at top right, ${gradientColor} 0%, transparent 70%)`
        }}
      />

      <div className="flex items-center gap-2 px-2.5 py-1.5 relative">
        <button
          type="button"
          ref={setActivatorNodeRef}
          {...attributes}
          {...listeners}
          className="cursor-grab active:cursor-grabbing touch-none text-muted-foreground hover:text-foreground transition-colors flex-shrink-0"
          onClick={e => e.stopPropagation()}>
          <GripVertical className="size-3.5" />
        </button>

        <button
          type="button"
          onClick={e => e.stopPropagation()}
          onKeyDown={e => {
            if (e.key === 'Enter' || e.key === ' ') e.stopPropagation()
          }}
          className="flex-shrink-0">
          <Switch checked={section.enabled} onCheckedChange={onToggle} className="scale-75" />
        </button>

        <div className="flex-1 min-w-0 text-left">
          <p className="text-xs font-medium truncate">{section.displayName}</p>
          <p className="text-[10px] text-muted-foreground truncate">{section.description}</p>
        </div>

        <Badge variant="secondary" className={`text-[9px] px-1.5 py-0.5 font-normal flex-shrink-0 ${categoryColor}`}>
          {section.category}
        </Badge>
      </div>
    </button>
  )
}

function SectionSettingsPanel({
  section,
  onInstructionCommit
}: {
  section: SectionBlock | null
  onInstructionCommit: (text: string | null) => void
}) {
  const [localInstruction, setLocalInstruction] = useState('')
  const [trackedSection, setTrackedSection] = useState<string | null>(null)

  const sectionName = section?.name ?? null
  if (sectionName !== trackedSection) {
    setTrackedSection(sectionName)
    setLocalInstruction(section?.instructionOverride ?? '')
  }

  const handleBlur = () => {
    const committed = localInstruction || null
    if (committed !== (section?.instructionOverride ?? null)) {
      onInstructionCommit(committed)
    }
  }

  const handleReset = () => {
    setLocalInstruction('')
    onInstructionCommit(null)
  }

  if (!section) {
    return (
      <div className="h-full flex items-center justify-center p-4">
        <p className="text-sm text-muted-foreground">Select a section to edit</p>
      </div>
    )
  }

  if (!(section.isInstruction && section.enabled)) {
    return (
      <div className="h-full flex flex-col p-3 gap-3">
        <div>
          <p className="text-xs font-medium">{section.displayName}</p>
          <p className="text-[10px] text-muted-foreground">{section.description}</p>
        </div>
        <Separator />
        <p className="text-sm text-muted-foreground">
          {section.enabled ? 'No editable settings for this section.' : 'This section is disabled.'}
        </p>
      </div>
    )
  }

  return (
    <div className="h-full flex flex-col p-3 gap-3">
      <div>
        <p className="text-xs font-medium">{section.displayName}</p>
        <p className="text-[10px] text-muted-foreground">{section.description}</p>
      </div>
      <Separator />
      <p className="text-[10px] font-medium text-muted-foreground uppercase tracking-wider">Instruction Override</p>
      <Textarea
        value={localInstruction}
        onChange={e => setLocalInstruction(e.target.value)}
        onBlur={handleBlur}
        placeholder="Leave empty to use default instructions..."
        className="flex-1 font-mono text-[11px] leading-relaxed resize-none"
      />
      {localInstruction && (
        <Button variant="ghost" size="sm" className="h-6 text-[10px] self-start" onClick={handleReset}>
          Reset to default
        </Button>
      )}
    </div>
  )
}
