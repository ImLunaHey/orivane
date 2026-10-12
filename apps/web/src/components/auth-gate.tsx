import { Bot, LoaderCircle, ShieldCheck } from 'lucide-react'
import type { ReactNode } from 'react'
import { authClient } from '../lib/auth-client'
import { isDemoMode } from '../lib/api'

export function AuthGate({ children }: { children: ReactNode }) {
  const session = authClient.useSession()

  if (isDemoMode) return children
  if (session.isPending) {
    return <main className="auth-screen"><LoaderCircle className="spin" aria-hidden="true" /><p>Checking your session…</p></main>
  }
  if (session.data) return children

  return (
    <main className="auth-screen">
      <section className="auth-card" aria-labelledby="sign-in-title">
        <span className="auth-mark"><Bot size={28} aria-hidden="true" /></span>
        <p className="eyebrow">Orivane control centre</p>
        <h1 id="sign-in-title">Sign in to manage the bot</h1>
        <p>Discord is used to verify your identity and your Manage Server permissions. Bot credentials never enter the browser.</p>
        {session.error ? <div className="error-banner" role="alert">{session.error.message}</div> : null}
        <button className="discord-sign-in" onClick={() => void authClient.signIn.social({ provider: 'discord', callbackURL: window.location.href })}>
          <ShieldCheck size={19} aria-hidden="true" />Continue with Discord
        </button>
      </section>
    </main>
  )
}

