# Security Audit: Knowledge Research Modal and Research Button on List View

## Scope

Frontend-only change. Two files:
- `frontend/src/components/knowledge/NewResearchModal.tsx` (new)
- `frontend/src/pages/KnowledgePage.tsx` (modified)

No server-side changes. No new API endpoints. No new data models.

---

## Security Surface

### User-Controlled Input

The modal accepts a single free-text "topic hint" field. This value is sent as the `topic` field in `POST /api/knowledge/:slug/research` — an existing endpoint that already accepts this parameter.

**Finding 1 — Topic input is not sanitized in the UI.**
Action: ACCEPT. The `topic` field is a plain string sent in a JSON body over the existing authenticated channel. It is not rendered back as HTML anywhere in the UI — it is passed to a backend agent prompt where it is used as a research focus. XSS via this field is not possible. The backend is responsible for prompt injection defenses (pre-existing scope). No client-side sanitization is needed.

### Authentication / Authorization

No change. The research endpoint is behind the existing server auth middleware (token/cookie gate with local bypass). The new UI button is just a new code path to the same existing endpoint.

**Finding 2 — No new unauthenticated surface.**
Action: ACCEPT. Confirmed by reading `POST /api/knowledge/:slug/research` in `knowledge.rs` — it uses `State(app)` from the same authenticated router as all other knowledge endpoints.

### XSS

**Finding 3 — `entryTitle` is rendered directly in JSX.**
`<span ... title={entryTitle}>Research: {entryTitle}</span>` — React escapes this automatically. No `dangerouslySetInnerHTML`. No XSS vector.
Action: ACCEPT.

**Finding 4 — `error` state is rendered directly in JSX.**
Error message comes from `err.message` on a caught API exception — not from server-rendered HTML. Rendered as `<p className="text-xs text-destructive">{error}</p>` — React-escaped plain text.
Action: ACCEPT.

### Event Propagation

**Finding 5 — `e.stopPropagation()` on Research button click.**
Used to prevent the row select handler from firing when the Research button is clicked. This is correct and intentional. No security concern.
Action: ACCEPT.

### No New Dependencies

No new npm packages introduced. `FlaskConical` is from the existing `lucide-react` dependency.

---

## Verdict

**APPROVED.** No security findings requiring action. The change introduces no new attack surface beyond what already exists in the knowledge research endpoint.
