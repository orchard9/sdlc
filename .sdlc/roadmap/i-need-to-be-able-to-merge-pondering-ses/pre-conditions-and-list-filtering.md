## Pre-conditions for `sdlc ponder merge`

```
1. Source exists
2. Target exists
3. Source ≠ target
4. Source has no merged_into already set (not already absorbed)
5. Source is not committed
6. Target is not parked
7. Target is not committed
```

**Source = committed rejection message:**
`Cannot merge '<source>' — it is committed. Use git to trace its history.`

**Target = committed rejection message:**
`Cannot merge into '<target>' — it is committed. Target must be active (exploring or converging).`

---

## `ponder list` filtering

**Default behavior:** hide entries where `merged_into` is set.
Regular `parked` entries (no `merged_into`) continue to show.

**`--all` flag:** show everything, including absorbed entries.
STATUS column shows `↗ parked→<target-slug>` for merged entries.

**`ponder show` redirect banner:**
When `merged_into` is set, display at top:
```
⚠  This entry was merged into '<target>'. Run: sdlc ponder show <target>
```
Then continue with normal show output (agents may still need artifact list).
