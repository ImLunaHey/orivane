import { beforeEach, describe, expect, it, vi } from 'vitest'

vi.mock('./api', () => ({ recordDashboardActivity: vi.fn() }))

import { recordDashboardActivity } from './api'
import { downloadCsv } from './export'

describe('CSV export', () => {
  beforeEach(() => {
    vi.mocked(recordDashboardActivity).mockClear()
    vi.stubGlobal('URL', {
      ...URL,
      createObjectURL: vi.fn(() => 'blob:test'),
      revokeObjectURL: vi.fn(),
    })
  })

  it('escapes mixed values, downloads the file, and audits the export', () => {
    const click = vi.spyOn(HTMLAnchorElement.prototype, 'click').mockImplementation(() => undefined)
    const blob = vi.spyOn(globalThis, 'Blob')

    downloadCsv('audit.csv', [
      { name: 'Luna "hey"', count: 2, metadata: { ok: true } },
      { name: null, count: 0, metadata: null },
    ], 'guild-test')

    expect(click).toHaveBeenCalledOnce()
    expect(blob).toHaveBeenCalledWith([
      '"name","count","metadata"\n"Luna ""hey""","2","{""ok"":true}"\n"","0",""',
    ], { type: 'text/csv;charset=utf-8' })
    expect(URL.revokeObjectURL).toHaveBeenCalledWith('blob:test')
    expect(recordDashboardActivity).toHaveBeenCalledWith(expect.objectContaining({
      action: 'dashboard.data.exported',
      guild_id: 'guild-test',
      export: 'audit.csv',
    }))
  })
})
