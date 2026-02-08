import { AlertCircle, Check, Copy } from 'lucide-react'
import { Component, type ErrorInfo, type ReactNode } from 'react'
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'
import { Button } from '@/components/ui/button'

interface Props {
  children: ReactNode
}

interface State {
  error: Error | null
  componentStack: string | null
  copied: boolean
}

export class ErrorBoundary extends Component<Props, State> {
  state: State = { error: null, componentStack: null, copied: false }
  private copyTimer: ReturnType<typeof setTimeout> | null = null

  static getDerivedStateFromError(error: Error): Partial<State> {
    if (import.meta.env.DEV) throw error
    return { error }
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error('ErrorBoundary caught:', error, info.componentStack)
    this.setState({ componentStack: info.componentStack ?? null })
  }

  componentWillUnmount() {
    if (this.copyTimer) clearTimeout(this.copyTimer)
  }

  private handleCopy = () => {
    if (!this.state.error) return
    let text = `[Ralph Error R-9000] ${new Date().toISOString()}\n${this.state.error.message}`
    if (this.state.componentStack) {
      text += `\n\nComponent stack:\n${this.state.componentStack}`
    }
    navigator.clipboard.writeText(text)
    this.setState({ copied: true })
    this.copyTimer = setTimeout(() => this.setState({ copied: false }), 1500)
  }

  render() {
    if (this.state.error) {
      return (
        <div className="h-full flex items-center justify-center p-6">
          <Alert variant="destructive" className="max-w-md">
            <AlertCircle className="h-4 w-4" />
            <AlertTitle>Something went wrong</AlertTitle>
            <AlertDescription className="mt-2 space-y-3">
              <p className="text-xs font-mono break-all">{this.state.error.message}</p>
              <div className="flex gap-2">
                <Button
                  size="sm"
                  variant="outline"
                  onClick={() => this.setState({ error: null, componentStack: null })}>
                  Try again
                </Button>
                <Button size="sm" variant="outline" onClick={this.handleCopy}>
                  {this.state.copied ? <Check className="h-3 w-3 mr-1" /> : <Copy className="h-3 w-3 mr-1" />}
                  {this.state.copied ? 'Copied' : 'Copy error'}
                </Button>
              </div>
            </AlertDescription>
          </Alert>
        </div>
      )
    }
    return this.props.children
  }
}
