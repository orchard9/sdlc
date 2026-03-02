# Security Audit: knowledge-slash-command

## Scope

Three files modified:
1. `crates/sdlc-cli/src/cmd/init/commands/sdlc_knowledge.rs` — new static string constants (command templates)
2. `crates/sdlc-cli/src/cmd/init/commands/mod.rs` — module registration and `ALL_COMMANDS` slice entry
3. `crates/sdlc-cli/src/cmd/init/templates.rs` — additions to `GUIDANCE_MD_CONTENT` string constant

## Security Surface Analysis

### Static string constants (no runtime risk)

All three additions are Rust `const`/`static &str` literals compiled into the binary. They contain no runtime logic, no code execution, and no dynamic content. The strings are written to disk by `sdlc init` / `sdlc update` as slash command files in user home directories (`~/.claude/commands/`, `~/.gemini/commands/`, etc.).

**No injection risk.** The strings are written verbatim as plain text files — there is no template engine, no string interpolation, and no execution of the content at write time. The content is only executed later by the AI CLI that reads the installed file, which is fully outside this binary's trust boundary.

### Content safety

The command templates instruct agents to run `sdlc knowledge *` subcommands, `Grep`, `Glob`, `Read`, `WebSearch`, and `WebFetch`. These are all standard, bounded operations:

- `sdlc knowledge *` — reads/writes `.sdlc/knowledge/` only; scoped to project directory
- `Grep`, `Glob`, `Read` — read-only filesystem access
- `WebSearch`, `WebFetch` — outbound only, agent-controlled, no credentials exposed

No shell escalation, no `eval`, no `rm`, no sudo, no credential access in any command.

### `allowed-tools` frontmatter

The Claude Code command declares:
```
allowed-tools: Bash, Read, Write, Glob, Grep, WebSearch, WebFetch
```

`Bash` is needed for `sdlc knowledge *` subprocess calls. `Write` is not used by the current command content but is a standard inclusion for commands that may need to write research session logs. This is consistent with the pattern used by `sdlc-guideline` and `sdlc-ponder` commands. No escalation path through this set.

### Module registration

Adding `mod sdlc_knowledge;` and `&sdlc_knowledge::SDLC_KNOWLEDGE` to `ALL_COMMANDS` simply makes the constant visible and included in the install loop. No new code paths, no new file-system targets beyond the standard install destinations.

## Findings

No security findings. The change is:
- Purely additive static content
- No runtime logic introduced
- No new file-system write targets
- No new network calls from the Rust binary
- No credential handling
- Content follows established patterns with no escalation beyond the standard `sdlc knowledge` CLI scope

## Verdict

Approved. No security issues.
