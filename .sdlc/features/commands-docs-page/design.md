# Design: Commands Catalog — /docs/commands

## Overview

A static, client-rendered commands catalog that replaces the `/docs/commands` placeholder section. All data lives as a typed constant array in the frontend — no API, no async loading. The component renders into the existing `DocsPage` section switch.

## Component Architecture

```
frontend/src/pages/DocsPage.tsx          ← modified: add CommandsCatalog branch
frontend/src/components/docs/
  CommandsCatalog.tsx                    ← new: search + grouped command list
  commands-data.ts                       ← new: static data (CommandEntry[])
```

### No new routes

The existing route `<Route path="/docs/:section" element={<DocsPage />} />` already handles `/docs/commands`. `DocsPage` reads `section` from params and renders the appropriate content. Currently the `commands` section renders a placeholder `<div>`. This feature replaces that placeholder with `<CommandsCatalog />`.

## Data Model

File: `frontend/src/components/docs/commands-data.ts`

```typescript
export type CommandCategory =
  | 'lifecycle'
  | 'planning'
  | 'workspace'
  | 'analysis'
  | 'tooling'
  | 'project'

export interface CommandEntry {
  slug: string         // "sdlc-run" — used as key and for search matching
  invocation: string   // "/sdlc-run <feature-slug>" — copied to clipboard
  description: string  // one-sentence purpose
  category: CommandCategory
}

export const CATEGORY_LABELS: Record<CommandCategory, string> = {
  lifecycle: 'Lifecycle',
  planning:  'Planning',
  workspace: 'Workspace',
  analysis:  'Analysis & Quality',
  tooling:   'Tooling',
  project:   'Project Setup',
}

export const COMMANDS: CommandEntry[] = [
  // lifecycle
  { slug: 'sdlc-next',        invocation: '/sdlc-next <feature-slug>',      description: 'Get the next directive for a feature and act on it.',             category: 'lifecycle' },
  { slug: 'sdlc-run',         invocation: '/sdlc-run <feature-slug>',       description: 'Autonomously drive a feature to completion.',                     category: 'lifecycle' },
  { slug: 'sdlc-approve',     invocation: '/sdlc-approve <feature-slug>',   description: 'Approve the current pending artifact for a feature.',             category: 'lifecycle' },
  { slug: 'sdlc-status',      invocation: '/sdlc-status',                   description: 'Show project and feature status overview.',                       category: 'lifecycle' },
  // planning
  { slug: 'sdlc-plan',            invocation: '/sdlc-plan',                              description: 'Distribute a plan into milestones, features, and tasks.',                             category: 'planning' },
  { slug: 'sdlc-prepare',         invocation: '/sdlc-prepare <milestone-slug>',          description: 'Pre-flight a milestone — align features with vision, fix gaps, write wave plan.',    category: 'planning' },
  { slug: 'sdlc-run-wave',        invocation: '/sdlc-run-wave <milestone-slug>',         description: 'Execute Wave 1 features in parallel, advance to next wave.',                        category: 'planning' },
  { slug: 'sdlc-pressure-test',   invocation: '/sdlc-pressure-test <milestone-slug>',    description: 'Pressure-test a milestone against user perspectives.',                             category: 'planning' },
  { slug: 'sdlc-milestone-uat',   invocation: '/sdlc-milestone-uat <milestone-slug>',    description: 'Run acceptance test for a milestone.',                                             category: 'planning' },
  { slug: 'sdlc-specialize',      invocation: '/sdlc-specialize <feature-slug>',         description: 'Shape a feature with specialized domain knowledge.',                               category: 'planning' },
  // workspace
  { slug: 'sdlc-ponder',               invocation: '/sdlc-ponder [slug or new idea]',    description: 'Open the ideation workspace with recruited thought partners.',                category: 'workspace' },
  { slug: 'sdlc-ponder-commit',        invocation: '/sdlc-ponder-commit <slug>',          description: 'Crystallize a pondered idea into milestones and features.',                  category: 'workspace' },
  { slug: 'sdlc-recruit',              invocation: '/sdlc-recruit <role>',                description: 'Recruit an expert thought partner as a persistent agent.',                  category: 'workspace' },
  { slug: 'sdlc-empathy',              invocation: '/sdlc-empathy <subject>',             description: 'Deep user perspective interviews before making decisions.',                  category: 'workspace' },
  { slug: 'sdlc-spike',                invocation: '/sdlc-spike <topic>',                 description: 'Time-boxed technical investigation on an uncertain area.',                  category: 'workspace' },
  { slug: 'sdlc-hypothetical-planning',invocation: '/sdlc-hypothetical-planning <scenario>', description: 'Plan a hypothetical scenario without committing state.',               category: 'workspace' },
  { slug: 'sdlc-hypothetical-do',      invocation: '/sdlc-hypothetical-do <scenario>',   description: 'Execute a hypothetical scenario.',                                          category: 'workspace' },
  { slug: 'sdlc-convo-mine',           invocation: '/sdlc-convo-mine',                    description: 'Extract insights and tasks from conversation history.',                    category: 'workspace' },
  // analysis
  { slug: 'sdlc-enterprise-readiness', invocation: '/sdlc-enterprise-readiness', description: 'Production readiness analysis.',                                                      category: 'analysis' },
  { slug: 'sdlc-quality-fix',          invocation: '/sdlc-quality-fix',          description: 'Fix failing quality-check results — triage by failure count and apply targeted fixes.', category: 'analysis' },
  { slug: 'sdlc-setup-quality-gates',  invocation: '/sdlc-setup-quality-gates',  description: 'Set up pre-commit hooks.',                                                            category: 'analysis' },
  { slug: 'sdlc-vision-adjustment',    invocation: '/sdlc-vision-adjustment',    description: 'Adjust the project vision document.',                                                category: 'analysis' },
  { slug: 'sdlc-architecture-adjustment', invocation: '/sdlc-architecture-adjustment', description: 'Adjust the architecture document.',                                           category: 'analysis' },
  { slug: 'sdlc-guideline',            invocation: '/sdlc-guideline [slug]',     description: 'Open the guideline workspace — gather evidence and publish engineering guidelines.', category: 'analysis' },
  // tooling
  { slug: 'sdlc-tool-run',    invocation: '/sdlc-tool-run <tool-name>',   description: 'Run a custom tool.',                   category: 'tooling' },
  { slug: 'sdlc-tool-build',  invocation: '/sdlc-tool-build <tool-name>', description: 'Build a new custom tool.',              category: 'tooling' },
  { slug: 'sdlc-skill-build', invocation: '/sdlc-skill-build <skill-name>', description: 'Build a new agent skill.',           category: 'tooling' },
  { slug: 'sdlc-tool-audit',  invocation: '/sdlc-tool-audit <tool-name>', description: 'Audit a tool for correctness and quality.', category: 'tooling' },
  { slug: 'sdlc-tool-uat',    invocation: '/sdlc-tool-uat <tool-name>',   description: 'User acceptance test a custom tool.',  category: 'tooling' },
  { slug: 'sdlc-cookbook',    invocation: '/sdlc-cookbook',               description: 'Browse and run cookbook recipes.',      category: 'tooling' },
  { slug: 'sdlc-cookbook-run',invocation: '/sdlc-cookbook-run <recipe>',  description: 'Run a specific cookbook recipe.',       category: 'tooling' },
  { slug: 'sdlc-suggest',     invocation: '/sdlc-suggest',                description: 'Get suggestions for what to work on next.', category: 'tooling' },
  // project
  { slug: 'sdlc-init', invocation: '/sdlc-init', description: 'Interview to bootstrap vision, architecture, config, and team for a new project.', category: 'project' },
]
```

## CommandsCatalog Component

File: `frontend/src/components/docs/CommandsCatalog.tsx`

### Layout

```
┌─────────────────────────────────────────────────────────┐
│  [ Search commands…                         ]  34 total  │
├─────────────────────────────────────────────────────────┤
│  LIFECYCLE · 4                                           │
│  ┌───────────────────────────────────────────────────┐  │
│  │ /sdlc-next <feature-slug>          [copy]          │  │
│  │   Get the next directive for a feature…            │  │
│  └───────────────────────────────────────────────────┘  │
│  ┌───────────────────────────────────────────────────┐  │
│  │ /sdlc-run <feature-slug>           [copy]          │  │
│  │   Autonomously drive a feature to completion.      │  │
│  └───────────────────────────────────────────────────┘  │
│  …                                                       │
│  PLANNING · 6                                            │
│  …                                                       │
└─────────────────────────────────────────────────────────┘
```

### Behaviour

- `useState('')` for the search query, updated on every keystroke (no debounce needed — 34 items is trivially fast).
- Filter: `cmd.slug.includes(q) || cmd.description.toLowerCase().includes(q.toLowerCase())`
- Group filtered results by category; render only categories that have at least one match.
- If no results after filter: show "No commands match" message (same style as empty states elsewhere).
- Category order is fixed: lifecycle → planning → workspace → analysis → tooling → project.

### Row Structure (each command)

```tsx
<div className="flex items-start gap-3 py-3 px-4 rounded-lg border border-border bg-card hover:bg-accent/30 transition-colors">
  <div className="flex-1 min-w-0">
    <code className="text-sm font-mono font-medium text-foreground">{entry.invocation}</code>
    <p className="text-xs text-muted-foreground mt-0.5">{entry.description}</p>
  </div>
  <CopyButton text={entry.invocation} />
</div>
```

### Category header

```tsx
<div className="flex items-center gap-2 mb-2 mt-5 first:mt-0">
  <h3 className="text-[10px] font-semibold uppercase tracking-widest text-muted-foreground/60">
    {CATEGORY_LABELS[cat]}
  </h3>
  <span className="text-[10px] text-muted-foreground/40">· {count}</span>
</div>
```

## DocsPage Modification

`DocsPage.tsx` already has the `sections` object and renders a placeholder div for each section. The change is:

1. Import `CommandsCatalog` at the top.
2. In the JSX, before (or instead of) the placeholder `<div>`, check `section === 'commands'` and render `<CommandsCatalog />`.

```tsx
// in DocsPage return:
{section === 'commands'
  ? <CommandsCatalog />
  : (
    <div className="border border-dashed border-border rounded-xl p-10 text-center">
      <Icon className="w-8 h-8 text-muted-foreground/30 mx-auto mb-3" />
      <p className="text-sm text-muted-foreground">{entry.description}</p>
      <p className="text-xs text-muted-foreground/60 mt-1">{entry.placeholder}</p>
    </div>
  )
}
```

## File Summary

| File | Action | Notes |
|---|---|---|
| `frontend/src/components/docs/commands-data.ts` | Create | Static `COMMANDS` array + types |
| `frontend/src/components/docs/CommandsCatalog.tsx` | Create | Search + grouped rows |
| `frontend/src/pages/DocsPage.tsx` | Modify | Import + render `CommandsCatalog` for `commands` section |

No backend changes. No new routes. No new dependencies.

## Wireframe ASCII

```
/docs/commands
────────────────────────────────────────────────
Terminal  Commands
Reference for all /sdlc-* agent commands.

[ 🔍 Search commands…                  ] 34 total

LIFECYCLE · 4
──────────────────────────────────────────────
 /sdlc-next <feature-slug>                 [📋]
   Get the next directive for a feature and act on it.

 /sdlc-run <feature-slug>                  [📋]
   Autonomously drive a feature to completion.

 /sdlc-approve <feature-slug>              [📋]
   Approve the current pending artifact for a feature.

 /sdlc-status                              [📋]
   Show project and feature status overview.

PLANNING · 6
──────────────────────────────────────────────
 /sdlc-plan                                [📋]
   Distribute a plan into milestones, features, and tasks.
 …
```
