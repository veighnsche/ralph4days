import type { Meta, StoryObj } from '@storybook/react'
import type { TaskSignal } from '@/types/generated'
import { SignalDisplay } from './SignalDisplay'

const meta = {
  title: 'Workspace/Task Detail/SignalDisplay',
  component: SignalDisplay,
  parameters: {
    layout: 'padded'
  },
  tags: ['autodocs'],
  decorators: [
    Story => (
      <div className="max-w-2xl rounded-lg border bg-card p-4">
        <Story />
      </div>
    )
  ]
} satisfies Meta<typeof SignalDisplay>

export default meta
type Story = StoryObj<typeof meta>

const baseSignal: TaskSignal = {
  id: 101,
  author: 'frontend',
  body: 'Signal body',
  signal_verb: 'done'
}

function makeSignal(overrides: Partial<TaskSignal>): TaskSignal {
  return { ...baseSignal, ...overrides }
}

export const Done: Story = {
  args: {
    signal: makeSignal({
      id: 1,
      signal_verb: 'done',
      summary: 'Completed API integration and verified payload compatibility with the task detail panel.'
    })
  }
}

export const Partial: Story = {
  args: {
    signal: makeSignal({
      id: 2,
      signal_verb: 'partial',
      summary: 'Implemented optimistic updates and list rendering.',
      remaining: 'Need to finalize rollback handling and add visual loading states.'
    })
  }
}

export const Stuck: Story = {
  args: {
    signal: makeSignal({
      id: 3,
      signal_verb: 'stuck',
      reason: 'Backend endpoint returns inconsistent schema for nested reply payloads.'
    })
  }
}

export const AskBlocking: Story = {
  args: {
    signal: makeSignal({
      id: 4,
      signal_verb: 'ask',
      question: 'Should we keep replies sorted ascending by creation time after optimistic insertion?',
      blocking: true,
      options: ['Keep strictly ascending', 'Newest first', 'Group by priority then time'],
      preferred: 'Keep strictly ascending'
    })
  }
}

export const AskNonBlockingNoOptions: Story = {
  args: {
    signal: makeSignal({
      id: 5,
      signal_verb: 'ask',
      question: 'Is a compact mobile layout required in this milestone?',
      blocking: false
    })
  }
}

export const FlagBlocking: Story = {
  args: {
    signal: makeSignal({
      id: 6,
      signal_verb: 'flag',
      severity: 'blocking',
      category: 'data-integrity',
      what: 'Editing a top-level signal can overwrite stale local state after concurrent updates.'
    })
  }
}

export const FlagMinor: Story = {
  args: {
    signal: makeSignal({
      id: 7,
      signal_verb: 'flag',
      severity: 'minor',
      category: 'ui-polish',
      what: 'Timestamp alignment shifts by 1px on narrow widths.'
    })
  }
}

export const Learned: Story = {
  args: {
    signal: makeSignal({
      id: 8,
      signal_verb: 'learned',
      text: 'Using verb-specific fields keeps the payload easy to scan and avoids overloading body text.',
      kind: 'architecture',
      rationale: 'Reduces ambiguity and enables targeted rendering for each signal type.',
      scope: 'task-detail panel'
    })
  }
}

export const Suggest: Story = {
  args: {
    signal: makeSignal({
      id: 9,
      signal_verb: 'suggest',
      what: 'Split payload renderer into per-verb subcomponents to simplify testing and story coverage.',
      kind: 'refactor',
      why: 'Current branch-heavy rendering is harder to reason about and regression-test.'
    })
  }
}

export const Blocked: Story = {
  args: {
    signal: makeSignal({
      id: 10,
      signal_verb: 'blocked',
      kind: 'dependency',
      on: 'Awaiting API contract decision for signal reply threading.'
    })
  }
}

export const SparsePayloads: Story = {
  args: {
    signal: baseSignal
  },
  render: () => {
    const sparse: TaskSignal[] = [
      makeSignal({ id: 11, signal_verb: 'done' }),
      makeSignal({ id: 12, signal_verb: 'partial', summary: 'Started migration only.' }),
      makeSignal({ id: 13, signal_verb: 'stuck' }),
      makeSignal({ id: 14, signal_verb: 'ask', question: 'Need confirmation?' }),
      makeSignal({ id: 15, signal_verb: 'flag', what: 'Potential issue found.' }),
      makeSignal({ id: 16, signal_verb: 'learned', text: 'Cache hit ratio improves with memoized selectors.' }),
      makeSignal({ id: 17, signal_verb: 'suggest', what: 'Add keyboard shortcut for replies.' }),
      makeSignal({ id: 18, signal_verb: 'blocked' })
    ]

    return (
      <div className="space-y-4">
        {sparse.map(signal => (
          <div key={signal.id} className="rounded border bg-background p-3">
            <div className="mb-2 text-xs uppercase tracking-wide text-muted-foreground">{signal.signal_verb}</div>
            <SignalDisplay signal={signal} />
          </div>
        ))}
      </div>
    )
  }
}

export const UnknownOrMissingVerb: Story = {
  args: {
    signal: baseSignal
  },
  render: () => {
    const variants: TaskSignal[] = [
      makeSignal({ id: 19, signal_verb: undefined }),
      makeSignal({ id: 20, signal_verb: 'custom_verb_not_supported' })
    ]

    return (
      <div className="space-y-4">
        {variants.map(signal => (
          <div key={signal.id} className="rounded border bg-background p-3">
            <div className="mb-2 text-xs uppercase tracking-wide text-muted-foreground">
              verb: {signal.signal_verb ?? '(none)'}
            </div>
            <SignalDisplay signal={signal} />
            <div className="mt-2 text-xs text-muted-foreground">No payload is rendered for unsupported verbs.</div>
          </div>
        ))}
      </div>
    )
  }
}

export const AllVerbVariantsMatrix: Story = {
  args: {
    signal: baseSignal
  },
  render: () => {
    const variants: Array<{ label: string; signal: TaskSignal }> = [
      {
        label: 'done',
        signal: makeSignal({ id: 21, signal_verb: 'done', summary: 'Feature delivered and merged.' })
      },
      {
        label: 'partial',
        signal: makeSignal({
          id: 22,
          signal_verb: 'partial',
          summary: 'Base flow done.',
          remaining: 'Needs a11y and keyboard interactions.'
        })
      },
      {
        label: 'stuck',
        signal: makeSignal({ id: 23, signal_verb: 'stuck', reason: 'Cannot reproduce race condition reliably.' })
      },
      {
        label: 'ask',
        signal: makeSignal({
          id: 24,
          signal_verb: 'ask',
          question: 'Which pagination style should we use?',
          options: ['Cursor', 'Offset'],
          preferred: 'Cursor'
        })
      },
      {
        label: 'flag',
        signal: makeSignal({
          id: 25,
          signal_verb: 'flag',
          severity: 'major',
          category: 'performance',
          what: 'Large lists trigger expensive rerenders.'
        })
      },
      {
        label: 'learned',
        signal: makeSignal({
          id: 26,
          signal_verb: 'learned',
          text: 'Flattening reply trees before render simplifies ordering.',
          kind: 'implementation',
          rationale: 'Less repeated sorting in render path.'
        })
      },
      {
        label: 'suggest',
        signal: makeSignal({
          id: 27,
          signal_verb: 'suggest',
          what: 'Adopt per-verb schema validation in signal creation.',
          why: 'Prevents malformed payloads from reaching UI.'
        })
      },
      {
        label: 'blocked',
        signal: makeSignal({
          id: 28,
          signal_verb: 'blocked',
          kind: 'external',
          on: 'Pending design approval for card density changes.'
        })
      }
    ]

    return (
      <div className="grid gap-3 md:grid-cols-2">
        {variants.map(({ label, signal }) => (
          <div key={signal.id} className="rounded border bg-background p-3">
            <div className="mb-2 text-xs uppercase tracking-wide text-muted-foreground">{label}</div>
            <SignalDisplay signal={signal} />
          </div>
        ))}
      </div>
    )
  }
}
