# Spec: Vision and Architecture Guidance in Setup

## Problem

After `sdlc init`, new users face two gaps:

1. **First-run message points to the wrong next step**: `sdlc init` ends with "Next: sdlc feature create <slug> --title '...'" — but a first-time user should define Vision and Architecture in the UI before creating any features. Vision and Architecture documents are used by every AI agent to make decisions; without them, agent outputs are unanchored.

2. **No in-UI guidance for Vision/Architecture**: The setup screen (`/setup`) exists but is not linked from the dashboard empty state. A first-time user who opens the UI after `sdlc init` sees a dashboard with nothing on it and no clear path forward. Xist explored the URL bar to find `/setup` — not a discoverable flow.

3. **No README section on first steps**: The README documents install and features but never explains what to do after install — specifically that Vision and Architecture should be defined first.

Note: `sdlc-update-docs` handles the init completion message change (directing users to `sdlc ui`). This feature focuses on the UI guidance and README "First steps" section.

## Changes

### README.md — First steps section

**File:** `README.md`

Add a "First steps" section after the Updating section:

```markdown
## First steps

After running `sdlc init`, open the UI:

```bash
sdlc ui
```

Navigate to **Setup** (`/setup`) to define your project's Vision and Architecture:

- **Vision** — why the project exists and who it serves. AI agents use this to make decisions aligned with your goals.
- **Architecture** — how the system works, the key components, and technical constraints. Agents use this to understand boundaries.

Once Vision and Architecture are defined, you're ready to create features.
```

### Frontend — Setup UI subtitle/description text

**Files:** `frontend/src/` — the setup page or relevant setup component

Add descriptive subtitle text under the "Vision" and "Architecture" section headings in the setup UI (`/setup`) explaining what they are:

- Under **Vision heading**: "Explain why this project exists and who it serves. AI agents use your Vision to make decisions that stay aligned with your goals."
- Under **Architecture heading**: "Describe how the system works — key components, tech stack, and constraints. Agents use this to understand what's in scope."

### Frontend — Dashboard empty state or banner

**Files:** `frontend/src/` — the main dashboard component

When Vision or Architecture is missing from a project, show a persistent, dismissable banner or prominent empty-state message:

```
Your project hasn't defined Vision or Architecture yet.
These documents guide all agent decisions.
[Go to Setup →]
```

The banner should link directly to `/setup`. It should appear only when the documents are missing, not after they have been written.

## Scope

- **Files:** `README.md` and frontend setup/dashboard components
- **Frontend changes:** Add subtitle text to the two setup fields; add empty-state/banner to dashboard
- **Rust/CLI changes:** None (init message change is in sdlc-update-docs feature)
- **Existing users:** Dashboard banner is additive; setup text is additive — no breaking changes

## Non-Goals

- Making `/setup` the first screen shown automatically after init — that is a separate product decision
- Validating Vision/Architecture content quality — that is agent work, not UI work
