import { X } from "lucide-react";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { cn } from "@/lib/utils";

export interface BrowserTab {
  id: string;
  title: string;
}

interface BrowserTabsProps {
  tabs: BrowserTab[];
  activeTabId: string;
  onTabChange: (tabId: string) => void;
  onTabClose?: (tabId: string) => void;
  className?: string;
}

export function BrowserTabs({ tabs, activeTabId, onTabChange, onTabClose, className }: BrowserTabsProps) {
  return (
    <Tabs value={activeTabId} onValueChange={onTabChange} className={cn("w-full", className)}>
      <TabsList
        variant="line"
        className="w-full justify-start rounded-none border-b border-border h-auto p-0 px-1 pt-1 gap-1 bg-muted/30"
      >
        {tabs.map((tab) => (
          <TabsTrigger
            key={tab.id}
            value={tab.id}
            className={cn(
              "group relative flex-none",
              "rounded-t-md rounded-b-none px-3 py-1.5 gap-2",
              // Remove default shadcn tab styling
              "border after:hidden",
              // Inactive state - visible but muted
              "bg-muted/40 text-muted-foreground border-border/50",
              "hover:bg-muted/60 hover:text-foreground hover:border-border",
              // Active state - browser tab look with strong contrast
              "data-[state=active]:bg-background data-[state=active]:text-foreground",
              "data-[state=active]:border-border data-[state=active]:border-b-background",
              "data-[state=active]:shadow-md",
              // Smooth transitions
              "transition-all duration-150"
            )}
          >
            <span className="truncate max-w-[150px]">{tab.title}</span>

            {onTabClose && (
              <button
                type="button"
                onClick={(e) => {
                  e.stopPropagation();
                  onTabClose(tab.id);
                }}
                className={cn(
                  "ml-1 rounded-sm opacity-0 group-hover:opacity-100 transition-opacity",
                  "hover:bg-muted-foreground/20 hover:text-foreground",
                  "h-4 w-4 inline-flex items-center justify-center",
                  "text-muted-foreground"
                )}
              >
                <X className="h-3 w-3" />
                <span className="sr-only">Close {tab.title}</span>
              </button>
            )}
          </TabsTrigger>
        ))}
      </TabsList>
    </Tabs>
  );
}
