import { Link } from '@tanstack/react-router'
import { FileQuestion } from 'lucide-react'

export function NotFoundPage() {
  return <div className="not-found"><FileQuestion size={34} /><p className="eyebrow">404</p><h1>Documentation page not found</h1><p>The route may have moved, or the module name is not part of this release.</p><Link className="button primary" to="/modules">Browse every module</Link></div>
}
