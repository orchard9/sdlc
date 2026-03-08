# Spec: Weekly Rotating Quote System for Clean Git State

## Summary

When the git status API reports an all-green state (no dirty files, no conflicts, not behind remote), the git status chip area displays a rotating motivational quote. Quotes are embedded in the frontend bundle (no API call) and rotate on a 7-day cycle derived from the current week number.

## Requirements

### Functional

1. **Quote corpus**: A static array of at least 12 motivational/developer quotes embedded in the frontend source code. Each quote has a `text` and `author` field.
2. **Weekly rotation**: The displayed quote changes once per week, determined by `Math.floor(Date.now() / (7 * 24 * 60 * 60 * 1000)) % quotes.length`. All users see the same quote during the same calendar week.
3. **Display trigger**: The quote is shown only when the git status severity is `green` (clean working tree, synced with remote, no conflicts).
4. **Presentation**: The quote renders inside the git status chip region as a styled blockquote with author attribution. It replaces the normal status summary text when the state is green.
5. **No network dependency**: Quotes are bundled at build time. No API endpoint is required.

### Non-Functional

1. The quote array is a standalone module (`quotes.ts`) that can be extended without touching rendering logic.
2. The component accepts quotes as a prop for testability (default: the built-in corpus).
3. Bundle size impact is negligible (< 1 KB for the quote corpus).

## Out of Scope

- User-customizable quotes or favorites.
- Quote fetching from an external service.
- Animations or transitions between quotes.

## Dependencies

- `git-status-api` — provides the severity field that triggers quote display.
- `git-status-chip` — provides the UI container where the quote renders.

## Acceptance Criteria

1. When git status severity is `green`, a quote with author attribution is visible in the status chip area.
2. Refreshing the page during the same week shows the same quote.
3. When git status severity is not `green`, no quote is displayed.
4. The quote corpus contains at least 12 entries.
