import type { ButtonHTMLAttributes, HTMLAttributes, ReactNode } from 'react'

export function Panel({ children, className = '', ...props }: { children: ReactNode; className?: string } & HTMLAttributes<HTMLElement>) {
  return <section className={`panel ${className}`} {...props}>{children}</section>
}

export function SectionHeader({ eyebrow, title, detail, action, className = '' }: { eyebrow: string; title: string; detail?: string; action?: ReactNode; className?: string }) {
  return <header className={`section-header ${className}`}>
    <div><p className="eyebrow">{eyebrow}</p><h3>{title}</h3>{detail && <small>{detail}</small>}</div>
    {action && <div className="section-header-action">{action}</div>}
  </header>
}

export function Badge({ children, tone = 'neutral' }: { children: ReactNode; tone?: 'neutral' | 'green' | 'amber' | 'red' | 'violet' }) {
  return <span className={`badge badge-${tone}`}>{children}</span>
}

export function Button({ className = '', ...props }: ButtonHTMLAttributes<HTMLButtonElement>) {
  return <button className={`button ${className}`} {...props} />
}

export function Toggle({ checked, onChange, label }: { checked: boolean; onChange: (checked: boolean) => void; label: string }) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      aria-label={label}
      className={`toggle ${checked ? 'toggle-on' : ''}`}
      onClick={() => onChange(!checked)}
    >
      <span />
    </button>
  )
}
