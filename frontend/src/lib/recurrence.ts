/**
 * Recurrence string utilities for the orchestrator Actions page.
 *
 * Accepted format: `<number><unit>` where unit is one of:
 *   s = seconds, m = minutes, h = hours, d = days
 *
 * Examples: "10s", "30m", "1h", "2d"
 */

const UNITS: Record<string, number> = { s: 1, m: 60, h: 3600, d: 86400 }
const PATTERN = /^(\d+)(s|m|h|d)$/

/**
 * Parse a human-readable recurrence string into seconds.
 * Returns `null` if the string does not match the accepted format.
 */
export function parseRecurrence(s: string): number | null {
  const m = s.trim().match(PATTERN)
  if (!m) return null
  const n = parseInt(m[1], 10)
  return n * UNITS[m[2]]
}

/**
 * Format a recurrence duration (in seconds) as a human-readable string.
 * Chooses the largest unit that divides evenly without remainder.
 *
 * Examples: 86400 → "1d", 3600 → "1h", 60 → "1m", 10 → "10s", 3601 → "3601s"
 */
export function formatRecurrence(secs: number): string {
  if (secs % 86400 === 0) return `${secs / 86400}d`
  if (secs % 3600 === 0) return `${secs / 3600}h`
  if (secs % 60 === 0) return `${secs / 60}m`
  return `${secs}s`
}
