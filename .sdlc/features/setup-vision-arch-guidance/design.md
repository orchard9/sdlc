# Design: Vision and Architecture Guidance in Setup

## Overview

Three focused, additive changes that close the discoverability gap between `sdlc init` and a productive first session:

1. **README.md** — add a "First steps" section explaining that Vision and Architecture should be defined before creating features.
2. **SetupPage.tsx** — improve the subtitle text under the Vision and Architecture step headings so users understand what they are and why they matter.
3. **Dashboard.tsx** — the existing setup-incomplete banner already fires when Vision/Architecture are missing, but its condition is conflated with team setup. Refine the condition so the banner is specifically about Vision/Architecture missing, not general project incompleteness.

## Files Changed

| File | Change |
|---|---|
| `README.md` | Add "First steps" section after the "Initialize a project" section |
| `frontend/src/pages/SetupPage.tsx` | Improve subtitle/description text for Vision (step 2) and Architecture (step 3) |
| `frontend/src/pages/Dashboard.tsx` | Refine `setupIncomplete` condition: fire only when Vision or Architecture is missing, not when team is missing |

## Detailed Changes

### 1. README.md — First Steps Section

Insert after the "Initialize a project" section (after line 56, before "### Create a feature"):

```markdown
### First steps

After running `sdlc init`, open the UI:

```bash
sdlc ui
```

Navigate to **Setup** (`/setup`) to define your project's Vision and Architecture:

- **Vision** — why the project exists and who it serves. AI agents use this to make decisions aligned with your goals.
- **Architecture** — how the system works, the key components, and technical constraints. Agents use this to understand boundaries.

Once Vision and Architecture are defined, you're ready to create features.
```

### 2. SetupPage.tsx — Vision Step Subtitle

Current text (line 291):
```tsx
Edit the generated vision or write your own.{' '}
<code className="text-primary">VISION.md</code> tells agents what you're building and why.
```

Replace with:
```tsx
Explain why this project exists and who it serves.{' '}
<code className="text-primary">VISION.md</code>{' '}
is read by every AI agent to make decisions that stay aligned with your goals. Edit the generated draft or write your own.
```

### 3. SetupPage.tsx — Architecture Step Subtitle

Current text (line 335):
```tsx
Edit the generated architecture or write your own.{' '}
<code className="text-primary">ARCHITECTURE.md</code> maps your tech stack and key components.
```

Replace with:
```tsx
Describe how the system works — key components, tech stack, and constraints.{' '}
<code className="text-primary">ARCHITECTURE.md</code>{' '}
tells agents what's in scope. Edit the generated draft or write your own.
```

### 4. Dashboard.tsx — Refine Banner Condition

Current condition (lines 147-149):
```tsx
const noProject = !cfg?.project.description || (!vision?.exists && !arch?.exists)
const noTeam = agents.length === 0
setSetupIncomplete(noProject || noTeam)
```

New condition — fires only when Vision or Architecture is missing:
```tsx
const missingVisionOrArch = !vision?.exists || !arch?.exists
setSetupIncomplete(missingVisionOrArch)
```

The banner text also needs to be updated to be specific:

Current text:
```
Project setup is incomplete — agents won't have enough context to work with.
```

New text:
```
Vision or Architecture not defined — agents use these to make aligned decisions.
```

## What Is Not Changed

- The banner link target (`/setup`) stays the same.
- The `setupIncomplete` state variable name and banner JSX structure stay the same — only the condition and the text change.
- No backend/Rust changes.
- No new components.
- The `agents` and `config` fetches on dashboard load remain (they may still be used for other dashboard logic in future, and removing them now would be scope creep).

## ASCII Wireframes

### SetupPage — Vision step (step 2)

```
┌────────────────────────────────────────────────────┐
│ Vision                                       [GitBranch icon] │
│                                                    │
│ Explain why this project exists and who it         │
│ serves. VISION.md is read by every AI agent        │
│ to make decisions that stay aligned with your      │
│ goals. Edit the generated draft or write your own. │
│                                                    │
│ ┌──────────────────────────────────────────────┐  │
│ │ (textarea — monospace, 12 rows)              │  │
│ └──────────────────────────────────────────────┘  │
│                                                    │
│ [Save & Continue]                                  │
└────────────────────────────────────────────────────┘
```

### Dashboard — banner (Vision/Arch missing)

```
┌──────────────────────────────────────────────────────────────┐
│ ⚠  Vision or Architecture not defined — agents use these to  │
│    make aligned decisions.         [Go to Setup →]           │
└──────────────────────────────────────────────────────────────┘
```
