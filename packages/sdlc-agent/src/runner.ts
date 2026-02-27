import { query } from "@anthropic-ai/claude-agent-sdk";
import { SdlcClient } from "./sdlc-client.js";
import { createSdlcMcpServer, SDLC_SERVER_NAME } from "./tools/index.js";
import { agentForAction, isHumanGateAction, isTerminalAction } from "./agents/index.js";
import { loadSession, saveSession, clearSession } from "./session.js";
import type { SdlcDirective, AgentOptions, RunResult } from "./types.js";

const DEFAULT_MODEL = "claude-sonnet-4-6";
const DEFAULT_MAX_TURNS = 30;

function buildPrompt(directive: SdlcDirective): AsyncGenerator<
  { type: "user"; message: { role: "user"; content: string } },
  void,
  unknown
> {
  return (async function* () {
    const lines: string[] = [
      `# SDLC Directive: ${directive.action}`,
      ``,
      `**Feature:** ${directive.feature} — ${directive.title}`,
      `**Phase:** ${directive.current_phase}`,
      `**Action:** ${directive.action}`,
      ``,
      `## What to do`,
      directive.message,
      ``,
    ];

    if (directive.output_path) {
      lines.push(`**Output path:** \`${directive.output_path}\``);
    }
    if (directive.task_id) {
      lines.push(`**Task ID:** \`${directive.task_id}\``);
    }
    if (directive.gates && directive.gates.length > 0) {
      const shellGates = directive.gates.filter((g) => g.type === "shell" && g.auto);
      if (shellGates.length > 0) {
        lines.push(``, `**Auto gates** (run automatically before approval):`);
        for (const g of shellGates) {
          lines.push(`- \`${g.command}\` (${g.name})`);
        }
      }
    }

    lines.push(
      ``,
      `## Instructions`,
      `1. Call \`sdlc_get_directive\` to confirm the current state`,
      `2. Do the work described above`,
      `3. Call \`sdlc_approve_artifact\` (or \`sdlc_complete_task\`) when done`,
      `   - For artifact actions: approval will automatically run any configured gates`,
      `   - If gates fail: fix the issue and try approving again`,
    );

    yield {
      type: "user" as const,
      message: { role: "user" as const, content: lines.join("\n") },
    };
  })();
}

export async function runFeature(
  slug: string,
  opts: AgentOptions = {}
): Promise<RunResult> {
  const cwd = opts.cwd ?? process.cwd();
  const client = new SdlcClient({ cwd, bin: opts.sdlcBin ?? "sdlc" });
  let actionsCompleted = 0;

  // Load persisted session (if any) so Claude has context from prior iterations
  let sessionId = loadSession(cwd, slug);
  if (sessionId) {
    console.log(`[sdlc-agent] Resuming session: ${sessionId.slice(0, 8)}…`);
  }

  console.log(`[sdlc-agent] Starting feature: ${slug}`);

  while (true) {
    // Read current directive
    let directive: SdlcDirective;
    try {
      directive = await client.getDirective(slug);
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      console.error(`[sdlc-agent] Failed to get directive: ${error.message}`);
      return {
        feature: slug,
        finalPhase: "unknown",
        actionsCompleted,
        stoppedAt: "error",
        error,
      };
    }

    opts.onDirective?.(directive);
    console.log(
      `[sdlc-agent] ${directive.action} (phase: ${directive.current_phase})`
    );

    // Check terminal state
    if (isTerminalAction(directive.action)) {
      console.log(`[sdlc-agent] Feature complete: ${slug}`);
      return {
        feature: slug,
        finalPhase: directive.current_phase,
        actionsCompleted,
        stoppedAt: "done",
      };
    }

    // Check human gate — pause and give instructions
    if (isHumanGateAction(directive.action)) {
      const nextCmd = directive.next_command ?? `sdlc artifact approve ${slug} <type>`;
      console.log(
        `[sdlc-agent] Human gate: ${directive.action}\n` +
          `  → ${directive.message}\n` +
          `  → After approving, resume with: sdlc-agent run ${slug}`
      );
      console.log(`  → Command: ${nextCmd}`);
      return {
        feature: slug,
        finalPhase: directive.current_phase,
        actionsCompleted,
        stoppedAt: "human_gate",
      };
    }

    // Look up agent for this action
    const agentConfig = agentForAction(directive.action);
    if (!agentConfig) {
      console.warn(
        `[sdlc-agent] No agent defined for action "${directive.action}". Pausing for human.`
      );
      return {
        feature: slug,
        finalPhase: directive.current_phase,
        actionsCompleted,
        stoppedAt: "human_gate",
      };
    }

    // Build MCP server with the current directive's gates
    const mcpServer = createSdlcMcpServer(client, {
      gates: directive.gates ?? [],
      cwd,
    });

    console.log(
      `[sdlc-agent] Running ${agentConfig.model} agent for: ${directive.action}`
    );

    // Invoke Claude via Agent SDK
    // NOTE: MCP servers require streaming input mode (async generator prompt)
    try {
      for await (const message of query({
        prompt: buildPrompt(directive) as AsyncIterable<any>, // eslint-disable-line @typescript-eslint/no-explicit-any
        options: {
          cwd,
          model: opts.model ?? agentConfig.model ?? DEFAULT_MODEL,
          maxTurns: opts.maxTurns ?? DEFAULT_MAX_TURNS,
          mcpServers: { [SDLC_SERVER_NAME]: mcpServer },
          allowedTools: agentConfig.tools,
          permissionMode: "acceptEdits",
          systemPrompt: agentConfig.prompt,
          // Resume existing session so Claude remembers prior context
          ...(sessionId ? { resume: sessionId } : {}),
        },
      })) {
        opts.onMessage?.(message);

        const msg = message as Record<string, unknown>;

        // Capture session ID from the init message
        if (msg["type"] === "system" && msg["subtype"] === "init") {
          const newSessionId = msg["session_id"] as string | undefined;
          if (newSessionId && newSessionId !== sessionId) {
            sessionId = newSessionId;
            saveSession(cwd, slug, newSessionId);
          }
        }

        // Log result
        if (msg["type"] === "result") {
          if (msg["subtype"] === "success") {
            console.log(`[sdlc-agent] Action complete: ${directive.action}`);
          } else if (msg["subtype"] === "error_max_turns") {
            console.warn(`[sdlc-agent] Max turns reached for: ${directive.action}`);
          }
        }
      }
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      console.error(
        `[sdlc-agent] Agent error during ${directive.action}: ${error.message}`
      );
      // Clear the session on error to force a fresh start next time
      clearSession(cwd, slug);
      return {
        feature: slug,
        finalPhase: directive.current_phase,
        actionsCompleted,
        stoppedAt: "error",
        error,
      };
    }

    actionsCompleted++;
  }
}
