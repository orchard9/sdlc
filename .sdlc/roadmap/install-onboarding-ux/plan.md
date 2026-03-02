# Plan: Install and Onboarding Polish

## Milestone

**Slug:** `install-onboarding-polish`
**Title:** Installation and Onboarding Polish
**Vision:** A new developer can install SDLC, run `sdlc init`, and understand what to do next — without hitting undocumented friction or opaque error messages.

### Why this matters

Every new user hits the install and first-run flow before they see any value from the tool. One failed install means the tool is abandoned. The triage sessions with Kai Yamazaki (DX engineer), Xist (enterprise game dev who hit all three blockers), and Mara Solberg (documentation strategist) identified six concrete friction points — all fixable with small, targeted changes.

## Features

### Feature 1: `init-error-filenames`
**Title:** Add File Paths to Init Error Messages
**Track:** Class A (code) — one PR against `crates/sdlc-cli/src/cmd/init/mod.rs`

The root cause: bare `?` calls in the `.ai/` directory creation loop and AI index write emit `Permission denied (os error 13)` with no path context. A user running `sdlc init` in a P4/readonly directory gets an opaque error with no actionable information.

Specific changes:
1. `crates/sdlc-cli/src/cmd/init/mod.rs`, the `.ai/` dir loop — add `.with_context(|| format!("failed to create {}", p.display()))` to `io::ensure_dir(&p)?`
2. `crates/sdlc-cli/src/cmd/init/mod.rs`, the AI index write — add `.with_context(|| format!("failed to write {}", index_path.display()))` to `io::write_if_missing(&index_path, ...)?`
3. Anywhere `Config::save()` and `State::save()` are called in `init/mod.rs`, make the context string include the full path (e.g., use `config_path.display()` not just `"failed to write config.yaml"`)

Expected result: `error: failed to create /Users/xist/p4ws/project/.ai/patterns: Permission denied (os error 13)`

### Feature 2: `readme-install-guide`
**Title:** SSH and make install in README
**Track:** Class B (docs) — README.md

Two problems with the current install section:
- `cargo install --git https://github.com/...` fails silently for multi-SSH-key setups and corporate proxies
- `make install` (which handles both backend + frontend in one command) is only in DEVELOPER.md, never linked from README

Changes:
1. Add SSH URL as the primary `cargo install --git` option, labeled for enterprise/multi-key environments: `cargo install --git ssh://git@github.com/orchard9/sdlc sdlc-cli`
2. Add `make install` after `git clone` as the primary from-source path in README
3. Add a one-sentence link from README to DEVELOPER.md: "See DEVELOPER.md for the full contributor dev setup"

### Feature 3: `sdlc-update-docs`
**Title:** Document sdlc update as Update Mechanism
**Track:** Class B (docs) — README.md + init.rs completion message

`sdlc update` exists but is entirely undiscoverable. Users who upgrade the binary don't know they also need to refresh their AI command scaffolding.

Changes:
1. Add "Updating" section to README immediately after Install (~8 lines): how to re-install the binary + `sdlc update` to refresh scaffolding
2. README should explain what `sdlc update` does: "Refreshes your AI command scaffolding — run this after upgrading the sdlc binary"

### Feature 4: `setup-vision-arch-guidance`
**Title:** Vision and Architecture Guidance in Setup
**Track:** Class B (docs + init message)

After `sdlc init`, users are told "Next: sdlc feature create...". This is wrong for first-time users. They should open the UI and go to Setup first to define Vision and Architecture. Neither the README nor the UI guides them there.

Changes:
1. Update `sdlc init` completion message in `crates/sdlc-cli/src/cmd/init/mod.rs` to: "Next: sdlc ui (then visit /setup to define Vision and Architecture)"
2. Add "First steps" section to README explaining what Vision and Architecture are and why they matter before creating features
3. Add subtitle/description text to Vision and Architecture fields in the setup UI so users understand what to write

## Rationale

Two parallel tracks — code and docs — neither depends on the other:
- Class A (Features 1): Rust error message improvements. One PR, no docs impact.
- Class B (Features 2, 3, 4): README and first-run message improvements. No code impact except init message.

All four features are small-to-medium in scope. No new infrastructure needed.
