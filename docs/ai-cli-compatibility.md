# AI Coding CLI Compatibility Guide

## Claude Code · OpenCode · Gemini CLI

A comprehensive cross-reference of extensibility features across the three major AI coding CLI tools. Use this guide to write configurations that work across tools, migrate between them, or understand what's available in each.

> **Last verified:** February 2026
> **Versions:** Claude Code v1.0+, OpenCode (latest), Gemini CLI v0.26.0+

---

## At a Glance

| Feature | Claude Code | OpenCode | Gemini CLI |
|:---|:---:|:---:|:---:|
| Always-on context file | `CLAUDE.md` | `AGENTS.md` / `CLAUDE.md` | `GEMINI.md` / `AGENT.md` |
| Custom slash commands | Markdown | Markdown / JSON | TOML |
| Agent Skills (SKILL.md) | Yes | Yes | Yes |
| Custom subagents | Yes | Yes | Yes |
| Hooks (event automation) | JSON + shell/prompt/agent | JS/TS plugins | JSON + shell |
| Plugins / Extensions | Plugins + Marketplaces | npm plugins | Extensions |
| MCP servers | Yes | Yes | Yes |
| LSP integration | via plugins | built-in | No |
| Model flexibility | Claude only | 75+ providers | Gemini only (+ API key) |

---

## 1. Always-On Context File

The foundational configuration file that the AI reads at the start of every session.

### File Names and Locations

| Scope | Claude Code | OpenCode | Gemini CLI |
|:---|:---|:---|:---|
| **Project** | `CLAUDE.md` (project root) | `AGENTS.md` (preferred) or `CLAUDE.md` | `GEMINI.md` or `AGENT.md` |
| **User/Global** | `~/.claude/CLAUDE.md` | `~/.config/opencode/AGENTS.md` or `~/.claude/CLAUDE.md` | `~/.gemini/GEMINI.md` |
| **Subdirectory** | Auto-loads from subdirs as you work in them | Not auto-loaded from subdirs | Supports `@./path/to/file.md` imports |

### Behavior Differences

| Behavior | Claude Code | OpenCode | Gemini CLI |
|:---|:---|:---|:---|
| **Loading** | Additive — all levels contribute | First match wins (AGENTS.md > CLAUDE.md) | Hierarchical, walks up to root |
| **Conflict resolution** | Claude uses judgment; specific overrides general | First matching file at each level wins | Hierarchical merge |
| **Custom instruction files** | Only CLAUDE.md files | `"instructions"` field in `opencode.json` supports globs and remote URLs | `@./path/to/file.md` imports inside GEMINI.md |
| **Cross-tool compat** | — | Reads `~/.claude/CLAUDE.md` natively (disable with `OPENCODE_DISABLE_CLAUDE_CODE=1`) | Reads `AGENT.md` as generic alias |

### Cross-Compatible Pattern

To support all three tools from a single repo, create both files with the same content:

```
project-root/
├── CLAUDE.md          # Claude Code reads this
├── AGENTS.md          # OpenCode reads this (takes precedence over CLAUDE.md)
└── GEMINI.md          # Gemini CLI reads this
```

Or symlink them:

```bash
# Write your instructions in AGENTS.md, then symlink
ln -s AGENTS.md CLAUDE.md
ln -s AGENTS.md GEMINI.md
```

---

## 2. Custom Slash Commands

Saved prompts invoked by typing `/command-name`.

### Format Comparison

| Aspect | Claude Code | OpenCode | Gemini CLI |
|:---|:---|:---|:---|
| **File format** | Markdown (`.md`) | Markdown (`.md`) or JSON in config | TOML (`.toml`) |
| **Project location** | `.claude/commands/` | `.opencode/commands/` | `.gemini/commands/` |
| **User location** | `~/.claude/commands/` | `~/.config/opencode/commands/` | `~/.gemini/commands/` |
| **Naming** | Filename = command (e.g., `review.md` -> `/review`) | Filename = command, scoped by location (e.g., `user:review`) | Filename = command; directories create namespaces (e.g., `git/commit.toml` -> `/git:commit`) |

### Arguments and Dynamic Content

| Feature | Claude Code | OpenCode | Gemini CLI |
|:---|:---|:---|:---|
| **Argument placeholder** | `$ARGUMENTS` | `$ARGUMENTS` + positional params (`$1`, `$2`) | `{{args}}` |
| **Shell output injection** | `` !`command` `` (in skills only) | `!command` (inline) | `!{command}` |
| **File content injection** | Not built-in | `@filename` (inline) | `@{path/to/file}` |
| **Force subagent** | Via `agent:` frontmatter (in skills) | `subtask: true` in config | Not directly supported |
| **Model override** | Not per-command | `model:` field in config | Not per-command |
| **Reload command** | Restart required | Restart required | `/commands reload` (hot reload) |

### Example: Code Review Command

**Claude Code** — `.claude/commands/review.md`:
```markdown
Review the current staged changes for:
1. Code quality issues
2. Security vulnerabilities
3. Missing test coverage
Provide actionable feedback with file/line references.
```

**OpenCode** — `.opencode/commands/review.md`:
```markdown
---
description: Review staged changes for quality and security
---
Review the current staged changes for:
1. Code quality issues
2. Security vulnerabilities
3. Missing test coverage
Provide actionable feedback with file/line references.
```

**Gemini CLI** — `.gemini/commands/review.toml`:
```toml
description = "Review staged changes for quality and security"
prompt = """
Review the current staged changes for:
1. Code quality issues
2. Security vulnerabilities
3. Missing test coverage
Provide actionable feedback with file/line references.
"""
```

---

## 3. Agent Skills

Model-invoked capabilities that the AI autonomously decides to use based on the skill's description. All three tools follow the **Agent Skills open standard** using `SKILL.md` files.

### Locations

| Scope | Claude Code | OpenCode | Gemini CLI |
|:---|:---|:---|:---|
| **Project** | `.claude/skills/<name>/` | `.opencode/skills/<name>/` **or** `.claude/skills/<name>/` **or** `.agents/skills/<name>/` | `.gemini/skills/<name>/` **or** `.agents/skills/<name>/` |
| **User** | `~/.claude/skills/<name>/` | `~/.config/opencode/skills/<name>/` **or** `~/.claude/skills/<name>/` **or** `~/.agents/skills/<name>/` | `~/.gemini/skills/<name>/` **or** `~/.agents/skills/<name>/` |
| **Plugin/Extension** | Bundled in plugins | Bundled in plugins | Bundled in extensions |

### SKILL.md Format (Cross-Compatible)

The core `SKILL.md` format is identical across all three tools:

```markdown
---
name: code-reviewer
description: Review code for best practices and potential issues. Use when reviewing code, checking PRs, or analyzing code quality.
---

# Code Reviewer

## Instructions
1. Read the target files
2. Search for anti-patterns
3. Provide detailed feedback

## Checklist
- Error handling completeness
- Security concerns
- Test coverage gaps
```

### Feature Differences

| Feature | Claude Code | OpenCode | Gemini CLI |
|:---|:---|:---|:---|
| **Discovery** | Name + description injected; model decides | On-demand via native skill tool | Name + description injected into system prompt |
| **Activation** | Automatic (silent) | Automatic | Explicit consent prompt shown to user |
| **Tool restrictions** | `allowed-tools:` frontmatter | Not supported | Not supported |
| **Subagent execution** | `context: fork` + `agent:` field | Not documented | Not documented |
| **Shell preprocessing** | `` !`command` `` syntax | Not documented | Not documented |
| **Disable model invocation** | `disable-model-invocation: true` | Not documented | Via `/skills disable <name>` |

### Cross-Compatible Skill

Use the `.agents/skills/` directory — both OpenCode and Gemini CLI support this as a generic alias:

```
.agents/skills/my-skill/
├── SKILL.md
├── scripts/
│   └── helper.py
└── templates/
    └── template.txt
```

For Claude Code compatibility, also create a symlink or duplicate in `.claude/skills/`. OpenCode reads `.claude/skills/` natively.

---

## 4. Custom Subagents

Isolated specialist agents with their own context window, system prompt, and tool access.

### File Locations

| Scope | Claude Code | OpenCode | Gemini CLI |
|:---|:---|:---|:---|
| **Project** | `.claude/agents/*.md` | `.opencode/agents/*.md` | `.gemini/agents/*.md` |
| **User** | `~/.claude/agents/*.md` | `~/.config/opencode/agents/*.md` | `~/.gemini/agents/*.md` |
| **Interactive creation** | `/agents` command | `/agents` command | Manual file creation |

### File Format Comparison

All three use Markdown with YAML frontmatter, but the fields differ slightly:

**Claude Code**:
```markdown
---
name: security-reviewer
description: Analyze code for security vulnerabilities.
tools: Read, Grep, Glob
model: sonnet
---

You are a senior security engineer...
```

**OpenCode**:
```markdown
---
description: Analyze code for security vulnerabilities.
model: anthropic/claude-sonnet-4-5
mode: subagent
tools:
  - read
  - grep
  - glob
permissions:
  bash:
    deny: ["*"]
---

You are a senior security engineer...
```

**Gemini CLI**:
```markdown
---
name: security-reviewer
description: Analyze code for security vulnerabilities.
---

You are a senior security engineer...
```

### Feature Comparison

| Feature | Claude Code | OpenCode | Gemini CLI |
|:---|:---|:---|:---|
| **Built-in agents** | Explore, Plan, general-purpose | Build, Plan, Research, Explore | Codebase Investigator, Docs Lookup, Router |
| **Auto-delegation** | Based on description | Based on description | Based on description |
| **Model override** | `model: sonnet/opus/haiku` | `model: provider/model-id` (any provider) | Per-agent in settings.json |
| **Tool scoping** | `tools:` field (comma-separated) | `tools:` field (list) + `permissions:` block | Via system prompt / settings |
| **Agent modes** | All subagents | `mode: primary`, `subagent`, or `all` | Primary and subagent distinction |
| **Remote agents** | No | No | Agent-to-Agent (A2A) protocol |
| **Built-in browser agent** | No (separate product) | No | Yes |

---

## 5. Hooks (Event-Driven Automation)

Deterministic scripts that run at specific points in the agent lifecycle.

### Architecture Comparison

| Aspect | Claude Code | OpenCode | Gemini CLI |
|:---|:---|:---|:---|
| **Config format** | JSON in `settings.json` | JS/TS plugin modules | JSON in `settings.json` |
| **Hook language** | Shell scripts (+ LLM prompt + agent types) | JavaScript / TypeScript | Shell scripts (any language) |
| **Location** | `~/.claude/settings.json` or `.claude/settings.json` | `.opencode/plugins/*.ts` or `~/.config/opencode/plugins/*.ts` | `.gemini/settings.json` or `~/.gemini/settings.json` |

### Hook Events Mapping

| Purpose | Claude Code | OpenCode | Gemini CLI |
|:---|:---|:---|:---|
| **Before tool runs** | `PreToolUse` | `tool.execute.before` | `BeforeTool` |
| **After tool runs** | `PostToolUse` | `tool.execute.after` | `AfterTool` |
| **Session starts** | `SessionStart` | `session.created` | `SessionStart` |
| **Session ends** | `Stop` | `session.idle` / `session.deleted` | `SessionEnd` |
| **User sends message** | `UserPromptSubmit` | Not documented | `UserPrompt` |
| **Subagent completes** | `SubagentStop` | N/A | N/A |
| **Before model call** | N/A | N/A | `BeforeModel` |
| **After model response** | N/A | N/A | `AfterModel` |
| **Total events** | **8** | **25+** | **11+** |

### Hook Handler Types

| Type | Claude Code | OpenCode | Gemini CLI |
|:---|:---|:---|:---|
| **Shell command** | `type: "command"` | Use helper in plugin | `command` field |
| **LLM prompt** | `type: "prompt"` | No | No |
| **Agent/subagent** | `type: "agent"` | No | No |
| **JS/TS function** | No | Yes (native) | No |

### Blocking Actions

| Tool | How to block |
|:---|:---|
| **Claude Code** | Exit code 2; stderr becomes feedback to Claude |
| **OpenCode** | `throw new Error("reason")` in plugin |
| **Gemini CLI** | Exit code 2; stderr becomes replacement content; or `"decision": "deny"` in JSON output |

---

## 6. Plugins / Extensions

Shareable bundles that package commands, agents, skills, hooks, and integrations.

### Comparison

| Aspect | Claude Code | OpenCode | Gemini CLI |
|:---|:---|:---|:---|
| **Term** | Plugins + Marketplaces | Plugins | Extensions |
| **Bundle format** | Directory with `.claude-plugin/plugin.json` | JS/TS modules or npm packages | Directory with `gemini-extension.json` |
| **Can contain** | Commands, agents, skills, hooks, MCP servers, LSP servers | Hooks, custom tools, event handlers | Commands, agents, skills, hooks, MCP servers, themes |
| **Distribution** | Git repos (marketplaces) | npm registry or local files | GitHub repos or local paths |
| **Install command** | `/plugin marketplace add <repo>` then `/plugin install <name>` | Add to `"plugin"` array in `opencode.json` | `gemini extensions install <url-or-path>` |
| **Namespacing** | `plugin-name:command` | `plugin-name:command` | `extension-name:command` |

---

## 7. MCP Server Configuration

All three tools support Model Context Protocol for external tool integration.

| Aspect | Claude Code | OpenCode | Gemini CLI |
|:---|:---|:---|:---|
| **Config file** | `.claude/.mcp.json` or `~/.claude/.mcp.json` | `opencode.json` under `"mcpServers"` | `.gemini/settings.json` under `"mcpServers"` |
| **Transport** | stdio, SSE | stdio, SSE | stdio, SSE |
| **Tool conflicts** | Manual naming | Manual naming | Auto-prefixed with server alias |
| **Via plugins/extensions** | `.mcp.json` in plugin | Via config | `mcp.json` in extension |

---

## 8. Cross-Tool Compatibility Strategies

### Universal Directory Structure

For a project that supports all three tools:

```
project-root/
├── CLAUDE.md                      # Claude Code context
├── AGENTS.md                      # OpenCode context (symlink to CLAUDE.md if identical)
├── GEMINI.md                      # Gemini CLI context (symlink to CLAUDE.md if identical)
│
├── .agents/                       # Generic alias (OpenCode + Gemini CLI)
│   └── skills/
│       └── my-skill/
│           └── SKILL.md
│
├── .claude/
│   ├── skills/                    # Claude Code + OpenCode reads this
│   ├── agents/
│   ├── commands/
│   └── settings.json
│
├── .opencode/
│   ├── agents/
│   ├── commands/
│   ├── plugins/
│   └── opencode.json
│
├── .gemini/
│   ├── agents/
│   ├── commands/                  # TOML format
│   ├── skills/
│   └── settings.json
```

### What's Portable Across All Three

| Component | Portability | Notes |
|:---|:---|:---|
| **Context files** | Same content, different filenames | Symlink `CLAUDE.md`, `AGENTS.md`, `GEMINI.md` |
| **Skills (SKILL.md)** | Fully portable | Same format; use `.agents/skills/` for broadest reach |
| **Subagent definitions** | Mostly portable | Core frontmatter + markdown body shared; tool/model names differ |
| **Commands** | Not portable | Markdown vs Markdown vs TOML; different argument syntax |
| **Hooks** | Not portable | Different architectures and event names |
| **Plugins/Extensions** | Not portable | Different packaging and manifest formats |
| **MCP configs** | Mostly portable | Same protocol; config location and JSON structure differ |

---

## 9. Unique Strengths

### Claude Code
- **3 hook handler types**: Shell commands, LLM prompts, and agent-based hooks
- **Skill frontmatter power**: `allowed-tools`, `context: fork`, `agent:`, shell preprocessing
- **Native agent teams**: Multi-session coordination built-in
- **Background agents**: Send to background with Ctrl+B natively

### OpenCode
- **Model agnostic**: 75+ providers
- **Most hook events**: 25+ lifecycle events via JS/TS plugin SDK
- **Best cross-tool compat**: Natively reads `.claude/` directories and skills
- **Programmable plugins**: Full JS/TS SDK with typed APIs

### Gemini CLI
- **Best free tier**: 60 req/min, 1,000 req/day with a personal Google account
- **User consent for skills**: Explicit confirmation prompt before skill activation
- **Remote subagents**: Agent-to-Agent (A2A) protocol
- **Built-in browser agent**: Web interaction with accessibility tree
- **Strongest partner ecosystem**: Dynatrace, Elastic, Figma, Shopify, Snyk, Stripe

---

## 10. Shared Standards

| Standard | Status |
|:---|:---|
| **SKILL.md format** | Adopted by all three (Agent Skills open standard) |
| **`.agents/` directory alias** | OpenCode + Gemini CLI (Claude Code uses `.claude/` only) |
| **MCP (Model Context Protocol)** | Adopted by all three |
| **Markdown agent definitions** | All three use `.md` with YAML frontmatter for subagents |
| **JSON hooks over stdin/stdout** | Claude Code + Gemini CLI (OpenCode uses JS/TS) |
| **Hierarchical settings** | All three: user -> project -> managed/system |
