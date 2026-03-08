/**
 * Generate the next versioned iteration slug from a base slug.
 *
 * Examples:
 *   nextIterationSlug("foo", [])               → "foo-v2"
 *   nextIterationSlug("foo", ["foo-v2"])        → "foo-v3"
 *   nextIterationSlug("foo-v2", ["foo-v2"])     → "foo-v3"
 *   nextIterationSlug("foo-v3", ["foo-v3"])     → "foo-v4"
 *   nextIterationSlug("foo", ["foo-v2","foo-v3"]) → "foo-v4"
 */
export function nextIterationSlug(baseSlug: string, existingSlugs: string[]): string {
  // Strip any existing -vN suffix to get the root slug
  const root = baseSlug.replace(/-v\d+$/, '')

  // Find the highest existing version number for this root
  const versionPattern = new RegExp(`^${escapeRegex(root)}-v(\\d+)$`)
  let maxVersion = 1 // base slug without -vN is implicitly v1

  for (const slug of existingSlugs) {
    const match = slug.match(versionPattern)
    if (match) {
      const ver = parseInt(match[1], 10)
      if (ver > maxVersion) {
        maxVersion = ver
      }
    }
  }

  return `${root}-v${maxVersion + 1}`
}

function escapeRegex(str: string): string {
  return str.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
}
