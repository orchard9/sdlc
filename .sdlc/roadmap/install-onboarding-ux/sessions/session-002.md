---
session: 2
timestamp: 2026-03-01T00:00:00Z
orientation:
  current: "Triage complete. Three Tier-1 quick-wins identified (error filenames, SSH URL, sdlc update docs). Two Tier-2 structural fixes queued (make install promotion, Vision/Architecture first-run). P4 detection deferred. Root error-filename fix located to specific lines in init/mod.rs."
  next: "Execute Tier-1 fixes: (1) add .with_context() to bare `?` calls in init/mod.rs lines 94 and 98; (2) add SSH URL variant and sdlc update section to README.md; then queue Tier-2 as features."
  commit: "When: error messages include filenames, README has SSH URL + sdlc update + make install, and the Vision/Architecture gap has at least a docs fix. Then commit this ponder and create features."
---

## Session 2: Full Triage with Thought Partners

### Context loaded

Session 1 extracted six signals from Xist's Discord session:
1. SSH multi-key install URL fails
2. Permission denied (os error 13) with no filename
3. `make install` undiscoverable
4. Vision/Architecture setup not guided
5. DEVELOPER.md not communicated from README
6. `sdlc update` undocumented

Three thought partners recruited and registered:
- **Kai Yamazaki** — DX engineer from a Rust-centric CLI toolchain
- **Xist** — the enterprise game dev who hit all three blockers
- **Mara Solberg** — documentation strategist, README-as-funnel model

---

### Issue 1: SSH Multi-Key Install URL

**Kai Yamazaki · Developer Experience Engineer**

The README currently shows this as the from-source install path:

```
cargo install --git https://github.com/orchard9/sdlc sdlc-cli
```

This is wrong for a large class of users. HTTPS GitHub URLs fail in three common enterprise scenarios: (1) SSL inspection proxies that cargo doesn't trust, (2) multi-SSH-key setups where git's credential helper doesn't know which key to use, (3) corporate firewalls that block raw HTTPS to github.com.

The SSH URL variant Xist discovered by luck:

```
cargo install --git ssh://git@github.com/orchard9/sdlc sdlc-cli
```

This routes through SSH and respects `.ssh/config` key routing. It's the canonical path for engineers with complex SSH setups.

The fix is not to replace HTTPS with SSH — it's to show both and label them. Minimally:

```markdown
**From source** (requires Rust + Node.js ≥ 18):

```bash
# Works for multi-SSH-key setups, corporate proxies:
cargo install --git ssh://git@github.com/orchard9/sdlc sdlc-cli

# Or if HTTPS works in your environment:
cargo install --git https://github.com/orchard9/sdlc sdlc-cli
```
```

But the deeper structural issue is that `cargo install --git` shouldn't be the primary source path at all — that should be `make install` after a clone, which handles the frontend build. See Issue 3.

**Xist · Enterprise Game Dev**

The problem isn't just that it failed — it's that it failed silently in a way that gives you no next step. `cargo install --git https://...` just hangs or gives you a TLS error. If you don't already know SSH URL syntax for cargo, you have no idea where to go. I found it because I knew to look, not because anything pointed me there.

⚑  Decided: Add SSH URL as the primary `cargo install --git` option in README, labeled clearly for multi-SSH-key / enterprise environments. HTTPS variant remains as a secondary note.

---

### Issue 2: `error: Permission denied (os error 13)` — No Filename

**Kai Yamazaki · Developer Experience Engineer**

This is a well-understood failure mode in Rust CLI error design, and it has a clear fix.

The root error is `std::io::Error` with `os error 13` (EACCES). The Rust standard library surfaces the OS error kind but not the path — that's by design, because the OS doesn't always know the path. The caller knows the path. The caller is responsible for adding it to the error context.

I looked at `crates/sdlc-cli/src/cmd/init/mod.rs`. There are two classes of bare `?` calls with no context on the path:

**Class 1 — The `.ai/` subdirectory creation loop (lines 92-95):**

```rust
for dir in ai_lookup_dirs {
    let p = root.join(dir);
    io::ensure_dir(&p)?;   // ← bare ?, no path in error
}
```

If any of these directory creates fail, the error is just `Permission denied (os error 13)`. The fix:

```rust
io::ensure_dir(&p).with_context(|| format!("failed to create {}", p.display()))?;
```

**Class 2 — The AI index write (line 98):**

```rust
io::write_if_missing(&index_path, AI_LOOKUP_INDEX_CONTENT.as_bytes())?;   // ← bare ?
```

Fix:

```rust
io::write_if_missing(&index_path, AI_LOOKUP_INDEX_CONTENT.as_bytes())
    .with_context(|| format!("failed to write {}", index_path.display()))?;
```

There is a subtler issue too: `Config::save()` and `State::save()` in `sdlc-core` do their own `io::atomic_write` internally, and the callers in `init/mod.rs` add `.context("failed to write config.yaml")` — but that context string doesn't include the *full path*. For someone running `sdlc init` in `/Users/xist/p4ws/project`, knowing it's `config.yaml` is marginally helpful but knowing it's `/Users/xist/p4ws/project/.sdlc/config.yaml` is far more actionable. The paths are already computed above each call — they should be in the context string.

Note on `atomic_write` itself: it uses `NamedTempFile::new_in(dir)` — so a permission failure means the *directory* itself is not writable, not the target file. The error message "cannot write to `.sdlc/config.yaml`" is slightly misleading; "cannot create temp file in `.sdlc/`" is more accurate. But for user communication, "failed to write `.sdlc/config.yaml`" is good enough — just add the path.

**Xist · Enterprise Game Dev**

Every single second of my 20-minute debug session would have been eliminated by seeing the actual path in that error. I tried `--debug` and `--verbose` — neither exist. If I had seen:

```
error: failed to create /Users/xist/p4ws/project/.ai/patterns: Permission denied (os error 13)
```

I would have immediately known: the parent directory has a permission issue. I would have fixed it in 30 seconds. Instead I spent 20 minutes guessing.

⚑  Decided: Fix the two bare `?` calls in `init/mod.rs` (`.ai/` loop and AI index write). Both are one-line changes. Additionally, wherever `Config::save()` and `State::save()` are called in `init/mod.rs`, make the context string include the full path (e.g., `config_path.display()` not just the filename).

**Specific code locations for the fix:**
- `crates/sdlc-cli/src/cmd/init/mod.rs`, line 94: `io::ensure_dir(&p)?` → add `with_context`
- `crates/sdlc-cli/src/cmd/init/mod.rs`, line 98: `io::write_if_missing(&index_path, ...)` → add `with_context`
- `crates/sdlc-cli/src/cmd/init/mod.rs`, lines 60/70: `cfg.save(root).context("failed to write config.yaml")` → change to include `config_path.display()` / `state_path.display()`

---

### Issue 3: `make install` Undiscoverable

**Mara Solberg · Documentation Strategist**

This is a documentation structure problem, not a content problem. `make install` exists, it's the right path, and it's documented in DEVELOPER.md. The problem is where it lives relative to the user's discovery flow.

A new user's journey is: find the repo → read the README → try to install → hit friction → give up or search further. DEVELOPER.md is never in this path unless README links to it explicitly. It currently doesn't.

The README install section has this structure:
1. `curl | sh` (prebuilt binary, macOS/Linux)
2. PowerShell (prebuilt binary, Windows)
3. Homebrew
4. `cargo install --git` (from source)

The "from source" path is `cargo install --git`, which requires no clone but also provides no `make install`. For a user who has already cloned (common in enterprise where people clone internal mirrors), `make install` is the right path — and it handles the frontend build step automatically, which `cargo install --git` also does, but differently.

The structural fix:

**Move `make install` to README as the primary clone-and-build path.** The README install section should distinguish between:
- **Don't need to build** → prebuilt binary (curl/brew)
- **Building from source** → clone + `make install`
- **Cargo direct install** → `cargo install --git` (for Rust users who want sdlc from cargo without cloning)

Then DEVELOPER.md focuses on the contributor dev loop (hot reload, tests, build targets), linked from README with one sentence.

**Kai Yamazaki · Developer Experience Engineer**

Agreed. `make install` is the correct abstraction — it handles both backend and frontend in one command, which `cargo install --build-script` also does but without giving the user visibility into what's happening. Enterprise users who clone internally will reach for `make install` if it's visible.

There's also a discoverability question about the Makefile itself. If a user clones the repo and does `ls`, they'll see the Makefile. `make help` or `make` showing a help target would surface `make install` immediately. That's a small Makefile improvement that complements the README fix.

⚑  Decided: Tier-2 restructure of README install section to include `make install` after `git clone`. DEVELOPER.md gets a one-line link from README. This is not Tier-1 because it requires more thought about the overall README information architecture — but it should be a feature, not left as a note.

?  Open: Are the prebuilt binary paths (`curl | sh`, `brew`) working in production today? If not, they should be removed from README or marked "coming soon" — showing broken install paths is worse than not showing them.

---

### Issue 4: Vision and Architecture — First-Run Guidance Gap

**Mara Solberg · Documentation Strategist**

This is a hybrid documentation + product problem.

**Documentation side:** Neither the README nor any in-app text explains what Vision and Architecture are in the first-run context. A user who runs `sdlc init` and then `sdlc ui` lands on a dashboard that assumes they know what these are. They don't.

The minimum fix: add a "First steps" note to the README that says explicitly:
- After `sdlc ui`, navigate to `/setup` to define your project's Vision and Architecture
- Vision explains why the project exists and who it serves — AI agents use it to make decisions
- Architecture explains how the system works — agents use it to understand constraints

**Product side:** The empty dashboard is silent. When Vision and Architecture are missing, the UI should say so. This is the classic "empty state" design problem. An empty dashboard is not "no news is good news" — it's a dead end for new users.

The right prompt is not a modal — it's a persistent banner or a prominent section in the dashboard empty state:
```
Your project hasn't defined Vision or Architecture yet.
These documents guide all agent decisions.
[Define Vision] [Define Architecture]
```

This is a UI feature, not a documentation patch.

**Xist · Enterprise Game Dev**

I looked at the dashboard and saw... nothing. Not even "here's what you can do next." I eventually found `/setup` by looking at the URL after clicking around. The setup page exists and is useful — it just isn't linked from anywhere visible.

Adding a single link from the dashboard to the setup page would have eliminated my confusion. "First time here? Start with Setup." One line.

**Kai Yamazaki · Developer Experience Engineer**

The documentation fix and the product fix are independent. Do the docs fix now (it's 10 lines in README). Queue the UI feature separately — it's real product work.

For the docs fix: the key insight is that `sdlc init` ends with:
```
SDLC initialized successfully.
Next: sdlc feature create <slug> --title "..."
```

This is wrong for first-time users. A first-time user should be told to open the UI and go to Setup before creating features. The first-run output of `sdlc init` should say:
```
SDLC initialized successfully.
Next: sdlc ui    # then visit /setup to define Vision and Architecture
```

That's a one-line change in `crates/sdlc-cli/src/cmd/init/mod.rs` at line 119.

⚑  Decided: (a) Update `sdlc init` completion message to direct users to `sdlc ui` and `/setup`. (b) Add "First steps" section to README with explicit Vision/Architecture guidance. (c) Create a feature for the UI empty-state prompt with links to setup.

---

### Issue 5: `sdlc update` Undocumented

**Mara Solberg · Documentation Strategist**

This is the simplest fix of all. A user who has installed sdlc and gotten a new version needs two things: update the binary, update the scaffolding.

The binary update is: re-run the install command (or `brew upgrade`).
The scaffolding update is: `sdlc update`.

These need to appear together in a "Updating" section in README, immediately after the Install section. Users will search for "update" in the README — this section must be findable.

**Kai Yamazaki · Developer Experience Engineer**

`sdlc update` is doing two things most users don't expect: it's refreshing the command files in `~/.claude/commands/`, `~/.gemini/commands/`, and `~/.agents/skills/`. That's not obvious from the name. The README should explain this in one sentence: "Refreshes your AI command scaffolding — run this after upgrading the sdlc binary."

⚑  Decided: Add a "Updating" section to README immediately after Install. ~8 lines. Tier 1.

---

### Issue 6: The DEVELOPER.md vs README Split

**Mara Solberg · Documentation Strategist**

The current state:
- README has install instructions (partial, HTTPS-only cargo install)
- DEVELOPER.md has the correct install instructions (`make install`) plus the dev loop, hot reload, and testing
- README does not link to DEVELOPER.md
- DEVELOPER.md is written for contributors, not users

The correct split:

**README serves:** someone who found the repo, wants to know what it is, wants to install it, and wants to get a quick win. README ends with "see DEVELOPER.md for the full contributor setup."

**DEVELOPER.md serves:** someone who is contributing to sdlc itself — building, testing, running the dev loop. This is the current content of DEVELOPER.md, which is already well-structured for that purpose.

The problem is that `make install` — which is the right path for both users and contributors doing a clone-and-install — is only in DEVELOPER.md. Moving it to README (or adding it to README) is the right fix.

?  Open: Should DEVELOPER.md have a brief preamble that says "if you just want to install sdlc (not develop it), see README"? This prevents the wrong audience from ending up in DEVELOPER.md and following the hot-reload instructions.

---

### Systemic Pattern: The Two Classes of Friction

Working through these issues, I see two distinct problem classes:

**Class A — Tool-level friction (errors, CLI UX):** The permission error, the missing paths, the missing flags. These are failures of the tool to communicate its state. They are fixable entirely in Rust code and have zero impact on existing users — they are purely additive improvements to error messages.

**Class B — Documentation-level friction (README, DEVELOPER.md, first-run):** The SSH URL, `make install`, `sdlc update`, Vision/Architecture guidance. These are failures of the documentation to serve readers who don't already know the tool. They are fixable entirely in Markdown and have zero impact on the codebase.

These two classes should be treated as parallel tracks. Class A fixes go into a single PR against `crates/sdlc-cli/src/cmd/init/mod.rs`. Class B fixes go into a single PR against `README.md` (and possibly a first-run message in `init/mod.rs`).

Neither class depends on the other. Both should be executed immediately.

**Kai Yamazaki · Developer Experience Engineer**

The pattern I see in Class A is a lack of a consistent discipline around error context. It's not that the code is wrong — it's that the convention hasn't been enforced. Every `?` at a user-visible boundary (specifically in `sdlc init`, which is the most visible command for new users) should have `.with_context()`. This isn't even a new pattern in the codebase — look at how `write_guidance_md` already does it correctly:

```rust
io::atomic_write(&path, GUIDANCE_MD_CONTENT.as_bytes())
    .with_context(|| format!("cannot write {}", path.display()))?;
```

The two missing instances are inconsistencies, not design decisions. The fix is straightforward.

**Mara Solberg · Documentation Strategist**

The pattern in Class B is that the README was written to document the tool's features, not to onboard new users. It reads like a reference document, not a funnel. The install section is buried after the CLI reference. The quickstart exists but assumes you've already successfully installed. The SSH URL issue is a symptom of writing install docs for users who already understand cargo, not for users encountering it for the first time.

The README needs an architecture of its own: what is this → install → quick win → where to go next. That's a restructure, not a line edit. The individual fixes (SSH URL, `make install`, `sdlc update`) can happen as line edits now. The restructure is a Tier-2 feature.

---

### Open Questions

?  Open: Are the prebuilt binary paths (curl, Homebrew) in README actually working? If not, they must be removed — showing broken paths is trust-destroying.

?  Open: Should DEVELOPER.md have a "not a contributor? See README for install" preamble?

?  Open: What is the correct first-run message for `sdlc init`? Currently it says "Next: sdlc feature create...". Should it be "Next: sdlc ui (then visit /setup to define Vision and Architecture)"?

?  Open: P4 detection heuristic — worth a spike? The pattern would be: check for `P4CONFIG` env var or `.p4config` in parent dirs before emitting a permission error. Low priority given the generic filename fix.

---

### Summary of Decisions

1. ⚑  Add `.with_context()` to bare `?` calls in `init/mod.rs` at the `.ai/` dir creation loop and AI index write. Also make existing `config.yaml`/`state.yaml` context strings include full paths.

2. ⚑  Add SSH URL variant to README install section as primary option for multi-key/enterprise users.

3. ⚑  Add "Updating" section to README (after Install) with `sdlc update` explanation.

4. ⚑  Update `sdlc init` completion message to direct to `sdlc ui` and `/setup`.

5. ⚑  Promote `make install` to README as primary clone-and-build path (Tier 2 — restructure needed, create as feature).

6. ⚑  Create feature for UI empty-state prompt with links to Vision/Architecture setup.

7. ⚑  Documentation-only: add "First steps" to README explaining Vision/Architecture role.

8. ⚑  P4 detection deferred — the generic error-filename fix covers the use case adequately for now.
