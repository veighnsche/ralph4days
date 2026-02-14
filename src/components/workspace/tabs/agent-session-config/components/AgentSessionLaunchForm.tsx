import type { ReactNode } from 'react'
import { InlineError } from '@/components/shared'
import { FormDescription, FormHeader, FormTitle } from '@/components/ui/form-header'
import { Separator } from '@/components/ui/separator'
import type { TerminalBridgeModelOption } from '@/types/generated'
import { AgentProviderPicker, ModelPicker } from '.'

export type AgentSessionLaunchFormLayout = 'two_column' | 'stacked'

export function AgentSessionLaunchForm({
  layout = 'two_column',
  showHeader = true,
  models,
  loadingModels,
  error,
  footer
}: {
  layout?: AgentSessionLaunchFormLayout
  showHeader?: boolean
  models: TerminalBridgeModelOption[]
  loadingModels: boolean
  error: string | null
  footer?: ReactNode
}) {
  return (
    <div className="h-full flex flex-col">
      {showHeader ? (
        <>
          <div className="px-4">
            <FormHeader>
              <FormTitle>Start Agent Session</FormTitle>
              <FormDescription>Configure launch options, then start an agent session.</FormDescription>
            </FormHeader>
          </div>
          <Separator />
        </>
      ) : null}

      <div className="flex-1 min-h-0 overflow-hidden">
        {layout === 'two_column' ? (
          <div className="grid h-full grid-cols-1 gap-4 px-4 py-4 md:grid-cols-2">
            <div className="min-h-[26rem] min-w-0 overflow-hidden">
              <AgentProviderPicker />
            </div>
            <div className="min-h-0 min-w-0 flex flex-col">
              <ModelPicker models={models} loadingModels={loadingModels} />
            </div>
          </div>
        ) : (
          <div className="flex h-full flex-col gap-4 px-4 py-4 overflow-hidden">
            <div className="min-w-0 shrink-0">
              <AgentProviderPicker />
            </div>
            <div className="min-h-0 min-w-0 flex flex-col">
              <ModelPicker models={models} loadingModels={loadingModels} />
            </div>
          </div>
        )}
      </div>

      {footer ? (
        <>
          <Separator />
          {error ? (
            <div className="px-3 pt-1.5">
              <InlineError error={error} />
            </div>
          ) : null}
          <div className="px-3 py-1.5">{footer}</div>
        </>
      ) : error ? (
        <>
          <Separator />
          <div className="px-3 py-1.5">
            <InlineError error={error} />
          </div>
        </>
      ) : null}
    </div>
  )
}
