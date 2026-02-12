import { useEffect, useState } from 'react'
import { terminalBridgeListModelFormTree } from '@/lib/terminal/terminalBridgeClient'
import type { TerminalBridgeModelOption } from '@/types/generated'
import { groupModelsByAgent } from '../state'

export function useModelFormTreeByAgent() {
  const [formTreeByAgent, setFormTreeByAgent] = useState<Record<string, TerminalBridgeModelOption[]>>({})
  const [formTreeLoading, setFormTreeLoading] = useState(true)
  const [formTreeError, setFormTreeError] = useState<string | null>(null)

  useEffect(() => {
    let active = true

    const load = async () => {
      setFormTreeLoading(true)
      setFormTreeError(null)
      try {
        const result = await terminalBridgeListModelFormTree()
        if (!active) return
        setFormTreeByAgent(groupModelsByAgent(result.providers))
      } catch (error) {
        if (!active) return
        setFormTreeByAgent({})
        setFormTreeError(`Failed to load model form tree: ${String(error)}`)
      } finally {
        if (active) setFormTreeLoading(false)
      }
    }

    void load()
    return () => {
      active = false
    }
  }, [])

  return { formTreeByAgent, formTreeLoading, formTreeError }
}
