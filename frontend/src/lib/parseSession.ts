/**
 * parseSession — convert raw session Markdown into typed SessionEvent[].
 *
 * The session file format (written by the /sdlc-ponder agent):
 *
 *   ---
 *   session: 1
 *   timestamp: 2024-01-15T10:32:00Z
 *   orientation:
 *     current: Early discovery...
 *     next: Research...
 *     commit: When...
 *   ---
 *
 *   Seed text...
 *
 *   <!-- tool: read crates/foo.rs -->
 *   Found: no session primitive.
 *   <!-- /tool -->
 *
 *   **KAI · Systems Architect**
 *   The problem isn't storage...
 *
 *   Recruited: **JORDAN · PM / Skeptic**
 *
 *   ⚑ Decided: investigate failure modes first
 *   ? Open: is "memory" the right frame?
 *
 *   ```
 *   Option A: Decision Graph
 *   [decision: auth] → [result]
 *   ```
 */

export type SessionEvent =
  | { kind: 'tool'; tool: string; summary: string }
  | { kind: 'artifact'; filename: string; summary: string }
  | { kind: 'partner'; name: string; role: string; content: string }
  | { kind: 'recruited'; name: string; role: string }
  | { kind: 'decision'; content: string }
  | { kind: 'question'; content: string }
  | { kind: 'sketch'; content: string }
  | { kind: 'narrative'; content: string }

// Regex patterns
const ARTIFACT_TOOL = /^(sdlc_write_artifact|sdlc_ponder_capture|capture_artifact|write_artifact)(?:\s+(.+))?$/i
const PARTNER_HEADER = /^\*\*([A-Z][A-Z\s]+)\s*[·•·]\s*(.+?)\*\*\s*$/
const RECRUITED = /^Recruited:\s*\*\*([A-Z][A-Z\s]+)\s*[·•·]\s*(.+?)\*\*\s*$/i
const DECISION = /^⚑\s+(.+)$/
const QUESTION = /^\?\s+(.+)$/
const TOOL_OPEN = /^<!--\s*tool:\s*(.+?)\s*-->$/
const TOOL_CLOSE = /^<!--\s*\/tool\s*-->$/
const ARTIFACT_OPEN = /^<!--\s*artifact:\s*(.+?)\s*-->$/
const ARTIFACT_CLOSE = /^<!--\s*\/artifact\s*-->$/
const FENCE = /^```/
// ASCII box-drawing characters that mark a sketch block
const BOX_DRAWING = /[─│┌└┐┘├┤┬┴┼╔╗╚╝╠╣╦╩╬═║→←↑↓▸▶◦•]/

/** Strip YAML frontmatter fences from raw session content. */
function stripFrontmatter(raw: string): string {
  const trimmed = raw.trimStart()
  if (!trimmed.startsWith('---')) return raw
  const endIdx = trimmed.indexOf('\n---', 3)
  if (endIdx === -1) return raw
  return trimmed.slice(endIdx + 4).trimStart()
}

export function parseSession(raw: string): SessionEvent[] {
  const body = stripFrontmatter(raw)
  const lines = body.split('\n')
  const events: SessionEvent[] = []

  let i = 0

  while (i < lines.length) {
    const line = lines[i]

    // --- Explicit artifact comment block ---
    // <!-- artifact: filename.md --> ... summary ... <!-- /artifact -->
    const artifactCommentMatch = line.match(ARTIFACT_OPEN)
    if (artifactCommentMatch) {
      const filename = artifactCommentMatch[1].trim()
      const summaryLines: string[] = []
      i++
      while (i < lines.length && !lines[i].match(ARTIFACT_CLOSE)) {
        summaryLines.push(lines[i])
        i++
      }
      i++ // skip <!-- /artifact -->
      events.push({ kind: 'artifact', filename, summary: summaryLines.join('\n').trim() })
      continue
    }

    // --- Tool block ---
    const toolMatch = line.match(TOOL_OPEN)
    if (toolMatch) {
      const tool = toolMatch[1]
      const summaryLines: string[] = []
      i++
      while (i < lines.length && !lines[i].match(TOOL_CLOSE)) {
        summaryLines.push(lines[i])
        i++
      }
      i++ // skip <!-- /tool -->
      const summary = summaryLines.join('\n').trim()
      const artifactMatch = tool.match(ARTIFACT_TOOL)
      if (artifactMatch) {
        const filename = artifactMatch[2]?.trim() || 'artifact'
        events.push({ kind: 'artifact', filename, summary })
      } else {
        events.push({ kind: 'tool', tool, summary })
      }
      continue
    }

    // --- Recruited event ---
    const recruitedMatch = line.match(RECRUITED)
    if (recruitedMatch) {
      events.push({ kind: 'recruited', name: recruitedMatch[1].trim(), role: recruitedMatch[2].trim() })
      i++
      continue
    }

    // --- Partner message header ---
    const partnerMatch = line.match(PARTNER_HEADER)
    if (partnerMatch) {
      const name = partnerMatch[1].trim()
      const role = partnerMatch[2].trim()
      const contentLines: string[] = []
      i++
      // Collect lines until next header pattern or blank+header
      while (i < lines.length) {
        const next = lines[i]
        if (next.match(PARTNER_HEADER) || next.match(RECRUITED)) break
        if (next.match(TOOL_OPEN)) break
        if (next.match(DECISION) || next.match(QUESTION)) break
        contentLines.push(next)
        i++
      }
      const content = contentLines.join('\n').trim()
      if (content) {
        events.push({ kind: 'partner', name, role, content })
      }
      continue
    }

    // --- Decision ---
    const decisionMatch = line.match(DECISION)
    if (decisionMatch) {
      events.push({ kind: 'decision', content: decisionMatch[1].trim() })
      i++
      continue
    }

    // --- Question ---
    const questionMatch = line.match(QUESTION)
    if (questionMatch) {
      events.push({ kind: 'question', content: questionMatch[1].trim() })
      i++
      continue
    }

    // --- Code / Sketch block ---
    if (line.match(FENCE)) {
      const fenceLines: string[] = []
      i++
      while (i < lines.length && !lines[i].match(FENCE)) {
        fenceLines.push(lines[i])
        i++
      }
      i++ // skip closing fence
      const content = fenceLines.join('\n')
      const isSketch = BOX_DRAWING.test(content)
      events.push(isSketch ? { kind: 'sketch', content } : { kind: 'narrative', content: '```\n' + content + '\n```' })
      continue
    }

    // --- Narrative ---
    // Accumulate consecutive non-special lines
    if (line.trim()) {
      const narrativeLines: string[] = [line]
      i++
      while (i < lines.length) {
        const next = lines[i]
        if (!next.trim()) { i++; break }
        if (
          next.match(PARTNER_HEADER) ||
          next.match(RECRUITED) ||
          next.match(TOOL_OPEN) ||
          next.match(DECISION) ||
          next.match(QUESTION) ||
          next.match(FENCE)
        ) break
        narrativeLines.push(next)
        i++
      }
      events.push({ kind: 'narrative', content: narrativeLines.join('\n') })
      continue
    }

    // blank line — skip
    i++
  }

  return events
}
