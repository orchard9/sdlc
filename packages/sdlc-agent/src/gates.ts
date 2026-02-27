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

async function runShellGate(gate: GateDefinition, cwd: string): Promise<GateResult> {
  const command = gate.command!;

  // Split the command safely — support simple commands with args but not pipes/redirects
  // For complex commands, the gate config should be a single executable with args
  const parts = command.split(/\s+/u);
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
