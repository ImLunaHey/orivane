import { Link } from '@tanstack/react-router'
import type { ModuleDoc } from '../../content/types'

export function FeatureCard({ module }: { module: ModuleDoc }) {
  return <Link className="feature-card" to="/modules/$moduleId" params={{ moduleId: module.id }}>
    <span className="category-pill">{module.category}</span><h3>{module.title}</h3><p>{module.summary}</p><span className="card-link">Explore configuration →</span>
  </Link>
}
