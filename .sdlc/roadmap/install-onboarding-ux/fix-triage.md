# Fix Triage: install-onboarding-ux

Prioritized by: impact × effort inverse (quick wins first, structural second, deferred last).

---

## Tier 1 — Do immediately (high impact, low effort, code + docs)

### 1. Add filenames to all IO errors in `sdlc init`

**Problem:** `error: Permission denied (os error 13)` provides no path. Users cannot debug.

**Root cause:** Several calls in `crates/sdlc-cli/src/cmd/init/mod.rs` use bare `?` with no `.with_context()`:
- Line 94: `io::ensure_dir(&p)?;` — the `.ai/` subdirectory creation loop has no context on `p`
- Line 98: `io::write_if_missing(&index_path, AI_LOOKUP_INDEX_CONTENT.as_bytes())?;` — no path in error
- `Config::save()` in `crates/sdlc-core/src/config.rs` (line 220) calls `io::atomic_write` which returns `SdlcError::Io` — the caller in `init/mod.rs` adds `.context("failed to write config.yaml")` (line 60) but that context string doesn't include the full path
- `State::save()` same pattern (line 70)

**Note on `atomic_write` itself:** `io::atomic_write` in `crates/sdlc-core/src/io.rs` uses `NamedTempFile::new_in(dir)` — a permission failure here means the *directory* is not writable. The error is `os error 13` from the tempfile crate, with no path. The fix must be at the call site (caller wraps with `.with_context(|| format!("...: {}", path.display()))`), not inside `atomic_write`.

**Specific fixes needed in `init/mod.rs`:**
1. The `.ai/` dir creation loop (lines 92-95): change `io::ensure_dir(&p)?;` to `io::ensure_dir(&p).with_context(|| format!("failed to create {}", p.display()))?;`
2. The `io::write_if_missing` call for the AI index (line 98): add `.with_context(|| format!("failed to write {}", index_path.display()))?`

**Impact:** Every user who hits a permission error during init — including all P4/enterprise users — gets a debuggable message.

**Effort:** 2 lines changed. Zero architecture change.

---

### 2. Add SSH URL variant to README install section

**Problem:** `cargo install --git https://github.com/...` fails for multi-SSH-key users. The SSH URL variant works but is not documented.

**The correct SSH URL:**
```
cargo install --git ssh://git@github.com/orchard9/sdlc sdlc-cli
```

**Fix:** In `README.md`, the "From source" install option should show both:
```
# Works for all setups (multiple SSH keys, no HTTPS proxy issues):
cargo install --git ssh://git@github.com/orchard9/sdlc sdlc-cli

# Or if you prefer HTTPS:
cargo install --git https://github.com/orchard9/sdlc sdlc-cli
```

**Better fix:** Make `make install` the primary "from source" path (see Tier 2 below) and relegate `cargo install --git` to a secondary note. But the minimum viable fix is adding the SSH URL alongside the HTTPS one.

**Impact:** Unblocks the entire class of multi-SSH-key users who hit this immediately.

**Effort:** 3-5 lines in README.md.

---

### 3. Add `sdlc update` to README (updating/upgrading section)

**Problem:** "When you make changes, how do I get the changed?" — no answer is documented in a visible location.

**Fix:** Add a "Updating" section directly below the Install section in README.md:
```markdown
## Updating

Re-run the install command to get the latest binary. Then refresh your agent scaffolding:

```bash
sdlc update
```

`sdlc update` rewrites your `~/.claude/commands/`, `~/.gemini/commands/`, and `~/.agents/skills/` files with the latest sdlc command templates.
```

**Impact:** Answers the second question every new user asks.

**Effort:** ~8 lines in README.md.

---

## Tier 2 — Do next (structural, medium effort)

### 4. Promote `make install` as the primary source-install path in README

**Problem:** `make install` handles the frontend build step automatically — it is the correct "build from source" path. It is only documented in DEVELOPER.md, which is not linked from README.

**Current README structure:** `cargo install --git` is the only from-source path shown.

**Fix:** Add `make install` to the README as the primary from-source path. Recommended structure:

```
## Install

**Download a prebuilt binary (no prerequisites):**
[macOS/Linux curl command]
[Homebrew]

**From source** (requires Rust + Node.js ≥ 18):
git clone https://github.com/orchard9/sdlc
cd sdlc
make install
```

Then DEVELOPER.md link: "See DEVELOPER.md for the dev loop, testing, and build targets."

**Impact:** Engineers who clone and build (common in enterprise environments where `cargo install --git` may be blocked) have a clear path.

**Effort:** ~20 lines in README, 1 line linking to DEVELOPER.md.

---

### 5. Add a "Vision and Architecture" first-run prompt in the UI

**Problem:** New users open the dashboard and see an empty state with no guidance. Vision and Architecture aren't mentioned in the first-run context.

**Two-part fix:**

**Documentation side (fast):** Add a "First run" section to README (or reference `/setup`):
```
After `sdlc ui`, open the Setup page (/setup) to define your project's Vision and Architecture.
These are the guiding documents for all agent decisions.
```

**Product side (slower, higher impact):** The dashboard empty state should detect if Vision and Architecture are missing and show a call to action:
- "Your project has no Vision defined. [Define Vision]" → links to `/setup` or triggers `/sdlc-init` guidance
- This is a UI change, not a docs change

**Decision:** The documentation fix is Tier 2 (straightforward). The product fix (empty-state prompt) is worth doing as a feature — create an sdlc feature for it.

**Impact:** Eliminates the "what do I do for Vision and Architecture?" question.

**Effort:** Documentation: ~10 lines. Product UI: a feature, estimate 2-4 hours.

---

### 6. P4 detection: targeted error message for read-only writes

**Problem:** In Perforce environments, files that look writable may live in directories with restricted permissions. Xist hit this. A generic "Permission denied" gives no recovery path.

**Opportunity:** When `sdlc init` hits a permission error on a specific path, it could inspect whether that path appears to be under a P4 workspace (e.g., check for a `.p4` or `P4CONFIG` marker in parent directories). If so, emit:

```
error: Permission denied writing to <path>
This directory may be managed by Perforce. If so, ensure the workspace root is writable:
  p4 edit <path>
  # or check workspace permissions in P4Admin
```

**Assessment:** This is useful but requires: (a) reliable P4 detection heuristic, (b) non-P4 users should not see the message. The detection heuristic could be checking for `P4CONFIG` env var or `.p4config` file in parent dirs.

**Decision:** Do not implement in Tier 1. The generic fix (add path to all error messages, Tier 1 item #1) already gives users a debuggable path. The P4-specific message is additive polish.

**Effort:** Medium (heuristic + error formatting). Park for now.

---

## Tier 3 — Deferred (nice-to-have, low urgency)

### 7. `--debug` / `--verbose` flags for sdlc init

**Xist tried this.** These flags don't exist in sdlc. The "fix" in Tier 1 (filenames in errors) eliminates the need for most users. A proper `--debug` flag (that dumps all paths being written) is nice but is not the right lever — the error message fix is.

**Decision:** Defer. Once Tier 1 fixes are in, the most common debug need goes away.

---

### 8. Prebuilt binary for enterprise users

**Current state:** The README mentions `curl | sh` and `brew` but those are described as "no prerequisites" paths. If these are real and working, they should be the first thing shown. If they are placeholder text, they should be removed or marked "coming soon."

**Question to resolve:** Are the prebuilt binary install paths (`curl | sh`, `brew`) actually working today? If yes, promote them to the top of the install section — they are the right path for non-contributors. If no, remove them to avoid confusion.

**Decision:** Verify first, then act. Not a code change.

---

## Summary table

| # | Fix | Tier | File(s) | Effort |
|---|-----|------|---------|--------|
| 1 | Add paths to IO errors in sdlc init | 1 | `crates/sdlc-cli/src/cmd/init/mod.rs` | 2 lines |
| 2 | SSH URL in README install section | 1 | `README.md` | 3–5 lines |
| 3 | `sdlc update` in README | 1 | `README.md` | ~8 lines |
| 4 | `make install` as primary source path in README | 2 | `README.md`, `DEVELOPER.md` | ~20 lines |
| 5 | Vision/Architecture first-run guidance (docs) | 2 | `README.md`, UI | docs fast; UI = feature |
| 6 | P4 detection + targeted error message | 2 | `crates/sdlc-cli/src/cmd/init/mod.rs` | medium |
| 7 | `--debug` / `--verbose` flags | 3 | `crates/sdlc-cli/src/cmd/` | defer |
| 8 | Verify prebuilt binary paths | 3 | `README.md` | verify first |
