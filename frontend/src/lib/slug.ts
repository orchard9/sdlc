/**
 * Converts a human-readable title string into a URL-safe slug.
 * Lowercases, strips non-alphanumeric characters, collapses whitespace
 * and hyphens, and trims leading/trailing hyphens.
 */
export function titleToSlug(title: string): string {
  return title
    .toLowerCase()
    .replace(/[^a-z0-9\s-]/g, '')
    .replace(/\s+/g, '-')
    .replace(/-+/g, '-')
    .replace(/^-|-$/g, '')
}

/** Escape special regex characters in a string. */
function escapeRegex(s: string): string {
  return s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
}

/**
 * Given a base slug (possibly already versioned), compute the next
 * iteration slug by scanning existing slugs for version collisions.
 *
 * Examples:
 *   nextIterationSlug('foo', [])                        → 'foo-v2'
 *   nextIterationSlug('foo', ['foo-v2'])                → 'foo-v3'
 *   nextIterationSlug('foo-v2', ['foo-v2', 'foo-v3'])   → 'foo-v4'
 */
export function nextIterationSlug(
  baseSlug: string,
  existingSlugs: string[],
): string {
  const base = baseSlug.replace(/-v\d+$/, '')
  const pattern = new RegExp(`^${escapeRegex(base)}-v(\\d+)$`)

  let maxVersion = 1 // the original (unversioned) slug is implicitly v1
  for (const s of existingSlugs) {
    const match = s.match(pattern)
    if (match) {
      maxVersion = Math.max(maxVersion, parseInt(match[1], 10))
    }
  }

  return `${base}-v${maxVersion + 1}`.slice(0, 40)
}
