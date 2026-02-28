# Ponder Dialogue

The ponder detail panel is a dialogue stream — a living transcript of the thinking
that happened. Sessions accumulate chronologically. Tool calls, recruited voices,
ASCII sketches, decisions, and open questions all appear in one scrollable stream.
The owner can inject into the dialogue at any time; that message seeds the next
agent session.

---

## Layout

```
┌─ Left ───────────┬─ Right: Dialogue ─────────────────────────────────────────┐
│  Ponder      [+] │                                                            │
│  ───────────     │  Installable Audit Cards                      [exploring]  │
│  All        9    │  installable-audit-cards                                   │
│                  │                                                            │
│  Installable  ●  │  ┌─ Team ──────────────────────────────────── [+ recruit] ┤
│  Agent SDK       │  │  ┌────┐  ┌────┐  ┌────┐                               │
│  ...             │  │  │ VE │  │ MD │  │ HT │                               │
│                  │  │  └────┘  └────┘  └────┘                               │
│                  │  │  Vera    Marcus   Hiroshi                              │
│                  │  └────────────────────────────────────────────────────── ─┤
│                  │                                                            │
│                  │  ┌─ Orientation ──────────────────────────────────────── ┐ │
│                  │  │  WHERE WE ARE   Pipeline architecture taking shape     │ │
│                  │  │  → NEXT MOVE    Stress-test cold-start + trust signal  │ │
│                  │  │  COMMIT SIGNAL  Schema + pipeline both validated       │ │
│                  │  └──────────────────────────────────────────────────────┘ │
│                  │                                                            │
│                  │  ─────────── Session 1 · 3 days ago ───────────────────   │
│                  │  ...                                                       │
│                  │                                                            │
│                  │  ─────────── Session 2 · today ────────────────────────   │
│                  │  ⚑  Decided: schema-first, pipeline second                │
│                  │  ?  Open: cold-start when zero packages indexed           │
│                  │                          ← auto-scrolled to here          │
│                  │                                                            │
│                  │  ┌─────────────────────────────────────────────────────┐  │
│                  │  │  Add a thought, constraint, or question...    [→]   │  │
│                  │  └─────────────────────────────────────────────────────┘  │
└──────────────────┴────────────────────────────────────────────────────────────┘
```

---

## Interaction Flow

### Typing a message and sending

1. User types in the input box and hits `[→]`
2. Message is appended to the stream immediately as a `Owner` participant block
   using the git user name (e.g. `JORDAN · Owner`)
3. UI fires `POST /api/ponder/:slug/chat { message, owner_name }`
4. Server spawns agent subprocess running `/sdlc-ponder <slug> <message>`
5. Server emits SSE event `{ type: "ponder_run_started", slug }`
6. UI shows locked input + MCP call card + `◌ agent working...` placeholder
   under the new session separator
7. Agent runs, writes `sessions/session-NNN.md`, updates manifest orientation
8. Server emits SSE event `{ type: "ponder_run_completed", slug }`
9. UI re-fetches `GET /api/roadmap/:slug/sessions` and the new session content
10. Full session replaces the placeholder; orientation strip updates; input unlocks

The agent is always detached from the UI. The UI tracks run state via SSE, not
via the HTTP response. Any tab watching the same ponder sees the same live state.

### Locked state (session in progress)

```
│  ┌─ JORDAN · Owner ──────────────────────── just now ───┐    │
│  │  What about cold start — zero packages indexed,       │    │
│  │  empty state or seed with curated picks?              │    │
│  └───────────────────────────────────────────────────── ┘    │
│                                                               │
│  ┌─ sdlc_ponder_chat ──────────────────────── ● live ───┐    │
│  │  slug     installable-audit-cards                     │    │
│  │  message  "What about cold start — zero packages..."  │    │
│  │  session  3                                           │    │
│  └────────────────────────────────────────────────────── ┘    │
│                                                               │
│  ─────────── Session 3 · just now ──────────────────────      │
│                                                               │
│       ◌  agent working...                                     │
│                                                               │
│  ┌─────────────────────────────────────────────────────┐     │
│  │  session in progress...                      [stop] │     │
│  └─────────────────────────────────────────────────────┘     │
```

### After SSE fires — session lands

```
│  ─────────── Session 3 · just now ──────────────────────      │
│                                                               │
│  ▸ read: card-schema.md                                       │
│    No cold-start handling in current schema.                  │
│                                                               │
│  VERA · Supply Chain Security                                 │
│  Cold start is the honest state. Seeding with curated         │
│  picks creates false confidence. Ship the empty state.        │
│                                                               │
│  ⚑  Decided: ship empty state, no seeding                    │
│                                                               │
│  ┌─────────────────────────────────────────────────────┐     │
│  │  Add a thought, constraint, or question...    [→]   │     │
│  └─────────────────────────────────────────────────────┘     │
```

### Stop button

Sends `DELETE /api/ponder/:slug/chat/current` which kills the agent subprocess
(SIGTERM). The partial session file (if any) is left on disk as-is. SSE emits
`{ type: "ponder_run_stopped", slug }`. The session separator and placeholder
remain visible but the `◌ agent working...` indicator clears.

---

## Team Row

Team members appear as avatar chips across the top of the detail panel — above
the orientation strip, below the title. This makes the cast of the dialogue
visible at a glance before reading the stream.

```
┌─ Team ──────────────────────────────────────── [+ recruit] ──┐
│  ┌────┐  ┌────┐  ┌────┐                                      │
│  │ VE │  │ MD │  │ HT │                                      │
│  └────┘  └────┘  └────┘                                      │
│  Vera    Marcus   Hiroshi                                     │
└───────────────────────────────────────────────────────────── ┘
```

Clicking an avatar opens a modal with the full agent card from `team.yaml`:

```
┌─ Vera Okonkwo ──────────────────────────────────────────[×]─┐
│  Supply Chain Security & LLM Analysis Engineer               │
│  ────────────────────────────────────────────────────────    │
│  Built automated package analysis pipelines at Socket.dev    │
│  and Sonatype. Knows what static analysis + multi-pass LLM   │
│  can and cannot do at registry scale.                        │
│                                                              │
│  Perspective: Trust signals over metrics. Non-gameable       │
│  signals are behavioral, not numerical.                      │
│                                                              │
│  Recruited: Session 1  ·  Contributions: 8 messages         │
└──────────────────────────────────────────────────────────── ┘
```

Avatar initials are derived from the first letter of each word in the name.
Avatar background color is deterministic from the name string (hashed).

`[+ recruit]` copies `/sdlc-recruit <slug>` to clipboard. No inline form —
recruiting is an agent action.

---

## Orientation Strip

Three lines, always visible above the dialogue stream. Written by the agent at
the end of each session via the session frontmatter. Null-safe — shows
placeholder text if no sessions exist yet.

```
┌─ Orientation ──────────────────────────────────────────────┐
│  WHERE WE ARE   Early discovery — problem shape not clear   │
│  → NEXT MOVE    Research existing approaches                │
│  COMMIT SIGNAL  When ≥2 competing designs and a tiebreaker  │
└────────────────────────────────────────────────────────────┘
```

---

## Empty State (no sessions yet)

```
│  ─────────── No sessions yet ───────────────────────────────   │
│                                                                 │
│  The agent will interview this idea, recruit thought partners,  │
│  and write the dialogue here.                                   │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  Add a seed thought (optional)...               [→]     │   │
│  └─────────────────────────────────────────────────────────┘   │
```

Sending with an empty input fires `POST /api/ponder/:slug/chat {}` (no message),
running `/sdlc-ponder <slug>` with no seed.

---

## Owner Participant Messages

When the owner types a message it is written into the session file as a
participant block using the git user name + role `Owner`:

```markdown
**JORDAN · Owner**
What about cold start — zero packages indexed,
empty state or seed with curated picks?
```

The git user name is read from `git config user.name` on the server at
request time and returned in the `POST /api/ponder/:slug/chat` response so
the UI can render the optimistic message correctly before the session file lands.

---

## Auto-scroll

On initial load, the dialogue scrolls to the bottom automatically (most recent
content). New content arriving via SSE also scrolls to the bottom, unless the
user has manually scrolled up — in that case, new content lands silently and
a `↓ new content` nudge appears at the bottom.

---

## New API Endpoints

### `POST /api/ponder/:slug/chat`

Start a ponder session, optionally seeded with an owner message.

Request:
```json
{ "message": "What about cold start..." }
```

Response:
```json
{
  "session": 3,
  "owner_name": "Jordan Washburn",
  "run_id": "ponder-installable-audit-cards-3"
}
```

Errors:
- `409 Conflict` if a session is already running for this slug

### `DELETE /api/ponder/:slug/chat/current`

Kill the running agent subprocess for this ponder.

Response: `204 No Content`

---

## New SSE Event Types

Two new event types emitted on the existing `/api/events` stream:

```json
{ "type": "ponder_run_started", "slug": "installable-audit-cards", "session": 3 }
{ "type": "ponder_run_completed", "slug": "installable-audit-cards", "session": 3 }
{ "type": "ponder_run_stopped", "slug": "installable-audit-cards", "session": 3 }
```

Existing file-change SSE events continue to fire when session files land — the
UI uses `ponder_run_completed` to know when to re-fetch, not the file event.

---

## New MCP Tool: `sdlc_ponder_chat`

Exposed as an MCP tool so agents can also trigger ponder sessions
programmatically (e.g. from `/sdlc-ponder-commit` before crystallizing).

```json
{
  "name": "sdlc_ponder_chat",
  "description": "Start a ponder session for an idea, optionally seeded with a message",
  "schema": {
    "type": "object",
    "properties": {
      "slug": {
        "type": "string",
        "description": "Ponder entry slug"
      },
      "message": {
        "type": "string",
        "description": "Seed message to start the session with. If omitted, agent opens fresh."
      }
    },
    "required": ["slug"]
  }
}
```

Output:
```json
{ "session": 3, "status": "started" }
```

---

## Session File: Owner Message Format

Owner messages are written by the server (not the agent) before the agent runs.
The server prepends the owner block to the session file immediately, then the
agent appends its response. This ensures the owner's message is in the file
even if the agent run fails.

```markdown
---
session: 3
timestamp: 2024-01-18T14:22:00Z
orientation:
  current: ""
  next: ""
  commit: ""
---

**JORDAN · Owner**
What about cold start — zero packages indexed,
empty state or seed with curated picks?

<!-- agent response begins here -->
```

The agent reads the file on startup (it's passed via `--file` or the session
path is a known convention), incorporates the owner message as context, and
appends its full response. At the end, it updates the frontmatter orientation.

---

## What We're NOT Building

- Token-level streaming within a session — sessions appear when complete
- Multi-turn chat within a single session — one owner message seeds one session
- Owner messages without triggering an agent run — the input always starts a session
- Editing past messages — the log is append-only
- Multiple concurrent sessions on the same ponder — `409` if already running
