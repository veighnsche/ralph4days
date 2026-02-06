import { Brain, FileX, MessageSquare } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "@/components/ui/empty";
import type { PRDTask } from "@/types/prd";
import { PlaylistView } from "./PlaylistView";

interface PRDBodyProps {
  filteredTasks: PRDTask[];
  totalTasks: number;
  onTaskClick: (task: PRDTask) => void;
  onClearFilters: () => void;
  onBraindump: () => void;
  onYap: () => void;
}

export function PRDBody({ filteredTasks, totalTasks, onTaskClick, onClearFilters, onBraindump, onYap }: PRDBodyProps) {
  if (filteredTasks.length === 0) {
    // No tasks at all - show braindump CTA
    if (totalTasks === 0) {
      return (
        <Empty>
          <EmptyHeader>
            <EmptyMedia variant="icon">
              <Brain />
            </EmptyMedia>
            <EmptyTitle>No tasks yet</EmptyTitle>
            <EmptyDescription>
              Get started by braindumping your project ideas. Claude will help structure them into features and tasks.
            </EmptyDescription>
          </EmptyHeader>
          <EmptyContent>
            <div className="flex flex-col gap-2">
              <Button onClick={onBraindump}>
                <Brain className="h-4 w-4 mr-2" />
                Braindump Project
              </Button>
              <Button onClick={onYap} variant="outline">
                <MessageSquare className="h-4 w-4 mr-2" />
                Yap about Tasks
              </Button>
            </div>
          </EmptyContent>
        </Empty>
      );
    }

    // Tasks exist but filtered out
    return (
      <Empty>
        <EmptyHeader>
          <EmptyMedia variant="icon">
            <FileX />
          </EmptyMedia>
          <EmptyTitle>No tasks found</EmptyTitle>
          <EmptyDescription>
            No tasks match your current filters. Try adjusting your search criteria or clearing filters.
          </EmptyDescription>
        </EmptyHeader>
        <EmptyContent>
          <Button variant="outline" onClick={onClearFilters}>
            Clear all filters
          </Button>
        </EmptyContent>
      </Empty>
    );
  }

  return <PlaylistView tasks={filteredTasks} onTaskClick={onTaskClick} />;
}
