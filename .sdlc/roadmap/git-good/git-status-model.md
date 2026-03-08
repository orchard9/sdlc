# Git Status Model

## Composite State (not an enum)

The repo status is a composite of independent boolean/numeric fields, not a single enum.
Multiple conditions can be true simultaneously.

```rust
struct GitStatus {
    branch: String,
    has_uncommitted_changes: bool,   // working tree dirty
    has_staged_changes: bool,        // index differs from HEAD
    unpushed_commits: u32,           // ahead of origin
    unpulled_commits: u32,           // behind origin
    detached: bool,
    merge_in_progress: bool,
    rebase_in_progress: bool,
    has_remote: bool,                // remote configured
}
```

## UI Severity Mapping

| Level | Condition | UI Treatment |
|-------|-----------|-------------|
| Green | All false/zero | Fun celebration state, no action needed |
| Yellow | uncommitted OR unpushed (no conflict) | Commit button active |
| Red | merge/rebase in progress OR diverged | Warning with specific action |

## Polling Strategy

- `git status` is local-only, cheap — poll every 5s
- `git fetch` is network — poll every 60s, cache result
- SSE push on any agent commit (we know when `/sdlc-commit` runs)

⚑ Decided: Composite model, not enum — states are independent and combinable
⚑ Decided: Green/Yellow/Red severity levels for the status indicator
? Open: Exact polling intervals — need to test latency impact