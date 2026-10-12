import { createAuthClient } from 'better-auth/react'

const configuredApiBase = import.meta.env.VITE_API_BASE_URL as string | undefined
const apiBase = (configuredApiBase === 'same-origin' ? window.location.origin : configuredApiBase)?.replace(/\/$/, '')

export const authClient = createAuthClient({
  baseURL: apiBase,
  fetchOptions: { credentials: 'include' },
})

