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
