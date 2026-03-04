# Tasks: Feature create_design action produces HTML mockup

## Task T1 — Update `needs_design` rule message in `rules.rs`

**File:** `crates/sdlc-core/src/rules.rs`

Update the `message` closure in rule `needs_design` (rule 6) to replace the current
soft suggestion about HTML prototypes with an explicit requirement:

- For UI features: must produce `mockup.html` alongside `design.md`
- For non-UI features: ASCII wireframes or plain Markdown are sufficient
- Include the file placement and `design.md` reference requirement

Also update rule `design_rejected` (rule 8) to carry the same HTML mockup guidance
so that rejected + rewritten designs also follow the updated contract.

---

## Task T2 — Update `SDLC_NEXT_COMMAND` in `sdlc_next.rs`

**File:** `crates/sdlc-cli/src/cmd/init/commands/sdlc_next.rs`

In the `SDLC_NEXT_COMMAND` string (Claude Code slash command format):

Under "For artifact creation", add a new subsection after step 3:

```
For `create_design` on a **UI feature**, also write `mockup.html`:
- Single self-contained file — inline `<style>` and `<script>`, no external resources
- Valid HTML5 with a `<nav>` or tab bar to navigate between screens
- Named sections (`<section id="screen-*">`) for each major UI state
- Representative colors and typography; pixel-perfect fidelity not required
- Reference it from `design.md`: `[Mockup](mockup.html)`
- Place the file in the same directory as `design.md`

Non-UI features (backend, CLI, config): mockup is optional; ASCII wireframes are
sufficient.
```

---

## Task T3 — Update `SDLC_NEXT_PLAYBOOK` in `sdlc_next.rs`

**File:** `crates/sdlc-cli/src/cmd/init/commands/sdlc_next.rs`

In the `SDLC_NEXT_PLAYBOOK` string (Gemini / OpenCode format):

Under step 4 "For creation actions", add a bullet:

```
- For `create_design` on a UI feature: also write `mockup.html` (self-contained,
  inline CSS/JS, named screen sections, navigation bar). Reference it from `design.md`.
```

---

## Task T4 — Update `SDLC_NEXT_SKILL` in `sdlc_next.rs`

**File:** `crates/sdlc-cli/src/cmd/init/commands/sdlc_next.rs`

In the `SDLC_NEXT_SKILL` string (Agent Skills format):

Under "For creation actions", add a step note:

```
For `create_design` on a UI feature: produce `mockup.html` alongside `design.md`.
The mockup must be self-contained (no external resources), valid HTML5, with a nav
bar and named `<section id="screen-*">` blocks for each UI state.
```

---

## Task T5 — Run tests and verify no regressions

Run:
```bash
SDLC_NO_NPM=1 cargo test --all
cargo clippy --all -- -D warnings
```

Verify:
- All existing tests pass (no rule-engine tests changed)
- No clippy warnings
- The `rules.rs` message change does not affect any test assertions (tests check
  `action` type, not message text)

---

## Acceptance Check

After all tasks complete:

1. `sdlc next --for <any-ui-feature> --json` returns a `message` that explicitly
   mentions `mockup.html` and the self-contained HTML requirement.
2. `sdlc update` installs the updated `sdlc-next` Claude Code command to
   `~/.claude/commands/sdlc-next.md` with the new HTML mockup sub-step.
3. No test failures.
