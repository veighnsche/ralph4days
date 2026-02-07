import { render, waitFor } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { Terminal } from './terminal'

const mockTerminalDispose = vi.fn()
const mockTerminalOpen = vi.fn()
const mockTerminalOnResize = vi.fn()
const mockLoadAddon = vi.fn()
const mockFitAddonFit = vi.fn()

vi.mock('@xterm/xterm', () => {
  return {
    // biome-ignore lint/complexity/useArrowFunction: must use `function` for `new` calls
    Terminal: vi.fn().mockImplementation(function () {
      return {
        cols: 80,
        rows: 24,
        write: vi.fn(),
        writeln: vi.fn(),
        dispose: mockTerminalDispose,
        open: mockTerminalOpen,
        onData: vi.fn(),
        onResize: mockTerminalOnResize,
        loadAddon: mockLoadAddon
      }
    })
  }
})

vi.mock('@xterm/addon-fit', () => {
  return {
    // biome-ignore lint/complexity/useArrowFunction: must use `function` for `new` calls
    FitAddon: vi.fn().mockImplementation(function () {
      return { fit: mockFitAddonFit }
    })
  }
})

vi.mock('@xterm/addon-web-links', () => {
  return {
    // biome-ignore lint/complexity/useArrowFunction: must use `function` for `new` calls
    WebLinksAddon: vi.fn().mockImplementation(function () {
      return {}
    })
  }
})

// biome-ignore lint/complexity/useArrowFunction: must use `function` for `new` calls
global.ResizeObserver = vi.fn().mockImplementation(function () {
  return { observe: vi.fn(), unobserve: vi.fn(), disconnect: vi.fn() }
})

describe('Terminal', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  it('renders terminal container', () => {
    const { container } = render(<Terminal />)
    const terminalDiv = container.querySelector('div')
    expect(terminalDiv).toBeTruthy()
  })

  it('creates XTerm with hardcoded config', async () => {
    const { Terminal: MockXTerm } = await import('@xterm/xterm')

    render(<Terminal />)

    await waitFor(() => {
      expect(MockXTerm).toHaveBeenCalledWith(
        expect.objectContaining({
          cursorBlink: true,
          fontSize: 13,
          scrollback: 10000
        })
      )
    })
  })

  it('loads FitAddon and WebLinksAddon', async () => {
    const { FitAddon } = await import('@xterm/addon-fit')
    const { WebLinksAddon } = await import('@xterm/addon-web-links')

    render(<Terminal />)

    await waitFor(() => {
      expect(FitAddon).toHaveBeenCalled()
      expect(WebLinksAddon).toHaveBeenCalled()
      expect(mockLoadAddon).toHaveBeenCalledTimes(2)
    })
  })

  it('calls onReady when terminal is initialized', async () => {
    const onReady = vi.fn()
    render(<Terminal onReady={onReady} />)

    await waitFor(() => {
      expect(onReady).toHaveBeenCalledWith(expect.any(Object))
    })
  })

  it('opens terminal in container and fits', async () => {
    render(<Terminal />)

    await waitFor(() => {
      expect(mockTerminalOpen).toHaveBeenCalledWith(expect.any(HTMLDivElement))
      expect(mockFitAddonFit).toHaveBeenCalled()
    })
  })

  it('always registers resize handler', async () => {
    render(<Terminal />)

    await waitFor(() => {
      expect(mockTerminalOnResize).toHaveBeenCalledWith(expect.any(Function))
    })
  })

  it('sets up resize observer', async () => {
    render(<Terminal />)

    await waitFor(() => {
      expect(global.ResizeObserver).toHaveBeenCalled()
    })
  })

  it('disposes terminal on unmount', async () => {
    const { unmount } = render(<Terminal />)

    await waitFor(() => {
      expect(mockTerminalOpen).toHaveBeenCalled()
    })

    unmount()

    expect(mockTerminalDispose).toHaveBeenCalled()
  })

  it('does not recreate terminal on rerender', async () => {
    const { Terminal: MockXTerm } = await import('@xterm/xterm')

    const { rerender } = render(<Terminal />)

    await waitFor(() => {
      expect(MockXTerm).toHaveBeenCalledTimes(1)
    })

    rerender(<Terminal />)

    await waitFor(() => {
      expect(MockXTerm).toHaveBeenCalledTimes(1)
    })
  })

  it('applies dark theme colors', async () => {
    const { Terminal: MockXTerm } = await import('@xterm/xterm')

    render(<Terminal />)

    await waitFor(() => {
      expect(MockXTerm).toHaveBeenCalledWith(
        expect.objectContaining({
          theme: expect.objectContaining({
            background: '#0a0a0a',
            foreground: '#e0e0e0',
            cursor: '#e0e0e0'
          })
        })
      )
    })
  })
})
