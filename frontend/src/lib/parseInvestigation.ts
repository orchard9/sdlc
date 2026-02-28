import type { AreaArtifactMeta } from './types'

const AREA_NAMES: Record<string, string> = {
  'area-1': 'code_paths',
  'area-2': 'bottlenecks',
  'area-3': 'data_flow',
  'area-4': 'auth_chain',
  'area-5': 'environment',
}

/** Map a filename like "area-1-code-paths.md" to its area key "code_paths". */
export function areaKeyFromFilename(filename: string): string | null {
  const match = filename.match(/^(area-\d+)/)
  if (!match) return null
  return AREA_NAMES[match[1]] ?? null
}

/** Parse YAML-ish frontmatter from area artifact content.
 *
 * Expected format:
 * ---
 * area: code_paths
 * status: finding
 * confidence: 72
 * ---
 * One-line finding here.
 */
export function parseAreaArtifact(filename: string, content: string): AreaArtifactMeta | null {
  const areaKey = areaKeyFromFilename(filename)
  if (!areaKey) return null

  // Extract frontmatter block
  const fmMatch = content.match(/^---\r?\n([\s\S]*?)\r?\n---/)
  const meta: Partial<AreaArtifactMeta> = { area: areaKey, status: 'pending' }

  if (fmMatch) {
    const fm = fmMatch[1]
    const statusMatch = fm.match(/^status:\s*(.+)$/m)
    if (statusMatch) {
      const s = statusMatch[1].trim() as AreaArtifactMeta['status']
      if (['pending', 'investigating', 'finding', 'hypothesis'].includes(s)) {
        meta.status = s
      }
    }
    const confMatch = fm.match(/^confidence:\s*(\d+)$/m)
    if (confMatch) meta.confidence = parseInt(confMatch[1], 10)

    // First non-empty line after frontmatter as the finding
    const afterFm = content.slice(fmMatch[0].length).trimStart()
    const firstLine = afterFm.split('\n')[0].trim()
    if (firstLine) meta.finding = firstLine
  } else {
    // No frontmatter — treat first line as finding, status stays pending
    const firstLine = content.split('\n')[0].trim()
    if (firstLine) meta.finding = firstLine
  }

  return meta as AreaArtifactMeta
}
