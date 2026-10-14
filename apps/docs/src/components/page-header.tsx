import type { ReactNode } from 'react'

export function PageHeader({ eyebrow, title, description, aside }: { eyebrow: string; title: string; description: string; aside?: ReactNode }) {
  return <header className="page-header">
    <div><p className="eyebrow">{eyebrow}</p><h1>{title}</h1><p className="page-lede">{description}</p></div>
    {aside ? <div className="page-header-aside">{aside}</div> : null}
  </header>
}
