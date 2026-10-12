import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { RouterProvider } from '@tanstack/react-router'
import { act, cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { router } from '../router'

const serverId='901234567890123456'
function renderDashboard(){const client=new QueryClient({defaultOptions:{queries:{retry:false},mutations:{retry:false}}});return render(<QueryClientProvider client={client}><RouterProvider router={router}/></QueryClientProvider>)}
async function navigate(){const path=`/servers/${serverId}/modules/economy`;await act(async()=>{await router.navigate({to:path as never})});await waitFor(()=>expect(router.state.location.pathname).toBe(path))}

describe('economy dashboard',()=>{
  beforeEach(()=>localStorage.clear());afterEach(()=>cleanup())
  it('configures currency, games, and shop rewards and saves at the bottom',async()=>{
    renderDashboard();await navigate();await screen.findByRole('heading',{name:/Economy and games/})
    expect(screen.getByRole('table',{name:'Economy member accounts'})).toBeVisible()
    expect(screen.getByRole('table',{name:'Economy transaction ledger'})).toBeVisible()
    fireEvent.change(screen.getByLabelText(/^Currency name/),{target:{value:'moons'}})
    fireEvent.change(screen.getByLabelText(/^Daily reward/),{target:{value:'300'}})
    fireEvent.change(screen.getByLabelText(/^Maximum wager/),{target:{value:'50000'}})
    fireEvent.click(screen.getByRole('switch',{name:'Enable economy games'}))
    fireEvent.click(screen.getByRole('button',{name:/Add item/}))
    expect(screen.getByText('New reward')).toBeVisible()
    fireEvent.change(screen.getByLabelText(/^Price/),{target:{value:'750'}})
    fireEvent.click(screen.getByRole('button',{name:'Save changes'}))
    await screen.findByText('Saved')
    const saved=JSON.parse(localStorage.getItem(`orivane:config:${serverId}`)??'{}').modules.economy
    expect(saved).toMatchObject({currency_name:'moons',daily_amount:300,maximum_wager:50000,games_enabled:false})
    expect(saved.shop_items[0].price).toBe(750)
    expect(saved.shop_items).toHaveLength(1)
  })
})
