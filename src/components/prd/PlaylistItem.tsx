import { memo } from "react";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Item, ItemActions, ItemContent, ItemDescription, ItemTitle } from "@/components/ui/item";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { PRIORITY_CONFIG, STATUS_CONFIG } from "@/constants/prd";
import type { PRDTask } from "@/types/prd";
import { TaskIdDisplay } from "./TaskIdDisplay";

interface PlaylistItemProps {
  task: PRDTask;
  isNowPlaying?: boolean;
  isIssue?: boolean;
  onClick: () => void;
}

function getItemStyle(status: PRDTask["status"], statusConfig: (typeof STATUS_CONFIG)[keyof typeof STATUS_CONFIG]) {
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
      className="cursor-pointer transition-all duration-200 hover:opacity-80 border-l-4"
      style={getItemStyle(task.status, statusConfig)}
      onClick={onClick}
    >
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
        {task.blocked_by && (
          <Alert variant="destructive" className="mt-1 py-1.5 px-2">
            <AlertDescription className="text-xs flex items-center gap-1.5">{task.blocked_by}</AlertDescription>
          </Alert>
        )}
      </ItemContent>

      {/* Right Side: Priority + Metadata */}
      <ItemActions className="flex-col items-end gap-2">
        {/* Top Row: Counts + Priority */}
        <div className="flex items-center gap-2">
          {/* Metadata Badges */}
          {task.acceptance_criteria && task.acceptance_criteria.length > 0 && (
            <Tooltip>
              <TooltipTrigger asChild>
                <Badge variant="outline" className="text-xs px-1.5 py-0.5 h-5 cursor-help">
                  {task.acceptance_criteria.length} AC
                </Badge>
              </TooltipTrigger>
              <TooltipContent>{task.acceptance_criteria.length} Acceptance Criteria</TooltipContent>
            </Tooltip>
          )}

          {task.depends_on && task.depends_on.length > 0 && (
            <Tooltip>
              <TooltipTrigger asChild>
                <Badge variant="outline" className="text-xs px-1.5 py-0.5 h-5 cursor-help">
                  {task.depends_on.length} deps
                </Badge>
              </TooltipTrigger>
              <TooltipContent>{task.depends_on.length} Dependencies</TooltipContent>
            </Tooltip>
          )}

          {/* Priority Badge */}
          {priorityConfig && (
            <Tooltip>
              <TooltipTrigger asChild>
                <div
                  className="px-2 py-0.5 rounded text-xs font-medium cursor-help"
                  style={{
                    backgroundColor: priorityConfig.bgColor,
                    color: priorityConfig.color,
                  }}
                >
                  {priorityConfig.label}
                </div>
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
