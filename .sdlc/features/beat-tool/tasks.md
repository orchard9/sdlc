# Tasks: beat-tool

## T1 — Scaffold beat tool with streaming:true meta and scope/mode input schema

Create `.sdlc/tools/beat/tool.ts` with the tool skeleton:
- `meta` object with correct name, display_name, description, version
- Input schema: `scope` (string) and `mode` (enum: evaluate|week)
- Output schema matching the spec
- CLI entrypoint (`--meta`, `--run` modes)
- Stub `run()` that emits one `gathering` event and returns an empty result

Also create `.sdlc/tools/beat/README.md` with usage documentation.

## T2 — Implement state gathering via _shared/sdlc.ts

Create `.sdlc/tools/_shared/sdlc.ts` with:
- `readVision(root)` — reads VISION.md, returns string (empty string if missing)
- `readFeatures(root)` — `execSync('sdlc feature list --json')`, returns typed Feature[]
- `readMilestones(root)` — `execSync('sdlc milestone list --json')`, returns typed Milestone[]
- `readFeatureDetail(root, slug)` — `execSync('sdlc feature show <slug> --json')`, returns FeatureDetail
- All functions throw with descriptive message on CLI failure

Wire `readVision`, `readFeatures`, `readMilestones` into `run()` under the `gathering` step.

## T3 — Implement agent recruitment and loading via _shared/agent.ts

Create `.sdlc/tools/_shared/agent.ts` with:
- `ensureAgent(root, slug, roleDescription)` — checks if `.claude/agents/<slug>.md` exists; if not, calls `sdlc ponder recruit <slug> --role "<description>"`; returns absolute path to agent file
- `runAgent(agentPath, prompt, opts?)` — invokes `claude --print` with the agent content and prompt, returns raw output string
- Timeout: default 60s, configurable via `opts.timeout_ms`

Wire `ensureAgent` into `run()` under the `recruiting` step.

## T4 — Implement agent invocation and verdict parsing

In `run()`, after recruiting:
- Build evaluation prompt from scope context (vision + feature summary)
- Call `runAgent()` with cto-cpo-lens agent
- Parse JSON verdict from response: `{ verdict, score, concerns }`
- Retry once if JSON parse fails (with explicit "respond with JSON only" instruction)
- Return error result if second parse also fails
- Emit `evaluating` progress event before invocation

## T5 — Implement beat.yaml persistence via writeBeat()

Implement `writeBeat(root, record)`:
- Load existing `.sdlc/beat.yaml` or start with `{ beats: [] }`
- Generate next sequential ID (`beat-001`, `beat-002`, etc.)
- Append record, write back atomically
- Return the assigned beat ID

Wire into `run()` under the `writing` step. Emit `writing` event with beat ID.

## T6 — Implement week mode

In `run()`, when `mode === 'week'`:
- Load `.sdlc/beat.yaml`
- Filter beats to last 14 days
- Group concerns by text similarity (simple substring match)
- Score by recurrence count
- Return top-5 as `WeekItem[]` with priority 1–5
- No agent invocation, no write
- Emit `gathering` → `done` only

## T7 — Stream NDJSON progress events at each major step

Ensure all five event types are emitted at the right moments:
- `gathering` — before state reads, include summary (N features, N milestones)
- `recruiting` — before ensureAgent, include agent slug and path
- `evaluating` — before runAgent, include feature count
- `writing` — before writeBeat, include beat ID
- `done` — with full ToolResult as the last line
- `error` — on any caught exception, before returning error result

All events must be valid JSON on their own line (no trailing commas, no multiline).

## T8 — Run sdlc tool sync and verify tools.md updated

After all tool files are in place, run `sdlc tool sync` to regenerate `tools.md`.
Verify the beat tool appears with correct name, description, and run command.
Fix any metadata issues found during sync.
