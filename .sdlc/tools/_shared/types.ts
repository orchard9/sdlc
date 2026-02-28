/**
 * SDLC Tool Shared Interface
 *
 * Every SDLC tool imports from this file. It defines the full type contract
 * that tools must satisfy. Do not change the shape of these types without
 * updating all core tools and regenerating tools.md.
 *
 * Tool protocol (stdin/stdout):
 * - --meta   No stdin. Writes ToolMeta JSON to stdout.
 * - --run    Reads JSON from stdin. Writes ToolResult JSON to stdout. Exit 0 ok, 1 error.
 * - --setup  No stdin. Writes ToolResult JSON to stdout. Exit 0 ok, 1 error.
 *
 * All log output goes to STDERR. STDOUT is reserved for JSON only.
 */

/** Metadata describing a tool — returned by --meta mode. */
export interface ToolMeta {
  /** Matches the directory name exactly (e.g. "ama", "quality-check") */
  name: string
  /** Human-readable title shown in the tools list */
  display_name: string
  /** One sentence, present tense, no trailing period */
  description: string
  /** Semver, mirrors sdlc binary version at install time */
  version: string
  /** JSON Schema describing valid input for --run */
  input_schema: JsonSchema
  /** JSON Schema describing the data field in ToolResult */
  output_schema: JsonSchema
  /** True if --setup must run before first --run */
  requires_setup: boolean
  /** One sentence describing what setup does (required if requires_setup = true) */
  setup_description?: string
}

/** The result envelope returned by --run and --setup modes. */
export interface ToolResult<T = unknown> {
  ok: boolean
  data?: T
  /** Present only when ok = false */
  error?: string
  /** Wall-clock milliseconds for the operation */
  duration_ms?: number
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type JsonSchema = Record<string, any>
