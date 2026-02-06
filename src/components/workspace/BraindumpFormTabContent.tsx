/**
 * BraindumpFormTabContent - Form for braindumping project ideas to Claude
 *
 * TODO LIST:
 * 1. Wire up MCP server generation (mcp_generator.rs)
 *    - Generate bash MCP server that exposes .ralph/db/ as tools
 *    - Pass mcp_config to create_pty_session instead of hardcoded "interactive"
 *    - Claude needs create_task, create_feature, create_discipline commands
 *
 * 2. Add context prompt wrapper
 *    - Prepend system prompt: "You are helping structure a project. Use the tools..."
 *    - Include instructions to create features/disciplines/tasks from braindump
 *    - Tell Claude to ask clarifying questions if needed
 *
 * 3. Listen for database changes
 *    - After Claude creates tasks, invalidate React Query cache
 *    - Auto-refresh tasks/features pages to show new items
 *    - Show toast notification when items are created
 *
 * 4. Handle terminal close gracefully
 *    - If user closes terminal before Claude finishes, show warning
 *    - Option to continue in background or cancel operation
 *
 * 5. Show progress indicator
 *    - Parse Claude's stream output to show "Creating tasks..." status
 *    - Count how many tasks/features were created
 *    - Show summary when complete
 */

import { invoke } from "@tauri-apps/api/core";
import { Brain } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { toast } from "sonner";
import { type Model, ModelThinkingPicker } from "@/components/ModelThinkingPicker";
import { Button } from "@/components/ui/button";
import { FormDescription, FormHeader, FormTitle } from "@/components/ui/form-header";
import { Label } from "@/components/ui/label";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import { Textarea } from "@/components/ui/textarea";
import { useTabMeta } from "@/hooks/useTabMeta";
import type { WorkspaceTab } from "@/stores/useWorkspaceStore";
import { useWorkspaceStore } from "@/stores/useWorkspaceStore";

const DEFAULT_QUESTIONS = `What is this project about?

What problem does it solve?

Who will use this?

What are the main features you're thinking about?

What's the tech stack (if you know)?

Any constraints or special requirements?

What's your timeline or priorities?`;

interface BraindumpFormTabContentProps {
  tab: WorkspaceTab;
}

export function BraindumpFormTabContent({ tab }: BraindumpFormTabContentProps) {
  useTabMeta(tab.id, "Braindump", Brain);
  const { closeTab, openTab, tabs } = useWorkspaceStore();
  const [braindump, setBraindump] = useState(DEFAULT_QUESTIONS);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const isMountedRef = useRef(true);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      isMountedRef.current = false;
    };
  }, []);

  const handleSubmit = async (model: Model, thinking: boolean) => {
    // Validate input
    const trimmedBraindump = braindump.trim();
    if (!trimmedBraindump) {
      toast.error("Please enter some text before sending");
      return;
    }

    if (isSubmitting) return; // Prevent double submission

    setIsSubmitting(true);

    try {
      // Open new terminal tab with selected model and thinking
      const terminalId = openTab({
        type: "terminal",
        title: `Claude (${model})`,
        closeable: true,
        data: {
          model,
          thinking,
        },
      });

      // Wait for terminal to initialize (increased timeout for reliability)
      await new Promise((resolve) => setTimeout(resolve, 1000));

      // Check if component is still mounted
      if (!isMountedRef.current) return;

      // Verify terminal tab still exists
      const terminalExists = tabs.some((t) => t.id === terminalId);
      if (!terminalExists) {
        throw new Error("Terminal tab was closed before sending");
      }

      // Send braindump to terminal with auto-send
      const message = `${trimmedBraindump}\n`; // Auto-send with newline
      const bytes = Array.from(new TextEncoder().encode(message));

      await invoke("send_terminal_input", { sessionId: terminalId, data: bytes });

      // Check if component is still mounted before closing
      if (!isMountedRef.current) return;

      // Close the braindump form
      closeTab(tab.id);

      toast.success("Braindump sent to Claude");
    } catch (err) {
      console.error("Failed to send braindump:", err);
      const errorMessage = err instanceof Error ? err.message : "Unknown error";
      toast.error(`Failed to send braindump: ${errorMessage}`);

      // Only reset state if component is still mounted
      if (isMountedRef.current) {
        setIsSubmitting(false);
      }
    }
  };

  const handleCancel = () => {
    closeTab(tab.id);
  };

  return (
    <div className="flex h-full flex-col">
      <ScrollArea className="flex-1">
        <form className="px-4 space-y-4">
          <FormHeader>
            <FormTitle>Braindump Your Project</FormTitle>
            <FormDescription>
              Answer these questions in your own words. Claude will help structure this into features and tasks.
            </FormDescription>
          </FormHeader>

          <Separator />

          <div className="space-y-2">
            <Label htmlFor="braindump">Your thoughts (edit the questions as you like)</Label>
            <Textarea
              id="braindump"
              value={braindump}
              onChange={(e) => setBraindump(e.target.value)}
              className="min-h-[400px] font-mono text-sm"
              placeholder="Start typing your thoughts..."
            />
            <p className="text-xs text-muted-foreground">
              Tip: Be casual and conversational. Claude understands context and can work with messy notes.
            </p>
          </div>
        </form>
      </ScrollArea>

      <Separator />

      <div className="px-3 py-1.5 flex items-center justify-end gap-2">
        <Button type="button" variant="outline" size="default" onClick={handleCancel}>
          Cancel
        </Button>

        <ModelThinkingPicker
          onAction={handleSubmit}
          actionLabel={isSubmitting ? "Sending..." : "Send to Claude"}
          disabled={isSubmitting || !braindump.trim()}
        />
      </div>
    </div>
  );
}
