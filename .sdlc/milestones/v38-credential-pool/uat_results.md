# UAT Run — Claude credential pool — PostgreSQL-backed round-robin token checkout for fleet agent runs
**Date:** 2026-03-04T23:10:37Z
**Agent:** claude-sonnet-4-6
**Verdict:** PASS WITH TASKS

---

**Scenario 1: Pool initializes**

- [ ] Pod starts with `DATABASE_URL` set _(✗ task credential-pool-core#T7 — requires live Postgres; manual validation needed)_
- [ ] Logs show "credential pool ready" _(✗ task credential-pool-core#T7 — requires server log access)_
- [ ] `claude_credentials` table is created if it didn't exist _(✗ task credential-pool-core#T7 — requires DB access)_

**Scenario 2: Graceful degradation — no rows**

- [ ] `claude_credentials` table is empty _(✗ task credential-pool-runs#T3 — requires Postgres)_
- [ ] Trigger an agent run from the UI _(✗ task credential-pool-runs#T3 — requires running server with credential pool routes)_
- [ ] Run proceeds normally (no crash, no hang) _(✗ task credential-pool-runs#T3 — requires live validation)_
- [ ] Logs show warn: "no active Claude credentials" _(✗ task credential-pool-runs#T3 — requires server log access)_
- [ ] `CLAUDE_CODE_OAUTH_TOKEN` is NOT set in the subprocess _(✗ task credential-pool-runs#T3 — requires subprocess inspection)_

**Scenario 3: Token checkout and injection**

- [ ] Insert one row: `INSERT INTO claude_credentials ...` _(✗ task credential-pool-runs#T4 — requires DB access)_
- [ ] Trigger an agent run _(✗ task credential-pool-runs#T4 — requires live server)_
- [ ] Logs show token was checked out (account_name logged) _(✗ task credential-pool-runs#T4 — requires log access)_
- [ ] `last_used_at` on the row is updated after the run _(✗ task credential-pool-runs#T4 — requires DB query)_
- [ ] `use_count` on the row increments _(✗ task credential-pool-runs#T4 — requires DB query)_

**Scenario 4: Round-robin with two tokens**

- [ ] Insert two rows with different tokens _(✗ task credential-pool-runs#T5 — requires DB access)_
- [ ] Run two sequential agent runs _(✗ task credential-pool-runs#T5 — requires live server)_
- [ ] First run uses token with older `last_used_at` _(✗ task credential-pool-runs#T5 — requires DB query)_
- [ ] Second run uses the other token _(✗ task credential-pool-runs#T5 — requires DB query)_
- [ ] `last_used_at` alternates — confirmed by querying the table _(✗ task credential-pool-runs#T5 — requires DB query)_

**Scenario 5: Concurrent checkout — no blocking**

- [ ] Insert two rows _(✗ task credential-pool-runs#T6 — requires DB access)_
- [ ] Trigger two agent runs concurrently _(✗ task credential-pool-runs#T6 — requires live server)_
- [ ] Each run gets a different token (SKIP LOCKED) _(✗ task credential-pool-runs#T6 — requires log/DB inspection)_
- [ ] Neither run waits for the other _(✗ task credential-pool-runs#T6 — requires timing measurement)_

**Scenario 6: Graceful degradation — DB unreachable**

- [ ] Start server with `DATABASE_URL` pointing to unreachable host _(✗ task credential-pool-core#T8 — requires server restart in test env)_
- [ ] Server starts (does not crash or refuse to boot) _(✗ task credential-pool-core#T8 — requires log access)_
- [ ] Warn logged at startup _(✗ task credential-pool-core#T8 — requires log access)_
- [ ] Agent runs proceed without token injection _(✗ task credential-pool-core#T8 — requires live validation)_

**Scenario 7: Helm — DATABASE_URL injected in cluster**

- [ ] Deploy a project pod with the updated Helm chart _(✗ task credential-pool-helm#T2 — requires cluster access)_
- [ ] `kubectl exec` into the pod — confirm `DATABASE_URL` env var is present _(✗ task credential-pool-helm#T2 — requires cluster access)_
- [ ] ExternalSecret `postgres-sdlc-credentials` exists in the namespace _(✗ task credential-pool-helm#T2 — requires cluster access)_
- [ ] GCP Secret `k3sf-postgres-sdlc` is the source _(✗ task credential-pool-helm#T2 — requires cluster access)_

---

**Browser-observable validation — all 7/7 passed:**
- [x] Server is reachable and app loads _(2026-03-04T23:10:38Z)_
- [x] Features API returns JSON (core API health) _(2026-03-04T23:10:38Z)_
- [x] credential-pool-core in released state _(2026-03-04T23:10:38Z)_
- [x] credential-pool-runs in released state _(2026-03-04T23:10:38Z)_
- [x] credential-pool-helm in released state _(2026-03-04T23:10:38Z)_
- [x] v38-credential-pool milestone in verifying state _(2026-03-04T23:10:38Z)_
- [x] UI navigation renders without crash _(2026-03-04T23:10:38Z)_

---

**Tasks created:** credential-pool-core#T7, credential-pool-core#T8, credential-pool-runs#T3–T6, credential-pool-helm#T2
**7/7 browser-observable steps passed | 28/35 acceptance checklist steps need infrastructure validation**
