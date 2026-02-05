import { FileX } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "@/components/ui/empty";
import type { PRDTask } from "@/types/prd";
import { PlaylistView } from "./PlaylistView";

interface PRDBodyProps {
  filteredTasks: PRDTask[];
  onTaskClick: (task: PRDTask) => void;
  onClearFilters: () => void;
}

export function PRDBody({ filteredTasks, onTaskClick, onClearFilters }: PRDBodyProps) {
  if (filteredTasks.length === 0) {
    return (
      <div className="p-4">
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
      </div>
    );
  }

  return (
    <div className="p-4">
      <PlaylistView tasks={filteredTasks} onTaskClick={onTaskClick} />
    </div>
  );
}
