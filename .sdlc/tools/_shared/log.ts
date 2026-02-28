/**
 * Standard SDLC Tool Logger
 *
 * Writes structured log lines to STDERR (never stdout — stdout is reserved
 * for JSON output). Use this in every tool to produce consistent, parseable logs.
 *
 * Format: [sdlc-tool:<name>] LEVEL: message
 * Example: [sdlc-tool:ama] INFO:  Indexed 312 files in 842ms
 *
 * Set SDLC_TOOL_DEBUG=1 to enable debug-level output.
 */

export function makeLogger(toolName: string) {
  const prefix = `[sdlc-tool:${toolName}]`
  return {
    info:  (msg: string) => console.error(`${prefix} INFO:  ${msg}`),
    warn:  (msg: string) => console.error(`${prefix} WARN:  ${msg}`),
    error: (msg: string) => console.error(`${prefix} ERROR: ${msg}`),
    debug: (msg: string) => {
      if (process.env.SDLC_TOOL_DEBUG) console.error(`${prefix} DEBUG: ${msg}`)
    },
  }
}

export type Logger = ReturnType<typeof makeLogger>
