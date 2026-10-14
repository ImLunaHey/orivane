import { Link } from '@tanstack/react-router'
import { moduleCategories, modules } from '../../content/modules'

export function ModuleNav() {
  return <div className="module-nav">
    {moduleCategories.map((category) => <details open key={category}>
      <summary>{category}</summary>
      <div>{modules.filter((module) => module.category === category).map((module) =>
        <Link key={module.id} to="/modules/$moduleId" params={{ moduleId: module.id }} activeProps={{ 'data-active': true }}>{module.title}</Link>
      )}</div>
    </details>)}
  </div>
}
