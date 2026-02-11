export {}

type Command =
  | { id: string; type: 'eval'; code: string }
  | { id: string; type: 'click'; selector?: string; text?: string }
  | { id: string; type: 'type'; selector: string; text: string; clear?: boolean }
  | { id: string; type: 'scroll'; selector?: string; deltaY?: number; to?: 'top' | 'bottom' }

type BridgeResponse = { id: string; type: 'result'; value: unknown } | { id: string; type: 'error'; message: string }

function ok(id: string, value: unknown): BridgeResponse {
  return { id, type: 'result', value }
}

function err(id: string, message: string): BridgeResponse {
  return { id, type: 'error', message }
}

function getDepth(el: Element): number {
  let depth = 0
  let parent = el.parentElement
  while (parent) {
    depth++
    parent = parent.parentElement
  }
  return depth
}

function isVisible(el: HTMLElement): boolean {
  return el.offsetParent !== null || el.tagName === 'BODY' || el.tagName === 'HTML'
}

function findByText(searchText: string): Element | null {
  const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_ELEMENT)
  let best: Element | null = null
  let bestDepth = -1
  let bestIsExact = false

  while (walker.nextNode()) {
    const el = walker.currentNode as HTMLElement
    if (!isVisible(el)) continue

    const content = el.textContent?.trim() ?? ''
    if (!content.includes(searchText)) continue

    const depth = getDepth(el)
    const isExact = content === searchText

    if (isExact && (!bestIsExact || depth > bestDepth)) {
      best = el
      bestDepth = depth
      bestIsExact = true
    } else if (!bestIsExact && depth > bestDepth) {
      best = el
      bestDepth = depth
    }
  }

  return best
}

function describeElement(el: Element): string {
  const tag = el.tagName.toLowerCase()
  const cls = el.className ? ` class="${String(el.className).slice(0, 60)}"` : ''
  const text = el.textContent?.trim().slice(0, 50) ?? ''
  return `<${tag}${cls}>${text}</${tag}>`
}

function handleEval(cmd: Extract<Command, { type: 'eval' }>): BridgeResponse {
  let result: unknown
  try {
    result = new Function(`return (${cmd.code})`)()
  } catch {
    result = new Function(cmd.code)()
  }
  return ok(cmd.id, typeof result === 'undefined' ? 'undefined' : result)
}

function handleClick(cmd: Extract<Command, { type: 'click' }>): BridgeResponse {
  const el = cmd.selector ? document.querySelector(cmd.selector) : cmd.text ? findByText(cmd.text) : null
  if (!el) return err(cmd.id, `Element not found: ${cmd.selector ?? `text="${cmd.text}"`}`)
  ;(el as HTMLElement).click()
  return ok(cmd.id, `Clicked ${describeElement(el)}`)
}

function handleType(cmd: Extract<Command, { type: 'type' }>): BridgeResponse {
  const el = document.querySelector(cmd.selector) as HTMLInputElement | HTMLTextAreaElement | null
  if (!el) return err(cmd.id, `Element not found: ${cmd.selector}`)
  if (cmd.clear !== false) {
    el.value = ''
    el.dispatchEvent(new Event('input', { bubbles: true }))
  }
  const nativeSetter = Object.getOwnPropertyDescriptor(Object.getPrototypeOf(el), 'value')?.set
  if (nativeSetter) {
    nativeSetter.call(el, cmd.text)
  } else {
    el.value = cmd.text
  }
  el.dispatchEvent(new Event('input', { bubbles: true }))
  el.dispatchEvent(new Event('change', { bubbles: true }))
  return ok(cmd.id, `Typed "${cmd.text}" into ${describeElement(el)}`)
}

function handleScroll(cmd: Extract<Command, { type: 'scroll' }>): BridgeResponse {
  const el = cmd.selector ? document.querySelector(cmd.selector) : document.documentElement
  if (!el) return err(cmd.id, `Element not found: ${cmd.selector}`)
  if (cmd.to === 'top') {
    el.scrollTo({ top: 0 })
  } else if (cmd.to === 'bottom') {
    el.scrollTo({ top: el.scrollHeight })
  } else {
    el.scrollBy({ top: cmd.deltaY ?? 300 })
  }
  return ok(cmd.id, `scrollTop=${el.scrollTop}`)
}

function handleCommand(cmd: Command): BridgeResponse {
  try {
    switch (cmd.type) {
      case 'eval':
        return handleEval(cmd)
      case 'click':
        return handleClick(cmd)
      case 'type':
        return handleType(cmd)
      case 'scroll':
        return handleScroll(cmd)
      default: {
        const unknown = cmd as { id: string; type: string }
        return err(unknown.id, `Unknown command type: ${unknown.type}`)
      }
    }
  } catch (e) {
    const id = (cmd as { id: string }).id
    return err(id, e instanceof Error ? e.message : String(e))
  }
}

function connect() {
  const ws = new WebSocket('ws://localhost:9223')

  ws.onopen = () => {
    console.log('[dev-bridge] connected')
  }

  ws.onmessage = event => {
    let cmd: Command
    try {
      cmd = JSON.parse(String(event.data))
    } catch {
      return
    }
    const response = handleCommand(cmd)
    ws.send(JSON.stringify(response))
  }

  ws.onclose = () => {
    console.log('[dev-bridge] disconnected, reconnecting in 2s')
    setTimeout(connect, 2000)
  }

  ws.onerror = () => {
    ws.close()
  }
}

connect()
