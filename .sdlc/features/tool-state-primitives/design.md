# Design: _shared/sdlc.ts — typed state access for tools

## Overview

`_shared/sdlc.ts` is a single TypeScript module added to the existing `_shared/` directory alongside `types.ts`, `config.ts`, `log.ts`, and `runtime.ts`. It follows the exact same pattern: no external dependencies, pure functions, no global state.

The module is the "path registry + safe reader/writer" for `.sdlc/` from within the TypeScript tool layer. Every read is defensive (missing files return empty defaults, not throws). Every write uses atomic temp-file replacement.

## File Location

```
.sdlc/tools/_shared/sdlc.ts   ← new file (this feature)
.sdlc/tools/_shared/types.ts  ← existing
.sdlc/tools/_shared/config.ts ← existing
.sdlc/tools/_shared/log.ts    ← existing
.sdlc/tools/_shared/runtime.ts ← existing
```

## Module Structure

The module is organized into four sections:

```
1. Path helpers         — resolve .sdlc/ paths from root
2. YAML mini-parser     — parse manifest.yaml files (no deps)
3. Readers              — readVision, readFeatures, readMilestones, readBeat
4. Writers              — writeBeat, createPonder, appendPonderSession
```

## Section 1: Path Helpers

Private constants and helpers mirror the Rust `paths.rs` constants — they are not exported since tools should call the public functions, not construct paths themselves.

```typescript
// Private — tools use public functions, not paths directly
const FEATURES_DIR = '.sdlc/features'
const MILESTONES_DIR = '.sdlc/milestones'
const BEAT_FILE = '.sdlc/beat.yaml'
const VISION_FILE = 'VISION.md'

function featuresDir(root: string): string { return join(root, FEATURES_DIR) }
function milestonesDir(root: string): string { return join(root, MILESTONES_DIR) }
function beatPath(root: string): string { return join(root, BEAT_FILE) }
function visionPath(root: string): string { return join(root, VISION_FILE) }
```

## Section 2: YAML Mini-Parser

The existing `config.ts` parses flat `key: value` YAML only. The manifest files require one level of nesting and simple arrays. The new parser handles:

```
slug: my-feature
title: My Feature
phase: implementation
tasks:
  - id: T1
    title: Do the thing
    status: pending
  - id: T2
    title: Another thing
    status: completed
```

Implementation strategy: two-pass line scan.

**Pass 1 — flat scalars:** scan lines matching `^(\w[\w-]*): (.+)$`, collect into a flat map.

**Pass 2 — named arrays:** when a line matches `^(\w+):$` followed by indented list items (`^\s{2}-`), collect the block as an array of sub-objects. Each sub-object is parsed with the same flat scalar logic applied to its indented key-value pairs.

This is sufficient for `manifest.yaml` and `beat.yaml` schemas. It does not need to handle arbitrary nesting.

```typescript
function parseYamlScalars(content: string): Record<string, string>
function parseYamlArray(content: string, key: string): Record<string, string>[]
function parseManifest(content: string): Record<string, unknown>
```

## Section 3: Readers

### `getProjectRoot(): string`

```typescript
export function getProjectRoot(): string {
  return getEnv('SDLC_ROOT') ?? process.cwd()
}
```

Uses `getEnv` from `runtime.ts` for cross-runtime compat. Falls back to `process.cwd()`.

### `readVision(root: string): string`

```typescript
export function readVision(root: string): string {
  try {
    return readFileSync(visionPath(root), 'utf8')
  } catch {
    return ''
  }
}
```

Simple, defensive. No YAML — just a raw file read.

### `readFeatures(root: string): FeatureSummary[]`

```
1. readdirSync(featuresDir(root)) — list subdirectories
2. For each slug dir: readFileSync(join(dir, 'manifest.yaml'))
3. parseManifest() → extract slug, title, phase, description, tasks[]
4. Corrupted files: log to stderr, skip
5. Return FeatureSummary[]
```

Tasks are extracted from the `tasks:` array block in the manifest. The task status is one of `pending | in_progress | completed` per the manifest schema seen in the existing `manifest.yaml` files.

### `readMilestones(root: string): MilestoneSummary[]`

Same pattern as `readFeatures`. The milestone manifest `features:` field is a YAML array of feature slugs (simple string list, not objects).

### `readBeat(root: string): BeatState`

```
1. readFileSync(.sdlc/beat.yaml) — return { evaluations: [] } if ENOENT
2. parseManifest() for top-level fields: last_updated
3. parseYamlArray for evaluations[] block — each evaluation has concerns[] sub-array
4. parseYamlArray for weekly.items[] block (if present)
5. Return BeatState
```

The `beat.yaml` schema has two levels of nesting (evaluations contain concerns). The parser handles this by detecting the `concerns:` key within each evaluation block and recursing one level.

## Section 4: Writers

### `writeBeat(root: string, state: BeatState): void`

No external YAML library. Serialize using a template-style builder:

```typescript
function serializeBeat(state: BeatState): string {
  const lines: string[] = []
  lines.push(`last_updated: "${state.last_updated ?? new Date().toISOString().slice(0,10)}"`)
  lines.push('evaluations:')
  for (const ev of state.evaluations) {
    lines.push(`  - date: "${ev.date}"`)
    lines.push(`    scope: ${ev.scope}`)
    // ... etc
  }
  // weekly section if present
  return lines.join('\n') + '\n'
}
```

Atomic write pattern:
```typescript
const tmp = beatPath(root) + '.tmp'
writeFileSync(tmp, content, 'utf8')
renameSync(tmp, beatPath(root))
```

### `createPonder(root: string, title: string): string`

```typescript
export function createPonder(root: string, title: string): string {
  const result = spawnSync('sdlc', ['ponder', 'create', title], {
    cwd: root, encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe']
  })
  if (result.status !== 0) throw new Error(`sdlc ponder create failed: ${result.stderr}`)
  // Output: "Created ponder entry: <slug>"  or just slug on stdout
  const slug = result.stdout.trim().split(/\s+/).pop() ?? ''
  if (!slug) throw new Error('sdlc ponder create produced no slug')
  return slug
}
```

### `appendPonderSession(root: string, slug: string, content: string): void`

```typescript
export function appendPonderSession(root: string, slug: string, content: string): void {
  const tmp = `/tmp/ponder-session-${slug}-${Date.now()}.md`
  writeFileSync(tmp, content, 'utf8')
  const result = spawnSync('sdlc', ['ponder', 'session', 'log', slug, '--file', tmp], {
    cwd: root, encoding: 'utf8'
  })
  // Best-effort cleanup
  try { unlinkSync(tmp) } catch { /* ignore */ }
  if (result.status !== 0) throw new Error(`ponder session log failed: ${result.stderr}`)
}
```

This strictly follows the two-step session logging protocol from MEMORY.md. The temp file is cleaned up after the `sdlc` command completes.

## Exported Types

All types are re-exported from `sdlc.ts` so tools import from a single place:

```typescript
import type {
  FeatureSummary,
  TaskSummary,
  MilestoneSummary,
  BeatState,
  BeatEvaluation,
  BeatConcern,
  BeatWeeklyItem,
} from '../_shared/sdlc.ts'

import {
  getProjectRoot,
  readVision,
  readFeatures,
  readMilestones,
  readBeat,
  writeBeat,
  createPonder,
  appendPonderSession,
} from '../_shared/sdlc.ts'
```

## Import Pattern in Tool Files

Tools that currently use `execSync('sdlc feature list --json', ...)` can be progressively migrated:

**Before (dev-driver pattern):**
```typescript
const raw = execSync('sdlc next --json', { encoding: 'utf8', cwd: root })
const all = JSON.parse(raw) as FeatureDirective[]
```

**After (future, for simple reads):**
```typescript
import { readFeatures } from '../_shared/sdlc.ts'
const features = readFeatures(root)
```

Note: `readFeatures` returns manifest data, not directive data. Tools that need `action` and `next_command` still call `sdlc next --json` — `sdlc.ts` does not replicate classifier output.

## No External Dependencies

The module uses only Node.js built-ins:
- `node:fs` — `readFileSync`, `writeFileSync`, `readdirSync`, `renameSync`, `unlinkSync`, `statSync`
- `node:path` — `join`
- `node:child_process` — `spawnSync` (for `createPonder` and `appendPonderSession`)
- `../\_shared/runtime.ts` — `getEnv` (already a shared dep)

No `js-yaml`, no `yaml`, no `@types/node` beyond what's already in the tool environment.

## Task T5: Skill Updates

The `sdlc-tool-build` and `sdlc-tool-audit` skills in `init.rs` need one addition each:

**sdlc-tool-build:** Add a note under "Available shared modules":
> `_shared/sdlc.ts` — typed state access: `readFeatures()`, `readMilestones()`, `readBeat()/writeBeat()`, `createPonder()`, `readVision()`. Import when your tool needs to read or write `.sdlc/` state files. Use this instead of shelling out to `sdlc feature list --json` for simple reads.

**sdlc-tool-audit:** Add to the audit checklist:
> - If the tool reads `.sdlc/` files directly with `readFileSync`, recommend migrating to `_shared/sdlc.ts` primitives.

## Diagram: Module Relationships

```
tool.ts (e.g. dev-driver)
    │
    ├── ../_shared/types.ts      (ToolMeta, ToolResult)
    ├── ../_shared/log.ts        (makeLogger)
    ├── ../_shared/runtime.ts    (getArgs, readStdin, getEnv, exit)
    ├── ../_shared/config.ts     (loadToolConfig)
    └── ../_shared/sdlc.ts  ← NEW
            │
            ├── reads .sdlc/features/*/manifest.yaml
            ├── reads .sdlc/milestones/*/manifest.yaml
            ├── reads VISION.md
            ├── reads/writes .sdlc/beat.yaml
            └── spawns sdlc CLI (ponder create, ponder session log)
```
