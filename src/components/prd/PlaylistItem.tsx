import { memo } from "react";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Item, ItemActions, ItemContent, ItemDescription, ItemMedia, ItemTitle } from "@/components/ui/item";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { PRIORITY_CONFIG, STATUS_CONFIG } from "@/constants/prd";
import type { PRDTask } from "@/types/prd";

interface PlaylistItemProps {
  task: PRDTask;
  isNowPlaying?: boolean;
  onClick: () => void;
}

export const PlaylistItem = memo(function PlaylistItem({ task, isNowPlaying = false, onClick }: PlaylistItemProps) {
  const statusConfig = STATUS_CONFIG[task.status];
  const priorityConfig = task.priority ? PRIORITY_CONFIG[task.priority] : null;
  const StatusIcon = statusConfig.icon;

  return (
    <Item
      size="sm"
      variant="default"
      className={`
        cursor-pointer transition-all duration-200
        hover:bg-[hsl(var(--muted)/0.5)]
        ${isNowPlaying ? "bg-[hsl(var(--status-in-progress)/0.1)] border-l-4" : ""}
      `}
      style={isNowPlaying ? { borderLeftColor: statusConfig.color } : undefined}
      onClick={onClick}
    >
      {/* Status Icon */}
      <ItemMedia variant="icon" style={{ backgroundColor: statusConfig.bgColor }}>
        <StatusIcon style={{ color: statusConfig.color }} />
      </ItemMedia>

      {/* Task ID (like track number) */}
      <div className="flex-shrink-0 w-20">
        <span className="text-xs font-mono text-[hsl(var(--muted-foreground))]">{task.id}</span>
      </div>

      {/* Main Content: Title + Description */}
      <ItemContent>
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
          <Alert variant="destructive" className="mt-2 py-1.5 px-2">
            <AlertDescription className="text-xs flex items-center gap-1.5">{task.blocked_by}</AlertDescription>
          </Alert>
        )}
      </ItemContent>

      {/* Right Side: Priority + Metadata */}
      <ItemActions>
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

        {task.tags && task.tags.length > 0 && (
          <Tooltip>
            <TooltipTrigger asChild>
              <Badge variant="secondary" className="text-xs px-1.5 py-0.5 h-5 cursor-help">
                {task.tags.length} tags
              </Badge>
            </TooltipTrigger>
            <TooltipContent>{task.tags.join(", ")}</TooltipContent>
          </Tooltip>
        )}
      </ItemActions>
    </Item>
  );
});
