import { FileX } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "@/components/ui/empty";
import { ScrollArea } from "@/components/ui/scroll-area";
import type { PRDTask } from "@/types/prd";
import { KanbanView } from "./KanbanView";

interface PRDBodyProps {
  filteredTasks: PRDTask[];
  onTaskClick: (task: PRDTask) => void;
  onClearFilters: () => void;
}

export function PRDBody({ filteredTasks, onTaskClick, onClearFilters }: PRDBodyProps) {
  return (
    <div className="flex-1 min-h-0 overflow-hidden">
      {filteredTasks.length === 0 ? (
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
      ) : (
        <ScrollArea className="h-full w-full">
          <div className="p-4">
            <KanbanView tasks={filteredTasks} onTaskClick={onTaskClick} />
          </div>
        </ScrollArea>
      )}
    </div>
  );
}
