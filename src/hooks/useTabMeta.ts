import type { LucideIcon } from "lucide-react";
import { useEffect } from "react";
import { useWorkspaceStore } from "@/stores/useWorkspaceStore";

/**
 * Browser pattern: tab content declares its own title and icon.
 * Like <title> and <link rel="icon"> — the page tells the browser, not the other way around.
 */
export function useTabMeta(tabId: string, title: string, icon: LucideIcon) {
  const setTabMeta = useWorkspaceStore((s) => s.setTabMeta);

  useEffect(() => {
    setTabMeta(tabId, { title, icon });
  }, [tabId, title, icon, setTabMeta]);
}
