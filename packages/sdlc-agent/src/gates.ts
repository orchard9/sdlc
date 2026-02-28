import { execFile } from "node:child_process";
import { promisify } from "node:util";
import type { GateDefinition, GateResult } from "./types.js";

const execFileAsync = promisify(execFile);

const GATE_TIMEOUT_MS = 120_000; // 2 minutes per gate

/**
 * Run the auto shell gates from a directive's gate list.
 * Human gates and non-auto gates are skipped (marked passed/skipped).
 */
export async function runGates(
  gates: GateDefinition[],
  cwd: string
): Promise<GateResult[]> {
  const results: GateResult[] = [];

  for (const gate of gates) {
    if (!gate.auto || gate.type !== "shell" || !gate.command) {
      results.push({
        name: gate.name,
        type: gate.type,
        passed: true,
        output: `skipped (${gate.type}, auto=${gate.auto})`,
      });
      continue;
    }

    results.push(await runShellGate(gate, cwd));
  }

  return results;
}

/**
 * Parse a command string into an array of argument tokens.
 *
 * Supported:
 *   - Whitespace-separated bare tokens
 *   - Single-quoted strings: 'value with spaces' → one token, quotes stripped
 *   - Double-quoted strings: "value with spaces" → one token, quotes stripped
 *   - Backslash-escaped spaces: value\ with\ spaces → one token
 *
 * Not supported (use a wrapper script for these):
 *   - Pipes and redirects: | > < >>
 *   - Subshell expressions: $(...) or `...`
 *   - Environment variable expansion: $VAR or ${VAR}
 *   - Glob expansion: * ? [...]
 *
 * Throws if a quoted string is not closed.
 */
function parseCommandArgs(command: string): string[] {
  const tokens: string[] = [];
  let current = "";
  let i = 0;

  while (i < command.length) {
    // command[i] is always defined inside `i < command.length` bounds
    const ch = command[i]!;

    if (ch === "'" || ch === '"') {
      // Quoted string — consume until the matching closing quote
      const quote = ch;
      i++;
      while (i < command.length && command[i] !== quote) {
        current += command[i]!;
        i++;
      }
      if (i >= command.length) {
        throw new Error(
          `Unclosed ${quote === "'" ? "single" : "double"} quote in command: ${command}`
        );
      }
      i++; // skip closing quote
    } else if (ch === "\\") {
      // Backslash escape — treat next character literally
      i++;
      if (i < command.length) {
        current += command[i]!;
        i++;
      }
    } else if (/\s/u.test(ch)) {
      // Whitespace — flush current token if non-empty
      if (current.length > 0) {
        tokens.push(current);
        current = "";
      }
      i++;
    } else {
      current += ch;
      i++;
    }
  }

  if (current.length > 0) {
    tokens.push(current);
  }

  return tokens;
}

async function runShellGate(gate: GateDefinition, cwd: string): Promise<GateResult> {
  const command = gate.command!;

  // Parse the command into tokens with proper quoted-argument support.
  // Pipes, redirects, subshells, and env var expansion are not supported —
  // use a wrapper script for those cases.
  const parts = parseCommandArgs(command);
  const [bin, ...args] = parts;

  if (!bin) {
    return {
      name: gate.name,
      type: "shell",
      passed: false,
      error: `Gate "${gate.name}" has an empty command`,
    };
  }

  try {
    const { stdout, stderr } = await execFileAsync(bin, args, {
      cwd,
      timeout: GATE_TIMEOUT_MS,
    });
    return {
      name: gate.name,
      type: "shell",
      passed: true,
      output: (stdout + stderr).slice(0, 2000),
    };
  } catch (err: unknown) {
    const e = err as { stdout?: string; stderr?: string; message?: string; killed?: boolean };
    return {
      name: gate.name,
      type: "shell",
      passed: false,
      output: ((e.stdout ?? "") + (e.stderr ?? "")).slice(0, 2000),
      error: e.killed
        ? `Gate timed out after ${GATE_TIMEOUT_MS / 1000}s`
        : (e.message ?? String(err)),
    };
  }
}

export function allGatesPassed(results: GateResult[]): boolean {
  return results.every((r) => r.passed);
}

export function formatGateResults(results: GateResult[]): string {
  return results
    .map(
      (r) =>
        `${r.passed ? "✓" : "✗"} [${r.name}]: ${
          r.passed
            ? r.output?.trim()?.split("\n").at(-1) ?? "PASSED"
            : `FAILED — ${r.error ?? r.output?.slice(0, 200)}`
        }`
    )
    .join("\n");
}
