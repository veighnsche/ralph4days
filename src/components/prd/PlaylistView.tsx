import { AlertCircle } from "lucide-react";
import { memo, useMemo } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { ItemGroup, ItemSeparator } from "@/components/ui/item";
import { TooltipProvider } from "@/components/ui/tooltip";
import type { PRDTask } from "@/types/prd";
import { PlaylistItem } from "./PlaylistItem";

interface PlaylistViewProps {
  tasks: PRDTask[];
  onTaskClick: (task: PRDTask) => void;
}

export const PlaylistView = memo(function PlaylistView({ tasks, onTaskClick }: PlaylistViewProps) {
  const { blockedSkipped, done, inProgress, pending } = useMemo(() => {
    const result = {
      blockedSkipped: [] as PRDTask[],
      done: [] as PRDTask[],
      inProgress: [] as PRDTask[],
      pending: [] as PRDTask[],
    };

    tasks.forEach((task) => {
      if (task.status === "blocked" || task.status === "skipped") {
        result.blockedSkipped.push(task);
      } else if (task.status === "done") {
        result.done.push(task);
      } else if (task.status === "in_progress") {
        result.inProgress.push(task);
      } else if (task.status === "pending") {
        result.pending.push(task);
      }
    });

    return result;
  }, [tasks]);

  const hasBlockedOrSkipped = blockedSkipped.length > 0;

  return (
    <TooltipProvider>
      <div className="flex flex-col gap-6 pb-4">
        {/* Blocked/Skipped Section */}
        {hasBlockedOrSkipped && (
          <Card className="border-[hsl(var(--status-blocked))] bg-[hsl(var(--status-blocked)/0.05)]">
            <CardHeader className="pb-3">
              <CardTitle className="text-sm flex items-center gap-2" style={{ color: "hsl(var(--status-blocked))" }}>
                <AlertCircle className="h-4 w-4" />
                Issues Requiring Attention
                <span className="ml-auto text-xs font-normal opacity-70">({blockedSkipped.length})</span>
              </CardTitle>
            </CardHeader>
            <CardContent className="pt-0">
              <ItemGroup className="rounded-lg border bg-[hsl(var(--card))]">
                {blockedSkipped.map((task, index) => (
                  <>
                    <PlaylistItem key={task.id} task={task} onClick={() => onTaskClick(task)} />
                    {index < blockedSkipped.length - 1 && <ItemSeparator />}
                  </>
                ))}
              </ItemGroup>
            </CardContent>
          </Card>
        )}

        {/* Main Playlist */}
        <ItemGroup className="rounded-md border pb-8">
          {/* Completed Tasks */}
          {done.map((task) => (
            <>
              <PlaylistItem key={task.id} task={task} onClick={() => onTaskClick(task)} />
              <ItemSeparator />
            </>
          ))}

          {/* Now Playing */}
          {inProgress.map((task) => (
            <>
              <PlaylistItem key={task.id} task={task} isNowPlaying onClick={() => onTaskClick(task)} />
              <ItemSeparator />
            </>
          ))}

          {/* Pending Tasks */}
          {pending.map((task, index) => (
            <>
              <PlaylistItem key={task.id} task={task} onClick={() => onTaskClick(task)} />
              {index < pending.length - 1 && <ItemSeparator />}
            </>
          ))}

          {/* Empty State */}
          {tasks.length === 0 && (
            <div className="flex items-center justify-center h-32">
              <p className="text-sm text-[hsl(var(--muted-foreground))]">No tasks in playlist</p>
            </div>
          )}
        </ItemGroup>
      </div>
    </TooltipProvider>
  );
});
