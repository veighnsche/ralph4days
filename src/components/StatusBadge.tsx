import { Badge } from "@/components/ui/badge";
import { LoopState } from "@/stores/useLoopStore";

interface StatusBadgeProps {
  state: LoopState;
}

const stateConfig: Record<
  LoopState,
  { label: string; variant: "default" | "secondary" | "destructive" | "success" | "warning" }
> = {
  idle: { label: "Idle", variant: "secondary" },
  running: { label: "Running", variant: "default" },
  paused: { label: "Paused", variant: "warning" },
  rate_limited: { label: "Rate Limited", variant: "warning" },
  complete: { label: "Complete", variant: "success" },
  aborted: { label: "Aborted", variant: "destructive" },
};

export function StatusBadge({ state }: StatusBadgeProps) {
  const config = stateConfig[state];

  return (
    <Badge variant={config.variant} className="text-sm">
      {state === "running" && (
        <span className="mr-1.5 inline-block h-2 w-2 animate-pulse bg-white" />
      )}
      {config.label}
    </Badge>
  );
}
