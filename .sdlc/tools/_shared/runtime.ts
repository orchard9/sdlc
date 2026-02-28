/**
 * Cross-runtime helpers for Bun, Deno, and Node.
 *
 * Normalizes: argv access, stdin reading, env access, and process exit
 * across the three supported runtimes.
 *
 * Detection: checks for globalThis.Deno to identify Deno; falls back
 * to process (Node.js / Bun).
 */

/* eslint-disable @typescript-eslint/no-explicit-any */

/** Returns command-line arguments after the script name (process.argv[2+]). */
export function getArgs(): string[] {
  if (typeof (globalThis as any).Deno !== 'undefined') {
    return [...(globalThis as any).Deno.args]
  }
  return process.argv.slice(2)
}

/** Read all of stdin as a UTF-8 string. Returns empty string if stdin is a TTY or closed. */
export async function readStdin(): Promise<string> {
  if (typeof (globalThis as any).Deno !== 'undefined') {
    const chunks: Uint8Array[] = []
    const reader = (globalThis as any).Deno.stdin.readable.getReader()
    try {
      while (true) {
        const { done, value } = await reader.read()
        if (done) break
        chunks.push(value)
      }
    } finally {
      reader.releaseLock()
    }
    const total = chunks.reduce((sum: number, c: Uint8Array) => sum + c.length, 0)
    const merged = new Uint8Array(total)
    let offset = 0
    for (const chunk of chunks) {
      merged.set(chunk, offset)
      offset += chunk.length
    }
    return new TextDecoder().decode(merged)
  }
  // Node.js / Bun
  if ((process.stdin as any).isTTY) return ''
  const chunks: Buffer[] = []
  for await (const chunk of process.stdin) {
    chunks.push(Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk))
  }
  return Buffer.concat(chunks).toString('utf8')
}

/** Get a process environment variable. Works across Bun, Deno, and Node. */
export function getEnv(key: string): string | undefined {
  if (typeof (globalThis as any).Deno !== 'undefined') {
    return (globalThis as any).Deno.env.get(key)
  }
  return process.env[key]
}

/** Exit the process with the given code. */
export function exit(code: number): never {
  if (typeof (globalThis as any).Deno !== 'undefined') {
    ;(globalThis as any).Deno.exit(code)
  }
  process.exit(code)
  throw new Error('unreachable')
}
