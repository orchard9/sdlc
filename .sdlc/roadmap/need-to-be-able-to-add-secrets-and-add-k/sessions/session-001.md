---
session: 1
timestamp: 2026-03-03T03:30:00Z
orientation:
  current: "Gaps identified: UI is read-only for secrets (envs); can create new envs server-side safely; updating existing envs hits a security wall. Add-Key for recipients already works."
  next: "Scope a feature that adds (1) 'Add Environment' modal for new envs and (2) 'Set Secret' inline form, with a clear security boundary: server can create-new (encrypt-only), CLI required for update-existing (decrypt needed)."
  commit: "When the security boundary is agreed upon and the two UI interactions (new env + add to existing) are clearly defined with their constraints."
---

**jordan · Owner**
need to be able to add secrets and add keys to the "/secrets" interface

---

## Context audit

Before opening the discussion, I mapped the current state:

**What already works in the UI**
- `GET /api/secrets/keys` → list authorized public keys (recipients)
- `POST /api/secrets/keys` → add a recipient (the "Add Key" button + modal, fully functional)
- `DELETE /api/secrets/keys/:name` → remove a recipient
- `GET /api/secrets/envs` → list encrypted env files (metadata only)
- `DELETE /api/secrets/envs/:name` → delete an env

**What is missing**
- No way to *create* a new env file from the UI
- No way to *set* key-value pairs in an existing env from the UI
- The Environments section is effectively read-only (plus delete)

**Security architecture in play**
`secrets.rs` enforces a clear boundary: the server calls `age --encrypt` (safe, needs only public keys) but never calls `age --decrypt` (requires private key). The design intent is explicit: "values never leave your machine." This is a deliberate, good constraint.

The core `set_env_pairs` function signature exposes the tension:
```rust
pub fn set_env_pairs(root, env_name, pairs, identity: Option<&Path>) -> Result<()>
```
- `identity = None` is valid **only for brand-new env files** (no existing ciphertext to merge)
- `identity = Some(path)` is required to *update* an existing env (decrypt → merge → re-encrypt)

This means the server CAN create a new env file without ever seeing a private key — it only encrypts to the current recipients. But it CANNOT update an existing env without the user's private key.

---

## Session

**Facilitator**
Jordan, let's pull in Ben Hartley (developer UX) and Dan Reeves (systems minimalist) to pressure-test this. The request is clear — add and set secrets from the UI. But the shape of the solution depends heavily on where we draw the security line.

**Ben Hartley · Developer UX**
The UX gap is obvious when you sit with the page. You see a list of encrypted env files, their key names, when they were last updated. But there's no "+" button on the Environments section — only a delete button. The mental model breaks. A developer landing here for the first time thinks: "How do I add a secret?" The answer is: go to the CLI. That's fine for power users, but jarring for the web-first workflow.

The Add Key modal exists and it's good. So the UI already knows how to do a "create via form" interaction. We just need to extend that pattern to environments.

?  Open: Is the request about *adding new env files*, or about *adding key-value pairs to existing envs*, or both?

**Facilitator**
Looking at the seed carefully: "add secrets and add keys." I think "add secrets" means "add KEY=VALUE pairs to an env file" — the actual secret values. "Add keys" might already be solved (the Add Key recipient button). Or it might mean "add key-value pairs" (the same thing phrased differently). Let me assume both — we need to support creating environments and setting secrets.

**Dan Reeves · Systems Minimalist**
Before we design anything, let's be honest about the constraint. The server can do exactly one thing with secret values: encrypt them. It cannot decrypt. This isn't a limitation to work around — it's the security model. Any UI we build must honor it.

That gives us two natural affordances:

1. **Create a new env** — form with (env name, list of KEY=VALUE pairs). Server encrypts with current recipients. No private key needed. This is completely safe and server-side feasible.

2. **Add to an existing env** — here the server would need to: decrypt existing content (requires private key), merge in new pairs, re-encrypt. We can't do step 1 server-side.

⚑  Decided: Creating a brand-new env file from the UI is server-safe (encrypt-only). The API endpoint `POST /api/secrets/envs` can accept `{ env: "production", pairs: [{ key: "API_KEY", value: "..." }] }` and call `write_env()` directly.

**Ben Hartley · Developer UX**
I'd go further: for *updating* existing envs, the UI shouldn't pretend it can do what it can't. Show a "Manage in CLI" hint with a copy button for the exact command: `sdlc secrets env set production API_KEY=new_value`. That's honest. Developers understand the constraint when it's explained.

But here's the thing — most of the time when someone wants to "add a secret," they're adding it to a new environment they're bootstrapping, or they're adding a new key to an existing environment. The "edit existing secret value" workflow is actually rare for agents — agents typically set secrets once and the CLI handles rotation.

?  Open: Do we even need "add to existing" from the UI, or just "create new" + "CLI hint for updates"?

**Dan Reeves · Systems Minimalist**
I'd argue: solve "create new" cleanly first. Make the empty state actionable. The "add to existing" is a harder problem (requires client-side age encryption or a private-key-aware endpoint) and the payoff is lower — that's a rotation/update workflow, not an onboarding workflow.

The empty state currently shows:
```
No encrypted env files yet.
sdlc secrets env set production API_KEY=value
```
That command hint is good for CLI users. But we could make it a button that opens an "Add Environment" modal.

⚑  Decided: Phase 1 scope = "Add Environment" modal (name + key-value pairs, new env only). Existing envs get a "Set secret via CLI" copy-command affordance, not an edit form. No client-side encryption complexity needed.

**Ben Hartley · Developer UX**
The modal design for "Add Environment":
- Env name field (text, e.g. "production", "staging")
- Expandable key-value list (add row button, remove row)
- Submit → calls `POST /api/secrets/envs` → server calls `write_env()`
- Requires at least one authorized key to be configured (otherwise age has no recipients — show an inline error pointing to the Keys section)

For the existing env cards, add a small "+" button that's labeled "Add secret" which opens a drawer or inline form — but immediately shows: "To set a secret in an existing env, run: `sdlc secrets env set {env} KEY=value`" with a copy button. This is the honest UX for the constraint.

**Facilitator**
What about the server endpoint? The Rust layer needs:
- `POST /api/secrets/envs` → body `{ env: String, pairs: Vec<{key: String, value: String}> }` → calls `write_env()` with current keys
- Requirement: at least one authorized key must exist, env name must not already exist (or we document clearly that it overwrites)

?  Open: Should `POST /api/secrets/envs` fail if the env already exists, or should it always overwrite (write from scratch)?

**Dan Reeves · Systems Minimalist**
Fail if exists. The UI modal is for *creating* a new env. Overwriting an existing env from a web form would be dangerous — you'd lose secrets that were set via CLI. Server should return 409 Conflict if the env already exists. This forces users to the CLI for updates, which is correct.

⚑  Decided: `POST /api/secrets/envs` → 409 if env already exists. It's a create-only endpoint, not an upsert.

**Ben Hartley · Developer UX**
One more thing on the "add keys" part of the seed. The Add Key modal already exists and works. But let me check if the UX is discoverable enough. The button is a small text link: `+ Add Key`. It works, but it's subtle.

In the context of this feature, I think the "add keys" ask is already solved — Jordan may not have realized the button exists, or might have been referring to key-value pairs for secrets. Either way, the Add Key recipient modal is complete.

?  Open: Should we polish the Add Key button to be more prominent (e.g., a proper button with a border, not just a text link)?

**Facilitator**
That's polish scope. Let's not fold it into this feature unless Jordan specifically asks. Flag it as a follow-on.

**Dan Reeves · Systems Minimalist**
Summary of what this feature IS:
1. `POST /api/secrets/envs` endpoint — create new encrypted env, returns 409 if exists, requires keys configured
2. "Add Environment" modal on SecretsPage — env name + key-value list → calls that endpoint
3. "Add secret via CLI" hint on existing env cards — copy-button for `sdlc secrets env set`

What this feature IS NOT:
- Client-side age encryption (future roadmap if needed)
- Edit-in-place for existing secret values
- Private key handling in the server

⚑  Decided: Feature scope is locked at the three items above. This is a clean, safe feature that doesn't touch the security boundary.

---

## Commit signal

The shape is clear. The security model is respected. The API shape is defined. The UI interactions are specified. This is ready to crystallize into a feature.
