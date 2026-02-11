import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

type MockWS = {
  url: string
  onopen: (() => void) | null
  onmessage: ((event: { data: string }) => void) | null
  onclose: (() => void) | null
  onerror: (() => void) | null
  sent: string[]
  send: (data: string) => void
  close: () => void
}

let wsInstance: MockWS | null = null
let wsConstructions: string[] = []

function ws(): MockWS {
  if (!wsInstance) throw new Error('WebSocket not initialized')
  return wsInstance
}

function createMockWebSocket() {
  wsInstance = null
  wsConstructions = []

  return class MockWebSocket {
    url: string
    onopen: (() => void) | null = null
    onmessage: ((event: { data: string }) => void) | null = null
    onclose: (() => void) | null = null
    onerror: (() => void) | null = null
    sent: string[] = []

    constructor(url: string) {
      this.url = url
      wsConstructions.push(url)
      wsInstance = this as unknown as MockWS
    }

    send(data: string) {
      this.sent.push(data)
    }

    close() {
      this.onclose?.()
    }
  }
}

function sendCommand(cmd: object): object {
  const socket = ws()
  const before = socket.sent.length
  socket.onmessage?.({ data: JSON.stringify(cmd) })
  const after = socket.sent.length
  if (after <= before) throw new Error('No response sent')
  return JSON.parse(socket.sent[after - 1])
}

function makeVisible(el: HTMLElement) {
  Object.defineProperty(el, 'offsetParent', { value: document.body, configurable: true })
}

function el<T extends Element>(selector: string): T {
  const found = document.querySelector<T>(selector)
  if (!found) throw new Error(`Element not found: ${selector}`)
  return found
}

async function loadBridge() {
  await import('./dev-bridge')
  ws().onopen?.()
}

describe('dev-bridge', () => {
  beforeEach(() => {
    vi.useFakeTimers()
    vi.stubGlobal('WebSocket', createMockWebSocket())
    document.body.innerHTML = ''
  })

  afterEach(() => {
    vi.restoreAllMocks()
    vi.useRealTimers()
    vi.resetModules()
    wsInstance = null
  })

  describe('connection', () => {
    it('connects to ws://localhost:9223', async () => {
      await loadBridge()
      expect(wsConstructions).toEqual(['ws://localhost:9223'])
    })

    it('reconnects after 2s on close', async () => {
      await loadBridge()
      expect(wsConstructions).toHaveLength(1)

      ws().onclose?.()
      expect(wsConstructions).toHaveLength(1)

      vi.advanceTimersByTime(2000)
      expect(wsConstructions).toHaveLength(2)
      expect(wsConstructions[1]).toBe('ws://localhost:9223')
    })

    it('closes socket on error then reconnects', async () => {
      await loadBridge()
      const closeSpy = vi.fn()
      ws().close = closeSpy

      ws().onerror?.()
      expect(closeSpy).toHaveBeenCalled()
    })
  })

  describe('eval', () => {
    it('evaluates expression and returns result', async () => {
      await loadBridge()
      const res = sendCommand({ id: '1', type: 'eval', code: '2 + 2' })
      expect(res).toEqual({ id: '1', type: 'result', value: 4 })
    })

    it('evaluates string expression', async () => {
      await loadBridge()
      const res = sendCommand({ id: '2', type: 'eval', code: "'hello'" })
      expect(res).toEqual({ id: '2', type: 'result', value: 'hello' })
    })

    it('falls back to statement evaluation when expression fails', async () => {
      await loadBridge()
      const res = sendCommand({ id: '3', type: 'eval', code: 'if(true) { 42 }' })
      expect(res).toEqual({ id: '3', type: 'result', value: 'undefined' })
    })

    it('returns "undefined" for undefined results', async () => {
      await loadBridge()
      const res = sendCommand({ id: '4', type: 'eval', code: 'undefined' })
      expect(res).toEqual({ id: '4', type: 'result', value: 'undefined' })
    })

    it('returns error when both expression and statement fail', async () => {
      await loadBridge()
      const res = sendCommand({ id: '5', type: 'eval', code: '{{{invalid' })
      expect(res).toEqual({ id: '5', type: 'error', message: expect.stringContaining('') })
    })

    it('evaluates DOM queries', async () => {
      document.body.innerHTML = '<div id="test">Hello</div>'
      await loadBridge()
      const res = sendCommand({ id: '6', type: 'eval', code: "document.getElementById('test').textContent" })
      expect(res).toEqual({ id: '6', type: 'result', value: 'Hello' })
    })

    it('returns object results as JSON', async () => {
      await loadBridge()
      const res = sendCommand({ id: '7', type: 'eval', code: '({a: 1, b: 2})' })
      expect(res).toEqual({ id: '7', type: 'result', value: { a: 1, b: 2 } })
    })
  })

  describe('click by selector', () => {
    it('clicks element found by CSS selector', async () => {
      document.body.innerHTML = '<button id="btn">Submit</button>'
      const clicked = vi.fn()
      el('#btn').addEventListener('click', clicked)

      await loadBridge()
      const res = sendCommand({ id: '1', type: 'click', selector: '#btn' })

      expect(clicked).toHaveBeenCalledOnce()
      expect(res).toEqual({
        id: '1',
        type: 'result',
        value: expect.stringContaining('Clicked <button')
      })
    })

    it('returns error when selector finds nothing', async () => {
      await loadBridge()
      const res = sendCommand({ id: '2', type: 'click', selector: '#nonexistent' })
      expect(res).toEqual({
        id: '2',
        type: 'error',
        message: 'Element not found: #nonexistent'
      })
    })

    it('includes element description in response', async () => {
      document.body.innerHTML = '<button class="primary big">Save Changes</button>'
      await loadBridge()
      const res = sendCommand({ id: '3', type: 'click', selector: 'button.primary' })
      expect(res).toEqual({
        id: '3',
        type: 'result',
        value: 'Clicked <button class="primary big">Save Changes</button>'
      })
    })
  })

  describe('click by text', () => {
    it('finds and clicks element by exact text', async () => {
      document.body.innerHTML = '<div><span>Click me</span></div>'
      const span = el<HTMLElement>('span')
      makeVisible(span)
      const clicked = vi.fn()
      span.addEventListener('click', clicked)

      await loadBridge()
      const res = sendCommand({ id: '1', type: 'click', text: 'Click me' })

      expect(clicked).toHaveBeenCalledOnce()
      expect(res).toMatchObject({ id: '1', type: 'result' })
    })

    it('prefers exact match over partial', async () => {
      document.body.innerHTML = '<div><p>Click me now</p><span>Click me</span></div>'
      makeVisible(el<HTMLElement>('span'))
      makeVisible(el<HTMLElement>('p'))

      await loadBridge()
      const res = sendCommand({ id: '2', type: 'click', text: 'Click me' })
      expect(res).toEqual({
        id: '2',
        type: 'result',
        value: expect.stringContaining('<span')
      })
    })

    it('finds deepest matching element', async () => {
      document.body.innerHTML = '<div><p><span>Deep text</span></p></div>'
      makeVisible(el<HTMLElement>('span'))
      makeVisible(el<HTMLElement>('p'))
      makeVisible(el<HTMLElement>('div'))

      await loadBridge()
      const res = sendCommand({ id: '3', type: 'click', text: 'Deep text' })
      expect(res).toEqual({
        id: '3',
        type: 'result',
        value: expect.stringContaining('<span')
      })
    })

    it('skips invisible elements', async () => {
      document.body.innerHTML = '<div><span class="hidden">Target</span><b>Target</b></div>'
      makeVisible(el<HTMLElement>('b'))

      await loadBridge()
      const res = sendCommand({ id: '4', type: 'click', text: 'Target' })
      expect(res).toEqual({
        id: '4',
        type: 'result',
        value: expect.stringContaining('<b')
      })
    })

    it('returns error when text not found', async () => {
      document.body.innerHTML = '<div>Nothing here</div>'
      await loadBridge()
      const res = sendCommand({ id: '5', type: 'click', text: 'Missing text' })
      expect(res).toEqual({
        id: '5',
        type: 'error',
        message: 'Element not found: text="Missing text"'
      })
    })

    it('returns error when neither selector nor text provided', async () => {
      await loadBridge()
      const res = sendCommand({ id: '6', type: 'click' })
      expect(res).toEqual({
        id: '6',
        type: 'error',
        message: 'Element not found: text="undefined"'
      })
    })
  })

  describe('type', () => {
    it('types text into input element', async () => {
      document.body.innerHTML = '<input id="inp" value="" />'
      const input = el<HTMLInputElement>('#inp')

      await loadBridge()
      const res = sendCommand({ id: '1', type: 'type', selector: '#inp', text: 'hello' })

      expect(input.value).toBe('hello')
      expect(res).toEqual({
        id: '1',
        type: 'result',
        value: expect.stringContaining('Typed "hello" into <input')
      })
    })

    it('clears existing value by default', async () => {
      document.body.innerHTML = '<input id="inp" value="old" />'
      const input = el<HTMLInputElement>('#inp')
      const events: string[] = []
      input.addEventListener('input', () => events.push('input'))

      await loadBridge()
      sendCommand({ id: '2', type: 'type', selector: '#inp', text: 'new' })

      expect(input.value).toBe('new')
      expect(events.length).toBeGreaterThanOrEqual(2)
    })

    it('preserves existing value when clear=false', async () => {
      document.body.innerHTML = '<input id="inp" value="old" />'
      const input = el<HTMLInputElement>('#inp')
      const events: string[] = []
      input.addEventListener('input', () => events.push('input'))
      input.addEventListener('change', () => events.push('change'))

      await loadBridge()
      sendCommand({ id: '3', type: 'type', selector: '#inp', text: 'new', clear: false })

      expect(events).toEqual(['input', 'change'])
    })

    it('dispatches input and change events', async () => {
      document.body.innerHTML = '<input id="inp" />'
      const input = el<HTMLInputElement>('#inp')
      const events: string[] = []
      input.addEventListener('input', () => events.push('input'))
      input.addEventListener('change', () => events.push('change'))

      await loadBridge()
      sendCommand({ id: '4', type: 'type', selector: '#inp', text: 'test' })

      expect(events).toContain('input')
      expect(events).toContain('change')
    })

    it('works with textarea elements', async () => {
      document.body.innerHTML = '<textarea id="ta"></textarea>'
      const ta = el<HTMLTextAreaElement>('#ta')

      await loadBridge()
      sendCommand({ id: '5', type: 'type', selector: '#ta', text: 'multi\nline' })

      expect(ta.value).toBe('multi\nline')
    })

    it('returns error when element not found', async () => {
      await loadBridge()
      const res = sendCommand({ id: '6', type: 'type', selector: '#missing', text: 'test' })
      expect(res).toEqual({
        id: '6',
        type: 'error',
        message: 'Element not found: #missing'
      })
    })
  })

  describe('scroll', () => {
    function createScrollable(id: string) {
      const div = document.createElement('div')
      div.id = id
      let scrollTopVal = 0
      Object.defineProperties(div, {
        scrollHeight: { value: 1000, configurable: true },
        scrollTop: {
          get: () => scrollTopVal,
          set: (v: number) => {
            scrollTopVal = v
          },
          configurable: true
        }
      })
      div.scrollTo = vi.fn(({ top }: ScrollToOptions) => {
        scrollTopVal = top ?? 0
      }) as typeof div.scrollTo
      div.scrollBy = vi.fn(({ top }: ScrollToOptions) => {
        scrollTopVal += top ?? 0
      }) as typeof div.scrollBy
      document.body.appendChild(div)
      return div
    }

    it('scrolls by deltaY', async () => {
      createScrollable('scr')

      await loadBridge()
      const res = sendCommand({ id: '1', type: 'scroll', selector: '#scr', deltaY: 200 })

      expect(res).toEqual({ id: '1', type: 'result', value: 'scrollTop=200' })
    })

    it('uses default deltaY of 300', async () => {
      createScrollable('scr')

      await loadBridge()
      const res = sendCommand({ id: '2', type: 'scroll', selector: '#scr' })

      expect(res).toEqual({ id: '2', type: 'result', value: 'scrollTop=300' })
    })

    it('scrolls to top', async () => {
      const div = createScrollable('scr')
      div.scrollTop = 500

      await loadBridge()
      const res = sendCommand({ id: '3', type: 'scroll', selector: '#scr', to: 'top' })

      expect(res).toEqual({ id: '3', type: 'result', value: 'scrollTop=0' })
    })

    it('scrolls to bottom', async () => {
      createScrollable('scr')

      await loadBridge()
      const res = sendCommand({ id: '4', type: 'scroll', selector: '#scr', to: 'bottom' })

      expect(res).toEqual({ id: '4', type: 'result', value: 'scrollTop=1000' })
    })

    it('scrolls documentElement when no selector', async () => {
      let scrollTopVal = 0
      Object.defineProperty(document.documentElement, 'scrollHeight', { value: 2000, configurable: true })
      Object.defineProperty(document.documentElement, 'scrollTop', {
        get: () => scrollTopVal,
        set: (v: number) => {
          scrollTopVal = v
        },
        configurable: true
      })
      document.documentElement.scrollBy = vi.fn(({ top }: ScrollToOptions) => {
        scrollTopVal += top ?? 0
      }) as typeof document.documentElement.scrollBy

      await loadBridge()
      const res = sendCommand({ id: '5', type: 'scroll', deltaY: 150 })

      expect(res).toEqual({ id: '5', type: 'result', value: 'scrollTop=150' })
    })

    it('returns error when selector not found', async () => {
      await loadBridge()
      const res = sendCommand({ id: '6', type: 'scroll', selector: '#missing' })
      expect(res).toEqual({
        id: '6',
        type: 'error',
        message: 'Element not found: #missing'
      })
    })

    it('accumulates multiple scrollBy calls', async () => {
      createScrollable('scr')

      await loadBridge()
      sendCommand({ id: '7', type: 'scroll', selector: '#scr', deltaY: 100 })
      const res = sendCommand({ id: '8', type: 'scroll', selector: '#scr', deltaY: 100 })

      expect(res).toEqual({ id: '8', type: 'result', value: 'scrollTop=200' })
    })
  })

  describe('unknown commands', () => {
    it('returns error for unknown command type', async () => {
      await loadBridge()
      const res = sendCommand({ id: '1', type: 'bogus' })
      expect(res).toEqual({
        id: '1',
        type: 'error',
        message: 'Unknown command type: bogus'
      })
    })
  })

  describe('malformed messages', () => {
    it('ignores unparseable JSON', async () => {
      await loadBridge()
      const before = ws().sent.length
      ws().onmessage?.({ data: 'not json{{{' })
      expect(ws().sent).toHaveLength(before)
    })
  })
})
