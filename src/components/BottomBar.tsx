import { Play, Square } from "lucide-react";
import { LoopToggle } from "@/components/LoopToggle";
import { NavigationMenu } from "@/components/NavigationMenu";
import { Settings } from "@/components/Settings";
import { Button } from "@/components/ui/button";
import type { Page } from "@/hooks/useNavigation";

interface BottomBarProps {
  lockedProject: string;
  currentPage: Page;
  onPageChange: (page: Page) => void;
}

// TODO: Wire up to new terminal-based loop system
export function BottomBar({ lockedProject: _lockedProject, currentPage, onPageChange }: BottomBarProps) {
  return (
    <div className="border-t bg-[hsl(var(--background))] px-4 py-3">
      <div className="flex items-center justify-between gap-4">
        {/* Left: Navigation Menu */}
        <div className="flex-1">
          <NavigationMenu currentPage={currentPage} onPageChange={onPageChange} />
        </div>

        {/* Center: Transport Controls (disabled stubs) */}
        <div className="flex items-center gap-3">
          <LoopToggle />

          <Button disabled size="icon" variant="default" title="Start" className="h-10 w-10">
            <Play className="h-5 w-5" />
          </Button>

          <Button disabled size="icon" variant="outline" title="Stop" className="h-10 w-10">
            <Square className="h-5 w-5" />
          </Button>
        </div>

        {/* Right: Settings */}
        <div className="flex-1 flex justify-end">
          <Settings />
        </div>
      </div>
    </div>
  );
}
