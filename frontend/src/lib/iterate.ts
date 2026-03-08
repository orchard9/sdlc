/**
 * Compute the next versioned iteration slug, avoiding collisions with existing slugs.
 *
 * Examples:
 *   nextIterationSlug("foo", [])                    -> "foo-v2"
 *   nextIterationSlug("foo", ["foo-v2"])             -> "foo-v3"
 *   nextIterationSlug("foo-v2", ["foo-v2", "foo-v3"]) -> "foo-v4"
 */
export function nextIterationSlug(slug: string, existingSlugs: string[]): string {
  const base = slug.replace(/-v\d+$/, '')
  const pattern = new RegExp(`^${base}-v(\\d+)$`)
  let maxVersion = 1
  for (const s of existingSlugs) {
    const match = s.match(pattern)
    if (match) {
      maxVersion = Math.max(maxVersion, parseInt(match[1], 10))
    }
  }
  return `${base}-v${maxVersion + 1}`.slice(0, 40)
}
