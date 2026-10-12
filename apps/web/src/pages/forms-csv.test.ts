import { describe, expect, it } from 'vitest'
import { submissionsCsv } from './forms'

describe('form submission exports', () => {
  it('quotes fields and prevents spreadsheet formula injection', () => {
    const csv = submissionsCsv([{
      id: 1,
      guild_id: '10',
      form_id: 'apply',
      submitter_id: '20',
      username: '=IMPORTXML("https://example.com")',
      display_name: 'Luna, Admin',
      answers: [{ question_id: 'why', label: 'Why?', answer: '"Because"' }],
      status: 'submitted',
      reviewer_id: null,
      review_note: null,
      reviewed_at: null,
      created_at: '2026-10-12T12:00:00Z',
    }])
    expect(csv).toContain("'=IMPORTXML")
    expect(csv).toContain('"Luna, Admin"')
    expect(csv).toContain('""answer""')
    expect(csv).toContain('Because')
  })
})
