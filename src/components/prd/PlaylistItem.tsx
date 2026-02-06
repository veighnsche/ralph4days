import { Bot, Cog, User } from "lucide-react";
import { memo } from "react";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Item, ItemActions, ItemContent, ItemDescription, ItemTitle } from "@/components/ui/item";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { INFERRED_STATUS_CONFIG, PRIORITY_CONFIG, STATUS_CONFIG } from "@/constants/prd";
import { getInferredStatusExplanation } from "@/lib/taskStatus";
import type { EnrichedTask } from "@/types/prd";
import { TaskIdDisplay } from "./TaskIdDisplay";

interface PlaylistItemProps {
  task: EnrichedTask;
  isNowPlaying?: boolean;
  isIssue?: boolean;
  onClick: () => void;
}

function getItemStyle(
  status: EnrichedTask["status"],
  statusConfig: (typeof STATUS_CONFIG)[keyof typeof STATUS_CONFIG]
) {
  return {
    borderLeftColor: statusConfig.color,
    backgroundColor: statusConfig.bgColor,
    opacity: status === "done" || status === "skipped" ? 0.5 : 1,
  };
}

export const PlaylistItem = memo(function PlaylistItem({ task, isNowPlaying = false, onClick }: PlaylistItemProps) {
  const statusConfig = STATUS_CONFIG[task.status];
  const priorityConfig = task.priority ? PRIORITY_CONFIG[task.priority] : null;

  return (
    <Item
      size="sm"
      variant="default"
      className="cursor-pointer transition-all duration-200 hover:opacity-80 border-l-4 relative overflow-hidden"
      style={getItemStyle(task.status, statusConfig)}
      onClick={onClick}
    >
      {/* Priority Color Gradient (upper right corner) */}
      {priorityConfig && (
        <div
          className="absolute top-0 right-0 w-32 h-32 pointer-events-none"
          style={{
            background: `radial-gradient(circle at top right, ${priorityConfig.bgColor} 0%, transparent 70%)`,
            opacity: 1.0,
          }}
        />
      )}

      {/* Task ID with Icon */}
      <div className="flex-shrink-0 self-start">
        <TaskIdDisplay task={task} />
      </div>

      {/* Main Content: Title + Description */}
      <ItemContent className="gap-0">
        <ItemTitle
          className={isNowPlaying ? "text-base" : "text-sm"}
          style={isNowPlaying ? { color: statusConfig.color } : undefined}
        >
          {task.title}
          {isNowPlaying && <span className="ml-2 text-xs opacity-70">[NOW PLAYING]</span>}
        </ItemTitle>

        {task.description && <ItemDescription className="truncate">{task.description}</ItemDescription>}

        {/* Blocked By Alert */}
        {task.blockedBy && (
          <Alert variant="destructive" className="mt-1 py-1.5 px-2">
            <AlertDescription className="text-xs flex items-center gap-1.5">{task.blockedBy}</AlertDescription>
          </Alert>
        )}
      </ItemContent>

      {/* Right Side: Priority + Metadata */}
      <ItemActions className="flex-col items-end gap-2">
        {/* Top Row: Counts + Priority */}
        <div className="flex items-center gap-2">
          {/* Provenance Icon */}
          {task.provenance &&
            (() => {
              const Icon = task.provenance === "agent" ? Bot : task.provenance === "human" ? User : Cog;
              return (
                <Tooltip>
                  <TooltipTrigger asChild>
                    <div className="flex items-center h-5 px-1 cursor-help">
                      <Icon className="h-3.5 w-3.5 text-muted-foreground" />
                    </div>
                  </TooltipTrigger>
                  <TooltipContent>Created by {task.provenance}</TooltipContent>
                </Tooltip>
              );
            })()}

          {/* Metadata Badges */}
          {task.acceptanceCriteria && task.acceptanceCriteria.length > 0 && (
            <Tooltip>
              <TooltipTrigger asChild>
                <Badge variant="outline" className="text-xs px-1.5 py-0.5 h-5 cursor-help">
                  {task.acceptanceCriteria.length} AC
                </Badge>
              </TooltipTrigger>
              <TooltipContent>{task.acceptanceCriteria.length} Acceptance Criteria</TooltipContent>
            </Tooltip>
          )}

          {/* Dependencies Badge - enhanced with inferred status color */}
          {task.dependsOn &&
            task.dependsOn.length > 0 &&
            (() => {
              const isWaiting = task.inferredStatus === "waiting_on_deps";
              const inferredConfig = isWaiting ? INFERRED_STATUS_CONFIG.waiting_on_deps : null;

              return (
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Badge
                      variant="outline"
                      className="text-xs px-1.5 py-0.5 h-5 cursor-help"
                      style={
                        inferredConfig
                          ? {
                              borderColor: inferredConfig.color,
                              color: inferredConfig.color,
                              backgroundColor: inferredConfig.bgColor,
                            }
                          : undefined
                      }
                    >
                      {task.dependsOn.length} dep{task.dependsOn.length !== 1 ? "s" : ""}
                    </Badge>
                  </TooltipTrigger>
                  <TooltipContent>
                    {isWaiting
                      ? getInferredStatusExplanation(task.status, task.inferredStatus, task.dependsOn.length)
                      : `${task.dependsOn.length} ${task.dependsOn.length === 1 ? "Dependency" : "Dependencies"}`}
                  </TooltipContent>
                </Tooltip>
              );
            })()}

          {/* Priority Badge */}
          {priorityConfig && (
            <Tooltip>
              <TooltipTrigger asChild>
                <Badge
                  variant="outline"
                  className="text-xs px-2 py-0.5 h-5 cursor-help"
                  style={{
                    backgroundColor: priorityConfig.bgColor,
                    color: priorityConfig.color,
                    borderColor: priorityConfig.color,
                  }}
                >
                  {priorityConfig.label}
                </Badge>
              </TooltipTrigger>
              <TooltipContent>{priorityConfig.label} Priority</TooltipContent>
            </Tooltip>
          )}
        </div>

        {/* Bottom Row: Individual Tags */}
        {task.tags && task.tags.length > 0 && (
          <div className="flex flex-wrap gap-1 justify-end">
            {task.tags.map((tag) => (
              <Badge key={tag} variant="outline" className="text-xs px-2.5 py-0.5 h-5 min-w-[3rem]">
                {tag}
              </Badge>
            ))}
          </div>
        )}
      </ItemActions>
    </Item>
  );
});
