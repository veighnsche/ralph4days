import { useEffect, useRef } from "react";
import { ScrollArea } from "@/components/ui/scroll-area";
import { useLoopStore } from "@/stores/useLoopStore";
import { cn } from "@/lib/utils";

export function OutputPanel() {
  const { output, rateLimitInfo } = useLoopStore();
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [output]);

  const formatTime = (date: Date) => {
    return date.toLocaleTimeString("en-US", {
      hour12: false,
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
    });
  };

  return (
    <div className="flex h-full flex-col">
      {rateLimitInfo && (
        <div className="mb-2 rounded-md bg-yellow-600/20 border border-yellow-600/50 p-3">
          <div className="flex items-center gap-2">
            <span className="text-yellow-500 font-medium">Rate Limited</span>
            <span className="text-sm text-[hsl(var(--muted-foreground))]">
              Retry attempt {rateLimitInfo.attempt} of {rateLimitInfo.maxAttempts}
            </span>
          </div>
          <RateLimitCountdown info={rateLimitInfo} />
        </div>
      )}

      <ScrollArea
        ref={scrollRef}
        className="flex-1 rounded-md border border-[hsl(var(--border))] bg-black/50 p-3 font-mono text-sm"
      >
        {output.length === 0 ? (
          <div className="text-[hsl(var(--muted-foreground))] italic">
            Output will appear here when the loop starts...
          </div>
        ) : (
          <div className="space-y-0.5">
            {output.map((line) => (
              <div key={line.id} className="flex gap-2">
                <span className="text-[hsl(var(--muted-foreground))] shrink-0">
                  [{formatTime(line.timestamp)}]
                </span>
                <span
                  className={cn("whitespace-pre-wrap break-all", {
                    "text-red-400": line.type === "error",
                    "text-blue-400": line.type === "info",
                    "text-green-400": line.type === "success",
                    "text-[hsl(var(--foreground))]": line.type === "output",
                  })}
                >
                  {line.text}
                </span>
              </div>
            ))}
          </div>
        )}
      </ScrollArea>
    </div>
  );
}

function RateLimitCountdown({ info }: { info: { retryInSecs: number; startTime: Date } }) {
  const elapsed = Math.floor((Date.now() - info.startTime.getTime()) / 1000);
  const remaining = Math.max(0, info.retryInSecs - elapsed);

  const minutes = Math.floor(remaining / 60);
  const seconds = remaining % 60;

  return (
    <div className="text-sm mt-1">
      Retrying in:{" "}
      <span className="font-mono text-yellow-400">
        {minutes}:{seconds.toString().padStart(2, "0")}
      </span>
    </div>
  );
}
