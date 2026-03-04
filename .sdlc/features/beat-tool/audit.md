# Security Audit: beat-tool

## Scope

Security audit of:
- `.sdlc/tools/beat/tool.ts` — main tool
- `.sdlc/tools/_shared/sdlc.ts` — state reader primitive
- `.sdlc/tools/_shared/agent.ts` — agent recruiter/invoker

The beat tool is a local CLI tool that runs in the developer's own project directory. It does not expose a network service, handle user authentication, or process untrusted external input. The security surface is narrow.

---

## Trust Model

| Actor | Trust Level | Notes |
|---|---|---|
| Tool invoker | Trusted (local developer or agent) | Has full project filesystem access already |
| Input JSON (scope, mode) | Untrusted string fields | Source: stdin or sdlc-server subprocess injection |
| `sdlc` CLI output | Trusted (project binary) | Same trust as invoker's environment |
| `claude` CLI output | Trusted (local AI CLI) | Invoked as subprocess with project environment |
| `.sdlc/beat.yaml` | Trusted (project file) | Writable only by tool itself |
| `.claude/agents/*.md` | Trusted (project file) | Agent files written by `ensureAgent` |

---

## Findings

### A1 — `scope` input is not sanitized before use in log messages and NDJSON output (low)

In `tool.ts` `runEvaluate`, the raw `input.scope` string is interpolated into `emit()` payloads and the evaluation prompt sent to the agent:

```typescript
const scopeNote = input.scope === 'project' ? ... : `domain ${input.scope}`
emit('gathering', { message: `Reading project state (${scopeNote})...` })
```

And in the prompt (line 342):
```typescript
const prompt = `You are reviewing ${scopeDescription} against its vision...`
```

If `input.scope` contains NDJSON-breaking characters (e.g., `\n`, `"`), the emitted JSON line could break the streaming protocol for the caller. If `input.scope` contains shell metacharacters, they are inert here since no shell execution happens with the scope value directly, but they could confuse the agent's reasoning.

**Severity:** Low. The tool runs locally; the input comes from the developer's own command or the server's trusted tool-invocation pipeline. No shell injection is possible here because the scope value is never passed to `execSync` or `spawnSync` directly.

**Action:** Track as a task — add `scope.replace(/[\n\r"]/g, ' ')` normalization before interpolation. Accepted for this release.

### A2 — `feature:<slug>` slug is interpolated into an agent prompt without validation (low)

When `scope` is `feature:<slug>`, the slug is extracted via `input.scope.slice(8)` and used in the prompt without slug format validation. A malformed slug like `../../../../etc/passwd` would be passed to `readFeatureDetail` (via `sdlc feature show`) if called, and to the evaluation prompt. However:

1. `readFeatureDetail` is not actually called in the current implementation — the slug is only used as a filter against the already-loaded feature list.
2. The slug appears only in `scopeDescription` which is interpolated into the agent prompt text (not a shell command).

**Severity:** Low. No filesystem path traversal is possible given current implementation. If `readFeatureDetail` is wired in the future, slug validation will be required.

**Action:** Track as a task — validate slug format (`/^[a-z0-9][a-z0-9-]*$/`) before use. Accepted for this release.

### A3 — `ensureAgent` writes agent files to `.claude/agents/` using a slug-derived path (low)

In `agent.ts` `ensureAgent`, the `slug` parameter is used to construct the agent file path:

```typescript
const agentPath = join(agentDir, `${slug}.md`)
```

If `slug` contains path traversal sequences (e.g., `../../../.bashrc`), `join` will resolve them. An attacker would need to control the `slug` parameter passed to `ensureAgent`. In the beat tool, the slug is hardcoded (`'cto-cpo-lens'` or `'tech-lead-lens'`), so this is not currently exploitable.

For future tools that accept user-provided agent slugs, this would be a moderate severity finding.

**Severity:** Low (hardcoded slugs in current callers). No action needed now; validate slug format in future callers.

### A4 — `parseBeatYaml` in `tool.ts` uses a regex with `[\s\S]*?` (potential ReDoS — informational)

The regex in `parseBeatYaml` (line 199) includes `[\s\S]*?` in a non-anchored pattern. With a pathologically crafted `beat.yaml`, this could trigger exponential backtracking. However:

1. `beat.yaml` is a project-owned file written only by the tool itself
2. The file is not user-uploaded or externally sourced
3. The regex is used once per file load, not in a loop per line

**Severity:** Informational. No action needed. The attack vector (malicious `beat.yaml`) requires existing write access to the project directory, which grants full access anyway.

---

## Positive Security Properties

- **No shell injection surface**: `_shared/sdlc.ts` uses `readdirSync` and `readFileSync` directly — no shell command construction with user input.
- **`agent.ts` uses `spawnSync` with argument arrays**: The `claude --print --system-prompt <content>` call passes `systemPrompt` as a separate argument element, not as a shell string. This prevents shell injection via agent file content.
- **`writeBeat` atomic write**: Uses temp-file + rename in `_shared/sdlc.ts writeBeat`. Prevents partial writes from corrupting `beat.yaml`.
- **No network calls**: The tool makes no outbound HTTP requests. All external I/O is local CLI subprocess invocation.
- **No credential handling**: The tool reads no API keys, tokens, or secrets. The `claude` CLI handles authentication independently.
- **Error paths return structured results**: All error exits go through `emit('error', ...)` + `return { ok: false, error: ... }` rather than throwing unhandled exceptions that could leak stack traces to untrusted callers.
- **`requires_setup: false`**: No setup scripts with elevated permissions.

---

## Summary

The beat tool has a narrow, local-only security surface. The three low-severity findings are all defense-in-depth improvements against future misuse, not currently exploitable vulnerabilities. The tool is safe to ship.

All findings are tracked as tasks for future hardening cycles. No blockers.
