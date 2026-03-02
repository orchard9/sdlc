# UAT Run — Reference consumer implementations
**Date:** 2026-03-02T03:55:55Z
**Agent:** claude-sonnet-4-6
**Verdict:** PASS WITH TASKS

---

- [x] `sdlc-agent run <slug>` runs the directive loop until a human gate or `done` _(2026-03-02T03:55:55Z)_
- [x] All SDLC operations go through typed MCP tools (7 tools: get_directive, write_artifact, approve_artifact, reject_artifact, add_task, complete_task, add_comment) _(2026-03-02T03:55:55Z)_
- [x] Claude receives the full directive `message` as its prompt for each action _(2026-03-02T03:55:55Z)_
- [x] Shell gates are executed before any `sdlc_approve_artifact` call succeeds _(2026-03-02T03:55:55Z)_
- [x] Human gates pause execution and print a clear resume message _(2026-03-02T03:55:55Z)_
- [x] Session context persists at `.sdlc/features/<slug>/.agent-session` across iterations _(2026-03-02T03:55:55Z)_
- [x] Correct specialized agent selected per ActionType (9 mappings verified at runtime) _(2026-03-02T03:55:55Z)_
- [x] `--model` flag overrides the default model (claude-sonnet-4-6) _(2026-03-02T03:55:55Z)_
- [x] Opus-level agents (create_review, create_audit) default to `claude-opus-4-6` _(2026-03-02T03:55:55Z)_
- [x] `run-all` discovers features needing work via `listFeatures`, filters released/merge phases _(2026-03-02T03:55:55Z)_
- [x] All output streamable via `onMessage` callback wired to Claude Agent SDK _(2026-03-02T03:55:55Z)_
- [ ] ~~Package works with both CLI mode and REST mode (`--rest <url>`)~~ _(✗ task claude-code-consumer#T2 — REST mode deferred; spec Open Questions note it as Phase 5 server integration)_

---

**Tasks created:** claude-code-consumer#T2
**11/12 steps passed**
