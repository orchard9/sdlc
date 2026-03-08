import { describe, it, expect } from 'vitest'
import { QUOTES, getWeeklyQuote, type Quote } from './quotes'

const MS_PER_WEEK = 7 * 24 * 60 * 60 * 1000

describe('QUOTES corpus', () => {
  it('contains at least 12 entries', () => {
    expect(QUOTES.length).toBeGreaterThanOrEqual(12)
  })

  it('every entry has non-empty text and author', () => {
    for (const q of QUOTES) {
      expect(q.text.trim().length).toBeGreaterThan(0)
      expect(q.author.trim().length).toBeGreaterThan(0)
    }
  })
})

describe('getWeeklyQuote', () => {
  it('returns the same quote for the same timestamp', () => {
    const now = Date.now()
    const a = getWeeklyQuote(QUOTES, now)
    const b = getWeeklyQuote(QUOTES, now)
    expect(a).toEqual(b)
  })

  it('returns the same quote for two times within the same week window', () => {
    const base = 100 * MS_PER_WEEK // pick a clean week boundary
    const a = getWeeklyQuote(QUOTES, base + 1000)
    const b = getWeeklyQuote(QUOTES, base + MS_PER_WEEK - 1000)
    expect(a).toEqual(b)
  })

  it('returns a different quote after exactly one week (given enough quotes)', () => {
    const base = 100 * MS_PER_WEEK
    const a = getWeeklyQuote(QUOTES, base)
    const b = getWeeklyQuote(QUOTES, base + MS_PER_WEEK)
    // With 16 quotes the indices differ by 1, so quotes should differ
    expect(a).not.toEqual(b)
  })

  it('cycles back after quotes.length weeks', () => {
    const corpus: Quote[] = [
      { text: 'A', author: 'X' },
      { text: 'B', author: 'Y' },
    ]
    const base = 0
    const first = getWeeklyQuote(corpus, base)
    const cycled = getWeeklyQuote(corpus, base + 2 * MS_PER_WEEK)
    expect(cycled).toEqual(first)
  })

  it('accepts a custom corpus', () => {
    const corpus: Quote[] = [{ text: 'Only one', author: 'Solo' }]
    const q = getWeeklyQuote(corpus)
    expect(q.text).toBe('Only one')
  })
})
