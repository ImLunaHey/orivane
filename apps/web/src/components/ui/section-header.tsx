import type { ReactNode } from 'react'

export function SectionHeader({ eyebrow, title, detail, action, className = '' }: { eyebrow: string; title: string; detail?: string; action?: ReactNode; className?: string }) {
  return <header className={`section-header ${className}`}><div><p className="eyebrow">{eyebrow}</p><h3>{title}</h3>{detail && <small>{detail}</small>}</div>{action && <div className="section-header-action">{action}</div>}</header>
}
