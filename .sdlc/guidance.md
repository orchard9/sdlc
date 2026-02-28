# Engineering Guidance

Read this before any implementation, bug fix, or test action.

## North Star: Vision & Architecture

Before writing a single line of code, read:

- **`VISION.md`** — *what* we are building and *why*. Every feature, every tradeoff, every design decision must serve this vision. If a proposed change works against it, surface it before proceeding.
- **`ARCHITECTURE.md`** — *how* the system works. Components, interfaces, data flows, and sequence diagrams showing how everything fits together. Code must conform to the architecture — never silently deviate.

These are the guiding light. When in doubt about any decision, return to them first.

## 1. Build It Right

Do it the proper way — not the quick way. The correct solution is one that
will still be correct in six months. Favor proven patterns, clear
abstractions, and designs that are easy to understand and extend. Never
trade long-term correctness for short-term convenience.

## 2. Understand Bugs Before Fixing Them

Before touching a bug, trace its root cause holistically — read surrounding
code, follow the data flow, understand why it broke. Fix the cause, not the
symptom. A patch that introduces a new bug in three months is worse than
no fix.

## 3. Enterprise Quality Bar

We build enterprise-grade software. The bar is Steve Jobs: relentless
attention to detail, nothing ships that embarrasses us, correctness and
reliability are non-negotiable. If something isn't right, make it right.

## 4. Philosophy of Software Design

Follow John Ousterhout's principles: deep modules, minimal exposed
complexity, interfaces that hide implementation detail, and code readable
in isolation. Complexity is the enemy — fight it at every level.

## 5. Meaningful, Reliable, Fast Tests

Tests must earn their place. When a test breaks, choose deliberately:
- **Remove** — if it adds little value or tests implementation detail
- **Rewrite** — if it was poorly structured for the scenario
- **Refactor** — if the interface it tests changed legitimately
- **Quick-fix** — only if the fix is obvious and the test is clearly valuable

Never keep a flaky or low-value test just to preserve coverage numbers.

## 6. Using sdlc

All state lives in `.sdlc/` YAML files. **Never edit them directly** — use the CLI.
Direct edits cause deserialization failures and corrupt state.

| Action | Command |
|---|---|
| Create feature | `sdlc feature create <slug> --title "…"` |
| Get next action | `sdlc next --for <slug> --json` |
| Write artifact | Write Markdown to `output_path` from the directive |
| Submit draft | `sdlc artifact draft <slug> <type>` |
| Approve artifact | `sdlc artifact approve <slug> <type>` |
| Reject artifact | `sdlc artifact reject <slug> <type>` |
| Merge (release feature) | `sdlc merge <slug>` |
| Add task | `sdlc task add <slug> "title"` |
| Start task | `sdlc task start <slug> <task-id>` |
| Complete task | `sdlc task complete <slug> <task-id>` |
| Block task | `sdlc task block <slug> <task-id> "reason"` |
| Add comment | `sdlc comment create <slug> "body"` |
| Show feature | `sdlc feature show <slug> --json` |
| List tasks | `sdlc task list <slug>` |
| Project state | `sdlc state` |
| Survey milestone waves | `sdlc project prepare [--milestone <slug>]` |
| Mark milestone prepared | `sdlc milestone mark-prepared <slug>` |
| Project phase | `sdlc project status` |
| Escalate to human | `sdlc escalate create --kind <kind> --title "…" --context "…" [--feature <slug>]` |
| List escalations | `sdlc escalate list` |
| Resolve escalation | `sdlc escalate resolve <id> "resolution note"` |

Phases advance automatically from artifact approvals — never call `sdlc feature transition`.
The only files you write directly are Markdown artifacts to `output_path`.

## 7. SDLC Tool Suite

Project-scoped TypeScript tools in `.sdlc/tools/` — callable by agents and humans during any lifecycle phase.
Read `.sdlc/tools/tools.md` for the full list, or each tool's `README.md` for detailed docs.

| Tool | Command | Purpose |
|---|---|---|
| ama | `sdlc tool run ama --setup` then `sdlc tool run ama --question "..."` | Search codebase for relevant file excerpts |

Build a custom tool: `sdlc tool scaffold <name> "<description>"`
Update the manifest after adding/changing tools: `sdlc tool sync`

## 8. Project Secrets

Encrypted secrets live in `.sdlc/secrets/`. The encrypted files (`.age`) and key
name sidecars (`.meta.yaml`) are **safe to commit**. Plain `.env.*` files must never
be committed — they are gitignored automatically.

| Action | Command |
|---|---|
| List environments | `sdlc secrets env list` |
| List key names (no decrypt) | `sdlc secrets env names <env>` |
| Load secrets into shell | `eval $(sdlc secrets env export <env>)` |
| Set a secret | `sdlc secrets env set <env> KEY=value` |
| List authorized keys | `sdlc secrets keys list` |
| Add a key | `sdlc secrets keys add --name <n> --key "$(cat ~/.ssh/id_ed25519.pub)"` |
| Rekey after key change | `sdlc secrets keys rekey` |

**For agents:** Check `sdlc secrets env names <env>` to see which variables are
available. Load the matching env before any task or build step that needs credentials:
- Feature/local work → `eval $(sdlc secrets env export development)`
- Deploy tasks → `eval $(sdlc secrets env export production)`

Never log or hardcode secret values. Reference by env var name only (e.g. `$ANTHROPIC_API_KEY`).

**In builds:** The vault is for local and agent use only. CI/CD platforms (GitHub Actions,
etc.) manage their own secrets separately — agents cannot inject into platform CI secrets.
If a build needs a credential that must live in CI, use `secret_request` escalation (§9).

## 9. Escalating to the Human

Escalations are for **actions only a human can take**. They are rare and deliberate — not a
general-purpose communication channel. Before escalating, ask: "Can I resolve this myself?"
If yes, do it. If not, escalate.

| Kind | When to escalate | Example |
|---|---|---|
| `secret_request` | Need a credential or env var that doesn't exist | "Add STRIPE_API_KEY to production env in Secrets page" |
| `question` | Strategic decision with no clear right answer | "Should checkout support crypto payments?" |
| `vision` | Product direction is undefined or contradictory | "No vision defined — what is the milestone goal?" |
| `manual_test` | Testing requires physical interaction | "Verify Google OAuth login in production browser" |

**Do NOT escalate:** code review findings, spec ambiguity you can resolve, implementation
decisions, anything an agent can handle autonomously.

**How to escalate:**

```bash
sdlc escalate create \
  --kind secret_request \
  --title "Need OPENAI_API_KEY in .env.production" \
  --context "AI summary feature calls OpenAI in prod. Dev works with a mock. Need the real key to test end-to-end." \
  --feature my-ai-feature   # omit if not feature-specific
```

**After creating:** stop the current run immediately. If `--feature` was specified, the feature
is now gated by an auto-added Blocker comment. The escalation appears in the Dashboard under
**"Needs Your Attention"**. The human must act before the feature can proceed.

**The difference from `comment --flag blocker`:**

- `comment --flag blocker` — an implementation concern the next agent cycle might fix
- `sdlc escalate create` — an action only a human can perform; stop until resolved

## 10. Frontend API Calls

Never hardcode `http://localhost:PORT` in frontend code — CORS blocks cross-origin
requests in development and the address is wrong in production.

**Pattern:**
- Use a relative base URL (`/api`) in all fetch/client code
- Configure the dev server proxy (Vite `server.proxy`, Next.js `rewrites`,
  webpack `devServer.proxy`) to forward `/api` → `http://localhost:<API_PORT>`
- In production, frontend and API share the same origin — relative paths resolve correctly

When fixing a CORS error or adding a new API client, apply this pattern instead of
adding CORS headers or introducing environment-specific URLs.
