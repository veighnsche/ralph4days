import type { LucideIcon } from 'lucide-react'
import { useEffect } from 'react'
import { useWorkspaceStore } from '@/stores/useWorkspaceStore'

// WHY: Tab content declares its own title/icon (like <title> tag), not parent-driven
export function useTabMeta(tabId: string, title: string, icon: LucideIcon) {
  const setTabMeta = useWorkspaceStore(s => s.setTabMeta)

  useEffect(() => {
    setTabMeta(tabId, { title, icon })
  }, [tabId, title, icon, setTabMeta])
}
