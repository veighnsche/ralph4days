import type { Agent, PermissionLevel } from '@/hooks/preferences'

export const AGENT_OPTIONS: Agent[] = ['claude', 'codex']

export const AGENT_PROVIDER_META = {
  claude: {
    label: 'Claude',
    logoSrc: '/reference-logos/anthropic.svg',
    logoAlt: 'Anthropic logo'
  },
  codex: {
    label: 'Codex',
    logoSrc: '/reference-logos/openai.svg',
    logoAlt: 'OpenAI logo'
  }
} as const

export const PERMISSION_LEVEL_OPTIONS: Array<{ value: PermissionLevel; label: string }> = [
  { value: 'safe', label: 'Safe' },
  { value: 'balanced', label: 'Balanced' },
  { value: 'auto', label: 'Auto' },
  { value: 'full_auto', label: 'Full Auto' }
]

export const MODEL_FORM_TREE_QUERY_KEY = ['terminal', 'model-form-tree'] as const

export function isPermissionLevel(value: string): value is PermissionLevel {
  return PERMISSION_LEVEL_OPTIONS.some(option => option.value === value)
}
