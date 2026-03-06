# Security Audit: UnifiedDialoguePanel

## Scope

This audit covers the `unified-dialogue-panel` refactor:
- `frontend/src/components/shared/UnifiedDialoguePanel.tsx` (new shared component)
- `frontend/src/components/ponder/DialoguePanel.tsx` (thin wrapper, rewritten)
- `frontend/src/components/investigation/InvestigationDialoguePanel.tsx` (thin wrapper, rewritten)

This is a pure frontend refactor — no server-side code, no API endpoints, no data schema, and no authentication paths changed.

## Security Surface

**Attack surface is minimal.** This change:
- Adds no new API endpoints
- Changes no server-side request handling
- Introduces no new data persistence
- Introduces no new authentication or authorization logic
- Does not add new user input handling beyond what previously existed

The only user input accepted is the chat message in the `InputBar` component, which was present in both original components before this refactor.

## Findings

### FINDING 1 — User chat input passes through React's JSX rendering (safe) [INFO]

**Location:** `UnifiedDialoguePanel.tsx` — `pendingMessage.text` rendered in JSX (`line 435`)

**Observation:** The pending-message overlay displays the user's typed message:
```tsx
<p className="text-sm text-foreground/80 leading-relaxed">{pendingMessage.text}</p>
```

React automatically escapes string content in JSX. There is no `dangerouslySetInnerHTML`. XSS via message content is not possible through this rendering path.

**Assessment:** Safe. No action needed.

### FINDING 2 — `mcpLabel` string rendered in JSX without sanitization [INFO]

**Location:** `UnifiedDialoguePanel.tsx` — `McpCallCard` renders `mcpLabel` as `{mcpLabel}`

**Observation:** The `mcpLabel` prop is sourced from module-level adapter constants (`'sdlc_ponder_chat'` and `'sdlc_investigation_chat'`), not from user input or API data. These are static string literals defined in the source code.

**Assessment:** No injection risk. Static, developer-controlled strings only.

### FINDING 3 — SSE event handler filters by slug before acting [INFO]

**Location:** `UnifiedDialoguePanel.tsx` lines 278–305

**Observation:** Both SSE handlers (`handlePonderEvent`, `handleInvestigationEvent`) check `event.slug !== slug` before applying state changes. This prevents a rogue or cross-slug SSE event from corrupting the run state of an unrelated panel.

**Assessment:** Correct and safe. Defense-in-depth for event routing already in place.

### FINDING 4 — Adapter `sseEventType` determines which handler activates [INFO]

**Location:** `UnifiedDialoguePanel.tsx` lines 317–322

**Observation:** The unified panel wires `useSSE` with only the handler matching `adapter.sseEventType` as non-undefined. Since adapters are module-level constants, there is no runtime path where a user could influence which SSE event family is subscribed to.

**Assessment:** Safe. No escalation path.

### FINDING 5 — `api.startPonderChat` called directly from emptyState button [INFO]

**Location:** `DialoguePanel.tsx` line 85 — zero-state "Start from title & brief" button

**Observation:** This calls the API client directly without going through the adapter, bypassing the optimistic UI state update. This is a pre-existing pattern and was noted in the code review. The API call itself uses the same authenticated client as all other API interactions — there is no new authentication bypass.

**Assessment:** No security concern. This is a UX limitation (optimistic overlay), not a security finding.

## Overall Assessment

**No security concerns found.** This is an internal frontend refactoring that consolidates duplicate UI logic into a shared component. The security surface is unchanged from the pre-refactor state:

- No new user input vectors
- No new API calls or endpoints
- No changes to authentication, authorization, or session management
- React JSX rendering provides built-in XSS protection for all user-supplied content
- SSE event routing retains slug-based filtering

The refactor reduces overall code surface and eliminates drift between two near-identical components, which is a net security improvement (fewer places for future vulnerabilities to diverge).

**Verdict: PASS — no action items required.**
