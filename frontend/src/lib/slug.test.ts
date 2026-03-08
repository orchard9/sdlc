import { describe, it, expect } from 'vitest'
import { titleToSlug, nextIterationSlug } from './slug'

describe('titleToSlug', () => {
  it('converts a title to a slug', () => {
    expect(titleToSlug('Hello World')).toBe('hello-world')
  })

  it('strips special characters', () => {
    expect(titleToSlug('Feature: Add Login!')).toBe('feature-add-login')
  })

  it('collapses multiple hyphens', () => {
    expect(titleToSlug('a--b---c')).toBe('a-b-c')
  })
})

describe('nextIterationSlug', () => {
  it('returns -v2 when no existing versions', () => {
    expect(nextIterationSlug('foo', [])).toBe('foo-v2')
  })

  it('returns -v3 when v2 exists', () => {
    expect(nextIterationSlug('foo', ['foo-v2'])).toBe('foo-v3')
  })

  it('returns -v5 when v2, v3, v4 exist', () => {
    expect(nextIterationSlug('foo', ['foo-v2', 'foo-v3', 'foo-v4'])).toBe(
      'foo-v5',
    )
  })

  it('skips gaps and returns max+1', () => {
    expect(nextIterationSlug('foo', ['foo-v2', 'foo-v5'])).toBe('foo-v6')
  })

  it('strips -vN from input slug before computing', () => {
    expect(nextIterationSlug('foo-v2', ['foo-v2', 'foo-v3'])).toBe('foo-v4')
  })

  it('ignores unrelated slugs', () => {
    expect(
      nextIterationSlug('foo', ['bar-v2', 'baz-v3', 'foobar-v2']),
    ).toBe('foo-v2')
  })

  it('ignores the base slug without a version suffix', () => {
    expect(nextIterationSlug('foo', ['foo'])).toBe('foo-v2')
  })

  it('truncates result to 40 characters', () => {
    const longSlug = 'a-very-long-slug-name-that-exceeds-the-forty-char-limit'
    const result = nextIterationSlug(longSlug, [])
    expect(result.length).toBeLessThanOrEqual(40)
  })

  it('handles hyphenated base slugs correctly', () => {
    expect(
      nextIterationSlug('git-status-indicator', [
        'git-status-indicator-v2',
      ]),
    ).toBe('git-status-indicator-v3')
  })

  it('handles high version numbers', () => {
    expect(nextIterationSlug('foo', ['foo-v99'])).toBe('foo-v100')
  })
})
