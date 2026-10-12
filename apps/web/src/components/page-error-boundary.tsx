import { Component, type ErrorInfo, type ReactNode } from 'react'
import { AlertTriangle, RotateCcw } from 'lucide-react'
import { Button } from './ui'

type Props = { children: ReactNode }
type State = { error: Error | null }

export class PageErrorBoundary extends Component<Props, State> {
  state: State = { error: null }

  static getDerivedStateFromError(error: Error): State {
    return { error }
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error('Page content crashed', error, info)
  }

  render() {
    if (!this.state.error) return this.props.children
    return <section className="page-error" role="alert">
      <AlertTriangle size={28} />
      <div><p className="eyebrow">Page error</p><h2>This page could not be displayed</h2><p>The rest of the dashboard is still available. Retry this page, or use the navigation to go somewhere else.</p><details><summary>Technical details</summary><code>{this.state.error.message}</code></details></div>
      <Button onClick={() => this.setState({ error: null })}><RotateCcw size={16} />Retry page</Button>
    </section>
  }
}
