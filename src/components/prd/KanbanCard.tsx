import { memo } from "react";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  CardDescription,
  CardFooter,
  CardAction,
} from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { AlertCircle } from "lucide-react";
import { PRIORITY_CONFIG } from "@/constants/prd";
import type { PRDTask } from "@/types/prd";

interface KanbanCardProps {
  task: PRDTask;
  columnColor: string;
  onClick: () => void;
}

export const KanbanCard = memo(function KanbanCard({ task, columnColor, onClick }: KanbanCardProps) {
  const priorityConfig = task.priority ? PRIORITY_CONFIG[task.priority] : null;

  return (
    <Card
      className="cursor-pointer hover:shadow-lg transition-all duration-200 group py-0 gap-0 overflow-hidden flex flex-col"
      style={{ aspectRatio: "7/5" }}
      onClick={onClick}
    >
      {/* Status Color Bar */}
      <div className="h-1.5" style={{ backgroundColor: columnColor }} />

      <CardHeader className="pb-2 pt-3 flex-shrink-0">
        {/* Priority Badge as CardAction */}
        {priorityConfig && (
          <CardAction>
            <Tooltip>
              <TooltipTrigger asChild>
                <div
                  className="flex items-center px-2 py-0.5 rounded text-xs font-medium cursor-help"
                  style={{
                    backgroundColor: priorityConfig.bgColor,
                    color: priorityConfig.color,
                  }}
                >
                  {priorityConfig.label}
                </div>
              </TooltipTrigger>
              <TooltipContent>{priorityConfig.label} Priority Task</TooltipContent>
            </Tooltip>
          </CardAction>
        )}

        {/* Task Title */}
        <CardTitle className="text-sm line-clamp-2 leading-tight">{task.title}</CardTitle>

        {/* Task Description */}
        {task.description && (
          <CardDescription className="text-xs line-clamp-1 leading-relaxed">{task.description}</CardDescription>
        )}
      </CardHeader>

      <CardContent className="space-y-2 flex-1 overflow-auto">
        {/* Blocked By Alert */}
        {task.blocked_by && (
          <Tooltip>
            <TooltipTrigger asChild>
              <div
                className="flex items-start gap-1.5 text-xs px-2 py-1.5 rounded cursor-help"
                style={{
                  backgroundColor: "hsl(var(--status-blocked) / 0.1)",
                  color: "hsl(var(--status-blocked))",
                }}
              >
                <AlertCircle className="h-3.5 w-3.5 flex-shrink-0 mt-0.5" />
                <span className="line-clamp-2 flex-1">{task.blocked_by}</span>
              </div>
            </TooltipTrigger>
            <TooltipContent>
              <p className="max-w-xs">{task.blocked_by}</p>
            </TooltipContent>
          </Tooltip>
        )}

        {/* Metadata Badges */}
        <div className="flex items-center gap-1.5 flex-wrap">
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
        </div>

        {/* Tags */}
        {task.tags && task.tags.length > 0 && (
          <div className="flex gap-1 flex-wrap">
            {task.tags.slice(0, 3).map((tag) => (
              <span
                key={tag}
                className="text-xs bg-[hsl(var(--muted))] text-[hsl(var(--muted-foreground))] px-2 py-0.5 rounded-full"
              >
                {tag}
              </span>
            ))}
            {task.tags.length > 3 && (
              <Tooltip>
                <TooltipTrigger asChild>
                  <span className="text-xs text-[hsl(var(--muted-foreground))] px-1 cursor-help">
                    +{task.tags.length - 3}
                  </span>
                </TooltipTrigger>
                <TooltipContent>{task.tags.slice(3).join(", ")}</TooltipContent>
              </Tooltip>
            )}
          </div>
        )}
      </CardContent>

      {/* Task ID in CardFooter */}
      <CardFooter className="border-t flex-shrink-0">
        <span className="text-xs text-[hsl(var(--muted-foreground))] font-mono">{task.id}</span>
      </CardFooter>
    </Card>
  );
});
