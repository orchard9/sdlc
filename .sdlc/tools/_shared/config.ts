/**
 * SDLC Tool Config Loader
 *
 * Reads .sdlc/tools/<name>/config.yaml. If the file is missing or unparseable,
 * returns the provided defaults — tools should never hard-fail on missing config.
 *
 * Supports flat key: value YAML only. Arrays and nested objects are intentionally
 * not supported — keep tool configs simple scalars.
 */
import { readFileSync } from 'node:fs'
import { join } from 'node:path'

export function loadToolConfig<T extends Record<string, unknown>>(
  root: string,
  toolName: string,
  defaults: T,
): T {
  const configPath = join(root, '.sdlc', 'tools', toolName, 'config.yaml')
  try {
    const raw = readFileSync(configPath, 'utf8')
    const parsed = parseSimpleYaml(raw)
    return { ...defaults, ...parsed } as T
  } catch {
    return defaults
  }
}

/** Parse a flat key: value YAML file. Skips blank lines, comments, and array items. */
function parseSimpleYaml(content: string): Record<string, unknown> {
  const result: Record<string, unknown> = {}
  for (const line of content.split('\n')) {
    const trimmed = line.trim()
    if (!trimmed || trimmed.startsWith('#') || trimmed.startsWith('-')) continue
    const colonIdx = trimmed.indexOf(':')
    if (colonIdx === -1) continue
    const key = trimmed.slice(0, colonIdx).trim()
    const rawValue = trimmed.slice(colonIdx + 1).trim()
    if (!key || !rawValue) continue
    const value = rawValue.replace(/^["'](.*)["']$/, '$1')
    const num = Number(value)
    result[key] = Number.isNaN(num) ? value : num
  }
  return result
}
