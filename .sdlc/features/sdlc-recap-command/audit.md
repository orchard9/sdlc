# Security Audit: sdlc-recap-command

## Scope

This feature adds a `/sdlc-recap` slash command template installed via `sdlc init` / `sdlc update`. No new Rust code paths, no new server endpoints, no network communication, no authentication surface.

## Surface Analysis

### 1. Command file installation (write to `~/.claude/commands/`, etc.)

**Risk:** Low. The install mechanism (`write_user_command_scaffold`) uses `io::atomic_write` — the same atomic write path used by all other commands. No new write patterns introduced. Files are written to user home directories only, not to privileged system locations.

**Assessment:** No finding.

### 2. Command content — shell commands executed by the agent

The Claude command instructs the agent to run:
- `sdlc status --json` — read-only, no network
- `git log --oneline -20` — read-only
- `git diff --stat HEAD~5` — read-only
- `sdlc milestone info <slug> --json` — read-only
- `sdlc feature show <slug>` — read-only
- `sdlc task list <slug>` — read-only
- `sdlc ponder create "<question>" --brief "<context>"` — writes to `.sdlc/`, no network
- `sdlc task add <feature-slug> "<description>"` — writes to `.sdlc/`, no network
- `git add -A && git commit -m "session: <summary>"` — local git operation, no network

**Risk:** Low. All commands operate locally. No network calls. No external services invoked. No credentials accessed. `git add -A` is broad but this is standard practice documented across other sdlc commands (`sdlc-milestone-uat` uses the same pattern).

**Assessment:** No finding.

### 3. Argument injection via `$ARGUMENTS`

The command accepts `[feature-slug | milestone-slug]` as an optional argument passed to `sdlc milestone info <slug>` and `sdlc feature show <slug>`. These are `sdlc` CLI commands — the CLI validates slugs against the filesystem (slugs must be existing directories). A malformed slug would produce an error, not a code execution path.

**Risk:** Negligible. Slug arguments are not interpolated into shell via eval or similar patterns.

**Assessment:** No finding.

### 4. Template content injection

The command template is a static Rust `&str` constant. It cannot be modified at runtime. No user input reaches the template content.

**Assessment:** No finding.

### 5. Information disclosure

The recap synthesizes: git log, git diff stats, sdlc feature phases, milestone info. All of this is already visible to the user in their local repo. No sensitive data (secrets, credentials, tokens) is read or included in the recap output.

**Assessment:** No finding.

## Verdict

No security findings. This feature has no meaningful security surface — it is a static command template installed to user home directories using the same atomic write mechanism as all other commands. The commands it instructs agents to run are read-only or write only to local `.sdlc/` state and local git history.

**Audit result: PASS — no issues found.**
