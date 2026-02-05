import type { Preview } from '@storybook/react-vite'
import type { ReactRenderer } from '@storybook/react'
import React from 'react'
import { TooltipProvider } from '@/components/ui/tooltip'
import '../src/index.css'; // Import Tailwind CSS

const preview: Preview = {
  parameters: {
    controls: {
      matchers: {
       color: /(background|color)$/i,
       date: /Date$/i,
      },
    },

    a11y: {
      // 'todo' - show a11y violations in the test UI only
      // 'error' - fail CI on a11y violations
      // 'off' - skip a11y checks entirely
      test: 'todo'
    }
  },
  decorators: [
    (Story) => (
      <TooltipProvider>
        <Story />
      </TooltipProvider>
    ),
  ],
};

export default preview;