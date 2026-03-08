import { describe, it, expect } from 'vitest'
import { nextIterationSlug } from './iterateSlug'

describe('nextIterationSlug', () => {
  it('appends -v2 when no existing versions', () => {
    expect(nextIterationSlug('foo', [])).toBe('foo-v2')
  })

  it('appends -v2 when existing slugs have no versions of the same root', () => {
    expect(nextIterationSlug('foo', ['bar', 'baz-v2'])).toBe('foo-v2')
  })

  it('increments past existing -v2', () => {
    expect(nextIterationSlug('foo', ['foo-v2'])).toBe('foo-v3')
  })

  it('increments past the highest existing version', () => {
    expect(nextIterationSlug('foo', ['foo-v2', 'foo-v3', 'foo-v5'])).toBe('foo-v6')
  })

  it('strips -vN suffix from input slug before generating', () => {
    expect(nextIterationSlug('foo-v2', ['foo-v2'])).toBe('foo-v3')
  })

  it('strips -vN suffix and finds highest version', () => {
    expect(nextIterationSlug('foo-v2', ['foo-v2', 'foo-v3'])).toBe('foo-v4')
  })

  it('handles slug with no -vN and no collisions', () => {
    expect(nextIterationSlug('my-feature', [])).toBe('my-feature-v2')
  })

  it('handles slug ending with -v but no number (not a version suffix)', () => {
    // "-v" without a digit is not a version suffix, so root stays "thing-v"
    expect(nextIterationSlug('thing-v', [])).toBe('thing-v-v2')
  })

  it('does not match partial root collisions', () => {
    // "foo-bar-v2" should not match root "foo"
    expect(nextIterationSlug('foo', ['foo-bar-v2'])).toBe('foo-v2')
  })

  it('handles complex slug with special chars safely', () => {
    expect(nextIterationSlug('my.feature', ['my.feature-v2'])).toBe('my.feature-v3')
  })
})
