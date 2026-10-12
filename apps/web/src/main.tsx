import { QueryClientProvider } from '@tanstack/react-query'
import { RouterProvider } from '@tanstack/react-router'
import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { DashboardDevtools } from './components/devtools'
import { queryClient } from './lib/query'
import { router } from './router'
import './styles.css'

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
      {import.meta.env.DEV ? <DashboardDevtools /> : null}
    </QueryClientProvider>
  </StrictMode>,
)
