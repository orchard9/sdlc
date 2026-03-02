# Security Audit: AppShell Panels — NAV Icon Rail Collapse and AgentPanel Resize

## Scope

This audit covers three modified/new files:

1. `frontend/src/components/ui/tooltip.tsx` (new)
2. `frontend/src/components/layout/Sidebar.tsx` (modified)
3. `frontend/src/components/layout/AgentPanel.tsx` (modified)

And one new dependency:
- `@radix-ui/react-tooltip` (npm package)

## Attack Surface

This is a pure UI state management change. No new API endpoints, no server-side code, no authentication logic, no data access, no network calls. The security surface is minimal.

## Findings

### F1: localStorage usage — ACCEPTED (no risk)

**Finding:** Both features store user preferences in `localStorage` (`sdlc:sidebar-collapsed`, `sdlc:agent-panel-width`).

**Analysis:**
- Values written: `"true"`, `"false"` (sidebar) and an integer string (panel width).
- Values read: compared with `=== 'true'` (sidebar) and parsed with `parseInt` + `isNaN` guard (panel width).
- `localStorage` in a browser tab is same-origin scoped — no cross-origin access.
- No sensitive data is stored. Both keys contain layout preferences, not credentials, PII, or tokens.
- On parse failure (`NaN`), the code defaults to `DEFAULT_WIDTH = 288` — no injection vector.
- The integer string is written back to an inline CSS style as `width: Npx` after clamping — the value is always a number, never a raw user string. No CSS injection risk.

**Decision:** Accept. No remediation needed.

### F2: `@radix-ui/react-tooltip` dependency — ACCEPTED

**Finding:** New npm dependency added.

**Analysis:**
- `@radix-ui` is the official Radix UI organization, widely used across the React ecosystem.
- The package is already a transitive dependency through other `@radix-ui/*` packages in use (`@radix-ui/react-slot`).
- Version pinned via `package-lock.json`.
- No network requests made by this package at runtime — it is a pure React component library.
- No eval, no dynamic imports, no data exfiltration surface.

**Decision:** Accept. Consistent with existing `@radix-ui` usage in the project.

### F3: Pointer event listeners on `window` — ACCEPTED (no risk)

**Finding:** `ResizeHandle` attaches `pointermove` and `pointerup` listeners to `window` during drag.

**Analysis:**
- Listeners are added only on `pointerdown` and removed on `pointerup`. No persistent global listener leak.
- Listener functions are closures that only call `onWidthChange` and `onResizeEnd` — both are pure React state setters and localStorage writes.
- No user input is directly injected into the DOM or evaluated.

**Decision:** Accept. Standard drag-and-drop pointer capture pattern. Cleanup is correct.

### F4: No XSS vector from nav labels — CONFIRMED SAFE

**Finding:** Nav item labels are rendered as JSX text children.

**Analysis:**
- Labels are static string constants defined in the `navGroups` array in the source file.
- They are never read from user input, API responses, or localStorage.
- React escapes text children by default — no `dangerouslySetInnerHTML` used anywhere.

**Decision:** No risk. No action needed.

### F5: Tooltip content — CONFIRMED SAFE

**Finding:** `TooltipContent` receives nav item labels.

**Analysis:** Same as F4 — labels are static source constants, not user-controlled data.

**Decision:** No risk.

## Summary

| Finding | Severity | Decision |
|---|---|---|
| F1: localStorage usage | Informational | Accept |
| F2: @radix-ui/react-tooltip dependency | Informational | Accept |
| F3: Window pointer listeners during drag | Informational | Accept |
| F4: Nav label XSS surface | Informational | Confirmed safe |
| F5: Tooltip content XSS surface | Informational | Confirmed safe |

No findings require remediation. All are informational-level observations confirmed safe.

## Verdict

APPROVED. No security concerns. This change is a pure client-side UI preference feature with no server interaction, no sensitive data handling, and no new attack vectors.
