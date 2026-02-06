/**
 * ModelThinkingPicker - Reusable component for selecting Claude model and thinking mode
 *
 * Provides a split button with dropdown menu to select:
 * - Model: haiku, sonnet, opus
 * - Extended thinking: on/off
 *
 * Persists preferences to localStorage using shared keys across the app.
 */

import { ChevronDown } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuLabel,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip";
import { type Model, useModelThinkingPreferences } from "@/hooks/useModelThinkingPreferences";
import { cn } from "@/lib/utils";

export type { Model } from "@/hooks/useModelThinkingPreferences";

interface ModelThinkingPickerProps {
  /** Called when user clicks the primary action button */
  onAction: (model: Model, thinking: boolean) => void;
  /** Text for the primary action button */
  actionLabel: string;
  /** Optional icon element for the primary action button */
  actionIcon?: React.ReactNode;
  /** Whether the action button is disabled */
  disabled?: boolean;
  /** Button variant - defaults to "default" */
  variant?: "default" | "outline" | "ghost";
  /** Button size - defaults to "lg" */
  size?: "sm" | "default" | "lg";
  /** Additional className for the button container */
  className?: string;
}

export function ModelThinkingPicker({
  onAction,
  actionLabel,
  actionIcon,
  disabled = false,
  variant = "default",
  size = "default",
  className,
}: ModelThinkingPickerProps) {
  const { model, setModel, thinking, setThinking } = useModelThinkingPreferences();

  const handleAction = () => {
    onAction(model, thinking);
  };

  return (
    <TooltipProvider>
      <div className={cn("flex", className)}>
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              type="button"
              variant={variant}
              size={size}
              onClick={handleAction}
              disabled={disabled}
              className="rounded-r-none"
            >
              {actionIcon}
              {actionLabel}
            </Button>
          </TooltipTrigger>
          <TooltipContent>
            <div className="space-y-1">
              <div>
                <span className="font-semibold">Model:</span> {model.charAt(0).toUpperCase() + model.slice(1)}
              </div>
              <div>
                <span className="font-semibold">Thinking:</span> {thinking ? "On" : "Off"}
              </div>
            </div>
          </TooltipContent>
        </Tooltip>

        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button
              type="button"
              variant={variant}
              size={size}
              disabled={disabled}
              className="rounded-l-none border-l px-2"
            >
              <ChevronDown className="h-4 w-4" />
              <span className="sr-only">Model options</span>
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end" className="w-48">
            <DropdownMenuLabel>Model</DropdownMenuLabel>
            <DropdownMenuRadioGroup value={model} onValueChange={(v) => setModel(v as Model)}>
              <DropdownMenuRadioItem value="haiku">Haiku (fast)</DropdownMenuRadioItem>
              <DropdownMenuRadioItem value="sonnet">Sonnet (balanced)</DropdownMenuRadioItem>
              <DropdownMenuRadioItem value="opus">Opus (smart)</DropdownMenuRadioItem>
            </DropdownMenuRadioGroup>

            <DropdownMenuSeparator />

            <DropdownMenuLabel>Options</DropdownMenuLabel>
            <DropdownMenuCheckboxItem checked={thinking} onCheckedChange={setThinking}>
              Extended thinking
            </DropdownMenuCheckboxItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>
    </TooltipProvider>
  );
}
