import type { ReactNode } from 'react'

export function InfoPanel({ title, children, icon }: { title: string; children: ReactNode; icon?: ReactNode }) {
  return <div className="doc-panel">{icon}<h3>{title}</h3><p>{children}</p></div>
}
