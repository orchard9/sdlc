# Security Audit: _shared/sdlc.ts — typed state access for tools

## Scope

This audit covers the security surface of `_shared/sdlc.ts` and the associated `init.rs` skill template changes.

---

## Threat Model

The module runs in the context of a user-controlled SDLC tool invoked via `sdlc tool run`. It has the same trust level as any other tool in `.sdlc/tools/`. The module:

1. Reads files from `$SDLC_ROOT/.sdlc/` — a user-controlled directory
2. Writes one file: `$SDLC_ROOT/.sdlc/beat.yaml`
3. Spawns two `sdlc` subprocesses: `sdlc ponder create` and `sdlc ponder session log`
4. Writes one temp file: `/tmp/ponder-session-<slug>-<ts>.md`

---

## Finding 1: Path traversal via `root` argument — ACCEPTED

**Risk:** `root` is supplied by the caller (ultimately from `SDLC_ROOT` or `process.cwd()`). A caller that controls `root` could potentially pass a path like `/etc` to read arbitrary system files via `readFeatures('/etc')`.

**Mitigating factors:**
- `SDLC_ROOT` is injected by the `sdlc` server process — it is always the project root, not user-supplied input.
- `process.cwd()` fallback resolves to the directory where the tool was launched, which is the project root in normal usage.
- All read operations use `readFileSync` on path-joined results — no path injection via user-controlled strings from outside the tool boundary.
- The module is a library for trusted tool authors, not an HTTP endpoint accepting untrusted input.

**Disposition:** Accept. The attack surface is identical to every other file read in the tool ecosystem (e.g., `quality-check/tool.ts` already reads config via `readFileSync`). Adding `root` validation would be security theater with no real benefit given the trust model.

---

## Finding 2: Temp file predictable path (`/tmp/ponder-session-<slug>-<ts>.ms`) — ACCEPTED

**Risk:** The temp file path includes a timestamp (`Date.now()`) but not a random suffix. A local attacker with access to the same system could potentially create a symlink at the predicted path before the write, redirecting the write to an attacker-controlled target.

**Mitigating factors:**
- This attack requires local execution access on the same machine — at which point the attacker already has access to `.sdlc/` directly.
- The temp file contains only the ponder session content (markdown text from a tool). This is non-sensitive data.
- The file is written and immediately consumed by the CLI subprocess and deleted. The window is milliseconds.
- This is the same pattern used in other SDLC session logging contexts (documented in MEMORY.md as the standard protocol).

**Disposition:** Accept. The risk is negligible given the trust model (local developer machine, no multi-tenant environment). Track improvement opportunity: add 4 random hex chars to the filename for defense in depth.

**Action taken:** Add task T6 to the backlog for filename entropy improvement.

---

## Finding 3: subprocess injection via `createPonder(title)` — ACCEPTED

**Risk:** `title` is passed as a command-line argument to `sdlc ponder create <title>`. If `title` contains shell metacharacters, could there be command injection?

**Mitigating factors:**
- `spawnSync` is used with an explicit argument array (`['ponder', 'create', title]`), NOT a shell string. The `shell: false` behavior of `spawnSync` with an array means the OS receives the argument as a literal string — no shell expansion occurs.
- No `exec` or template-literal shell command is used anywhere in the module.

**Disposition:** No issue. The implementation is correct and injection-safe.

---

## Finding 4: `writeBeat` overwrites existing content without backup — ACCEPTED

**Risk:** A buggy caller that passes malformed `BeatState` will silently overwrite valid history with corrupted data. The atomic rename prevents a partial write, but there is no backup of the previous state.

**Mitigating factors:**
- `beat.yaml` is committed to git — any bad write is recoverable via `git checkout .sdlc/beat.yaml`.
- The `BeatState` type contract (TypeScript) prevents structurally malformed data at the call site if the caller is type-checked.
- The module's contract is documented: callers should `readBeat()` → modify → `writeBeat()`.

**Disposition:** Accept. Git is the backup mechanism (as stated in CLAUDE.md: "Git is the undo button"). No runtime backup needed.

---

## Finding 5: No validation of `slug` in `appendPonderSession` — LOW

**Risk:** The `slug` parameter is interpolated into a temp file path (`/tmp/ponder-session-${slug}-<ts>.md`) and passed as a CLI argument. If `slug` contains `/`, `..`, or other path characters, the temp path could escape `/tmp/`.

**Mitigating factors:**
- SDLC slugs are validated by the Rust layer to match `^[a-z0-9][a-z0-9-]*[a-z0-9]$` — no `/` or `.` characters are valid in a slug.
- Callers typically obtain slugs from `createPonder()` (which returns what the CLI produced) or from `readFeatures().map(f => f.slug)` — both come from validated sources.
- The argument array form of `spawnSync` prevents shell injection regardless.

**Disposition:** Accept. The slug constraint eliminates the path traversal risk. No change needed, but document in JSDoc.

**Action taken:** The JSDoc for `appendPonderSession` notes the slug must be a valid SDLC slug (alphanumeric + hyphens).

---

## Summary Table

| Finding | Risk Level | Disposition | Action |
|---|---|---|---|
| Path traversal via `root` | Low | Accept — matches existing tool pattern | None |
| Predictable temp file path | Low | Accept — defense-in-depth improvement tracked | T6: add random suffix |
| subprocess injection via `title` | None | Not an issue — spawnSync array form used | None |
| `writeBeat` no backup | Low | Accept — git is the backup | None |
| Slug validation in `appendPonderSession` | Low | Accept — slug format constraint sufficient | Document in JSDoc |

**Audit result: APPROVED.** No blockers. Two improvement items tracked as T6.

---

## Rust/Init Changes

The `init.rs` skill template changes are text-only (no executable code). The documentation additions cannot create a security surface. **No security findings.**
