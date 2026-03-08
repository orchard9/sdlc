# Code Review: Git Status Chip

## Files Changed

1. `frontend/src/hooks/useGitStatus.ts` (new) -- Custom hook for polling git status API
2. `frontend/src/components/layout/GitStatusChip.tsx` (new) -- Sidebar chip component
3. `frontend/src/components/layout/Sidebar.tsx` (modified) -- Integration point

## Review Findings

### Correctness

- **PASS**: Hook correctly manages polling interval, visibility-based pause/resume, and focus-based re-fetch.
- **PASS**: Component correctly maps severity to color classes with appropriate Tailwind tokens.
- **PASS**: Summary text logic handles all states: conflicts, dirty, staged, ahead, clean.
- **PASS**: Commit button only renders when `staged_count > 0`.
- **PASS**: Collapsed mode shows icon-only with tooltip, expanded shows full text.
- **PASS**: TypeScript compiles cleanly with no errors.

### Code Quality

- **PASS**: Hook follows existing project patterns (similar to `useProjectState`).
- **PASS**: Component uses `cn()` utility consistently with existing sidebar buttons.
- **PASS**: No `unwrap()` equivalents or unsafe patterns.
- **PASS**: Error states handled gracefully -- grey dot, no crashes.

### Integration

- **PASS**: GitStatusChip is positioned first in the bottom utility section, above Ask Code.
- **PASS**: Respects `collapsed` prop from Sidebar.
- **PASS**: No layout shift risk -- fixed-size dot, truncated text.

### Potential Improvements (non-blocking)

- The commit POST currently sends no body (uses default message on server side). A future feature could add commit message composition.
- The polling interval (10s) is hardcoded in the hook default. Could be made configurable via props if needed.

## Verdict

**APPROVED** -- Clean implementation, follows existing patterns, TypeScript passes.
