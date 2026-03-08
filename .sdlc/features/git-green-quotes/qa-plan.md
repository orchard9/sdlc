# QA Plan: git-green-quotes

## Scope

Validate the quote corpus, weekly rotation logic, and component rendering.

## Test Cases

### TC1: Quote corpus completeness
- **Verify**: `QUOTES` array contains at least 12 entries.
- **Verify**: Every entry has non-empty `text` and `author` fields.

### TC2: Weekly rotation determinism
- **Verify**: Calling `getWeeklyQuote()` twice within the same millisecond returns the same quote.
- **Verify**: The index is derived from `Math.floor(Date.now() / 604800000) % quotes.length`.

### TC3: Rotation over time
- **Verify**: Two timestamps exactly 7 days apart produce different quote indices (unless the corpus has exactly 1 entry).

### TC4: GitGreenQuote renders correctly
- **Verify**: The component renders the quote text in italic.
- **Verify**: The component renders the author with an em-dash prefix.

### TC5: No quote when severity is not green
- **Verify**: Integration point in `GitStatusChip` does not render `GitGreenQuote` when severity is `yellow` or `red`. (This is validated at the chip level, not within this feature's tests.)

## Pass Criteria

All TC1-TC4 pass. TC5 is deferred to `git-status-chip` integration testing.
