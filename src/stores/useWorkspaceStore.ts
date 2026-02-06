import type { LucideIcon } from "lucide-react";
import { create } from "zustand";
import type { DisciplineConfig } from "@/hooks/useDisciplines";
import type { Feature, PRDTask } from "@/types/prd";

export type TabType =
  | "terminal"
  | "task-form"
  | "feature-form"
  | "discipline-form"
  | "task-detail"
  | "feature-detail"
  | "discipline-detail"
  | "braindump-form";

export interface WorkspaceTab {
  id: string;
  type: TabType;
  title: string;
  icon?: LucideIcon;
  closeable: boolean;
  data?: {
    mode?: "create" | "edit";
    entityId?: number | string;
    entity?: PRDTask | Feature | DisciplineConfig;
    sessionId?: string; // For output tabs
    model?: string; // For terminal tabs (haiku, sonnet, opus)
    thinking?: boolean; // For terminal tabs (extended thinking)
  };
}

interface WorkspaceStore {
  tabs: WorkspaceTab[];
  activeTabId: string;
  openTab: (tab: Omit<WorkspaceTab, "id"> & { id?: string }) => string;
  closeTab: (tabId: string) => void;
  switchTab: (tabId: string) => void;
  closeAllExcept: (tabId: string) => void;
  /** Browser pattern: tab content pushes its own title/icon up to the tab bar */
  setTabMeta: (tabId: string, meta: { title?: string; icon?: LucideIcon }) => void;
}

const MAX_TABS = 10;

function generateTabId(tab: Omit<WorkspaceTab, "id"> & { id?: string }): string {
  if (tab.id) return tab.id;
  const entityId = tab.data?.entityId;
  const mode = tab.data?.mode;
  if (mode) return `${tab.type}-${mode}${entityId != null ? `-${entityId}` : ""}`;
  if (entityId != null) return `${tab.type}-${entityId}`;
  return `${tab.type}-${Date.now()}`;
}

export const useWorkspaceStore = create<WorkspaceStore>((set, get) => ({
  tabs: [],
  activeTabId: "",

  openTab: (tabInput) => {
    const id = generateTabId(tabInput);
    const { tabs } = get();

    // Duplicate check — focus existing
    const existing = tabs.find((t) => t.id === id);
    if (existing) {
      set({ activeTabId: id });
      return id;
    }

    const tab: WorkspaceTab = { ...tabInput, id };

    // Enforce max tabs — close oldest closeable
    let nextTabs = [...tabs, tab];
    while (nextTabs.length > MAX_TABS) {
      const oldest = nextTabs.find((t) => t.closeable);
      if (!oldest) break;
      nextTabs = nextTabs.filter((t) => t.id !== oldest.id);
    }

    set({ tabs: nextTabs, activeTabId: id });
    return id;
  },

  closeTab: (tabId) => {
    const { tabs, activeTabId } = get();
    const tab = tabs.find((t) => t.id === tabId);
    if (!tab || !tab.closeable) return;

    const nextTabs = tabs.filter((t) => t.id !== tabId);
    let nextActive = activeTabId;

    if (activeTabId === tabId) {
      // Switch to the tab before the closed one, or first available
      const closedIndex = tabs.findIndex((t) => t.id === tabId);
      const prev = tabs[closedIndex - 1] || tabs[closedIndex + 1];
      nextActive = prev?.id ?? "";
    }

    set({ tabs: nextTabs, activeTabId: nextActive });
  },

  switchTab: (tabId) => {
    set({ activeTabId: tabId });
  },

  closeAllExcept: (tabId) => {
    const { tabs } = get();
    set({
      tabs: tabs.filter((t) => t.id === tabId || !t.closeable),
      activeTabId: tabId,
    });
  },

  setTabMeta: (tabId, meta) => {
    const { tabs } = get();
    set({
      tabs: tabs.map((t) =>
        t.id === tabId
          ? {
              ...t,
              ...(meta.title !== undefined && { title: meta.title }),
              ...(meta.icon !== undefined && { icon: meta.icon }),
            }
          : t
      ),
    });
  },
}));
