# Code Review: dev-driver-run-actions-flag

## Summary

This review covers the implementation of `sdlc ui --run-actions`: inverting the orchestrator-start default from opt-out (`--no-orchestrate`) to opt-in (`--run-actions`).

**Files changed:**
- `crates/sdlc-cli/src/main.rs` — flag rename in `Commands::Ui` variant and dispatch arm
- `crates/sdlc-cli/src/cmd/ui.rs` — flag rename in `UiSubcommand::Start`, `run()`, `run_start()`, and spawn condition inversion

**No other files were changed.** `DEVELOPER.md` had no prior mention of `--no-orchestrate` (already clean).

---

## Changes Reviewed

### `crates/sdlc-cli/src/main.rs`

```diff
-        /// Skip starting the orchestrator daemon
+        /// Start the orchestrator daemon and execute scheduled actions
         #[arg(long)]
-        no_orchestrate: bool,
+        run_actions: bool,
```

And in the dispatch arm:

```diff
-            no_orchestrate,
+            run_actions,
         } => cmd::ui::run(
             &root,
             subcommand,
             port,
             no_open,
             tunnel,
             tick_rate,
-            no_orchestrate,
+            run_actions,
         ),
```

Verdict: Correct. Field name and doc comment updated consistently. Clap will expose `--run-actions` on the CLI.

---

### `crates/sdlc-cli/src/cmd/ui.rs`

**`UiSubcommand::Start` variant:**

```diff
-        /// Skip starting the orchestrator daemon
+        /// Start the orchestrator daemon and execute scheduled actions
         #[arg(long)]
-        no_orchestrate: bool,
+        run_actions: bool,
```

Verdict: Correct. Both the top-level `sdlc ui` and `sdlc ui start` forms now use `--run-actions`.

**`run()` signature and dispatch:**

```diff
-    no_orchestrate: bool,
+    run_actions: bool,
 ) -> Result<()> {
     match subcommand {
-        None => run_start(root, port, no_open, tunnel, tick_rate, no_orchestrate),
-        Some(UiSubcommand::Start { port: p, no_open: n, tunnel: t, tick_rate: tr, no_orchestrate: no_orch })
-            => run_start(root, p, n, t, tr, no_orch),
+        None => run_start(root, port, no_open, tunnel, tick_rate, run_actions),
+        Some(UiSubcommand::Start { port: p, no_open: n, tunnel: t, tick_rate: tr, run_actions: ra })
+            => run_start(root, p, n, t, tr, ra),
```

Verdict: Correct. Both paths pass `run_actions` consistently to `run_start`.

**`run_start()` — spawn condition:**

```diff
-fn run_start(root, port, no_open, use_tunnel, tick_rate, no_orchestrate) -> Result<()> {
+fn run_start(root, port, no_open, use_tunnel, tick_rate, run_actions) -> Result<()> {

-    if !no_orchestrate {
+    if run_actions {
         // spawn orchestrator thread
     }
```

Verdict: Correct and intentional. The inversion is exactly the semantic change required. When `run_actions` is `false` (the default), the orchestrator does NOT start. When `true`, it does.

---

## Findings

| # | Severity | Finding | Disposition |
|---|---|---|---|
| F1 | INFO | Pre-existing `sdlc-server` build error (`SdlcError::TelegramTokenMissing` not covered in error.rs) | Pre-existing, unrelated to this feature. Not introduced by this change. |
| F2 | INFO | `--no-orchestrate` references remain in sdlc artifact Markdown files (spec, design, tasks) as historical documentation | Acceptable — these are design documents that explain the before/after, not live code. |

No blockers identified.

---

## Acceptance Criteria Verification

| AC | Status | Evidence |
|---|---|---|
| AC-1: `sdlc ui` without flag does not start orchestrator | PASS | `if run_actions {` guard; `run_actions` defaults to `false` via clap |
| AC-2: `sdlc ui --run-actions` starts orchestrator | PASS | `if run_actions {` guard is entered when flag is present |
| AC-3: `--no-orchestrate` is removed (clap error) | PASS | Field removed from both `Commands::Ui` and `UiSubcommand::Start` |
| AC-4: `sdlc ui start --run-actions` works | PASS | `UiSubcommand::Start` updated identically |
| AC-5: DEVELOPER.md updated | PASS | DEVELOPER.md had no `--no-orchestrate` mentions; confirmed via grep |
| AC-6: Other flags unaffected | PASS | `--tunnel`, `--no-open`, `--port`, `--tick-rate` unchanged |

---

## Review Decision

**APPROVED.** The implementation is correct, complete, and minimal. All acceptance criteria are met. The pre-existing sdlc-server error is not caused by this feature.
