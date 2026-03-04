# Spec: _shared/sdlc.ts — typed state access for tools

## Problem

Tools currently bypass the `.sdlc/` data model entirely. When a tool needs to read a feature list, it spawns `sdlc feature list --json` via `execSync` and parses the raw output — no types, no contract, no consistency. When the `dev-driver` tool needs to check milestones or ponder entries, it re-invents YAML parsing in-line.

This creates three compounding problems:

1. **Coupling to CLI output format.** Any CLI change breaks tool parsing silently. There are no compile-time or test-time guards.
2. **Duplication.** Every tool that reads features writes its own JSON parsing loop. Knowledge about paths (`features/<slug>/manifest.yaml`), YAML shapes, and state conventions is scattered.
3. **No write path.** Tools have no sanctioned way to write state. `writeBeat()` and `createPonder()` today require knowing the exact YAML schema and writing raw text. Without a shared module, each tool that needs to write `.sdlc/` data reinvents file conventions — or, worse, skips the write and loses state.

## Solution

A shared TypeScript module at `.sdlc/tools/_shared/sdlc.ts` that tools import to read and write `.sdlc/` state through correct, typed conventions.

The module follows the same "Rust is dumb data layer" principle applied to the TypeScript tool layer: it knows paths and shapes, not logic. Tools decide what to do; `sdlc.ts` knows where things live.

## API Contract

All functions accept `root: string` as the first argument — the project root directory (resolved via `getProjectRoot()` from `SDLC_ROOT` env var). This keeps every function pure and testable without environment mutation.

### `getProjectRoot(): string`

```typescript
export function getProjectRoot(): string
```

Resolves the project root. Priority:
1. `SDLC_ROOT` env var (injected by `sdlc tool run`)
2. `process.cwd()` fallback

Never throws — falls back silently. Tools call this once at startup and pass `root` down.

### `readVision(root: string): string`

```typescript
export function readVision(root: string): string
```

Reads `VISION.md` from the project root. Returns the full string content. Returns empty string if the file does not exist (no throw). Tools use the return value to decide whether to include VISION context in prompts or summaries.

### `readFeatures(root: string): FeatureSummary[]`

```typescript
export interface FeatureSummary {
  slug: string
  title: string
  phase: string
  description?: string
  tasks: TaskSummary[]
}

export interface TaskSummary {
  id: string
  title: string
  status: 'pending' | 'in_progress' | 'completed'
}

export function readFeatures(root: string): FeatureSummary[]
```

Walks `.sdlc/features/*/manifest.yaml` and parses each into `FeatureSummary`. Returns `[]` if the features directory does not exist or is empty. Skips corrupted manifests (logs a warning to stderr via the provided logger or `console.error`).

Does not load artifact markdown files — that is deliberately out of scope. If a tool needs artifact content, it reads the file directly.

### `readMilestones(root: string): MilestoneSummary[]`

```typescript
export interface MilestoneSummary {
  slug: string
  title: string
  status: string
  features: string[]  // feature slugs
}

export function readMilestones(root: string): MilestoneSummary[]
```

Walks `.sdlc/milestones/*/manifest.yaml` and returns summaries. Returns `[]` on empty or missing directory.

### `readBeat(root: string): BeatState`

```typescript
export interface BeatEvaluation {
  date: string
  scope: string
  lens: string
  verdict: 'on-track' | 'drifting' | 'off-course'
  summary: string
  concerns: BeatConcern[]
}

export interface BeatConcern {
  slug?: string
  title: string
  severity: 'high' | 'medium' | 'low'
  last_checked: string
  trend: 'improving' | 'stalling' | 'worsening' | null
}

export interface BeatWeeklyItem {
  id: string
  title: string
  domain: string
  severity: 'high' | 'medium' | 'low'
  last_checked: string | null
  verdict: string | null
  trend: string | null
}

export interface BeatState {
  last_updated?: string
  evaluations: BeatEvaluation[]
  weekly?: {
    generated: string
    items: BeatWeeklyItem[]
  }
}

export function readBeat(root: string): BeatState
```

Reads `.sdlc/beat.yaml`. Returns `{ evaluations: [] }` if the file does not exist.

### `writeBeat(root: string, state: BeatState): void`

```typescript
export function writeBeat(root: string, state: BeatState): void
```

Writes `.sdlc/beat.yaml`. Uses atomic write via temp file + rename. Serializes using a minimal YAML serializer (no external dependency — indented string builder, since `beat.yaml` has a known stable schema).

### `createPonder(root: string, title: string): string`

```typescript
export function createPonder(root: string, title: string): string
```

Creates a new ponder entry by spawning `sdlc ponder create "<title>"` and returning the generated slug. Throws if `sdlc` is not on PATH or the command fails. This is a thin wrapper — it does not replicate the creation logic, it delegates to the CLI which owns the slug generation and manifest init.

Returns the slug of the created entry.

### `appendPonderSession(root: string, slug: string, content: string): void`

```typescript
export function appendPonderSession(root: string, slug: string, content: string): void
```

Appends a session to an existing ponder entry using the two-step session log protocol:
1. Writes `content` to a temp file at `/tmp/ponder-session-<slug>-<timestamp>.md`
2. Spawns `sdlc ponder session log <slug> --file /tmp/ponder-session-<slug>-<timestamp>.md`

This respects the session logging invariant documented in MEMORY.md. Throws on failure.

## YAML Parsing Strategy

The module uses the same lightweight YAML parser pattern already established in `_shared/config.ts` — no external dependencies. For `readFeatures` and `readMilestones` (which need nested YAML), the module implements a minimal multi-line parser sufficient for the manifest schema. The parser handles:
- Scalar values (`key: value`)
- Simple lists (`- item`)
- Nested objects at one level

This is adequate for manifest.yaml files whose schemas are stable and controlled. It does not attempt to be a general YAML parser.

## What This Module Does NOT Do

- It does not replace `sdlc` CLI commands. The CLI is the authoritative interface for state transitions.
- It does not read artifact markdown files (spec.md, design.md, etc.) — tools read those directly if needed.
- It does not provide a full YAML parser. If a tool needs to read a complex schema, it should use `sdlc <command> --json` via `execSync` instead.
- It does not manage file locking. Beat state is small and single-writer — no concurrent write scenarios in the tool layer.

## Acceptance Criteria

1. `getProjectRoot()` returns `SDLC_ROOT` when set, `process.cwd()` otherwise.
2. `readFeatures()` on a project with 5 features returns 5 `FeatureSummary` objects with correct slugs, titles, and phases.
3. `readMilestones()` on a project with 2 milestones returns 2 `MilestoneSummary` objects with feature slug arrays.
4. `readBeat()` on a project with no `beat.yaml` returns `{ evaluations: [] }` without throwing.
5. `writeBeat()` writes a valid `beat.yaml` that `readBeat()` can round-trip.
6. `createPonder()` spawns `sdlc ponder create` and returns the slug from stdout.
7. `appendPonderSession()` follows the two-step temp file → `sdlc ponder session log` protocol.
8. `readVision()` returns empty string (not an error) when `VISION.md` does not exist.
9. All functions accept `root: string` and do not rely on global state.
10. No external npm dependencies introduced.
