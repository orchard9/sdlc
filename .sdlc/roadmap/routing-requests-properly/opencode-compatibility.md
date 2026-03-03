## OpenCode Protocol Compatibility — The Unknown

**What we know:**
- `claude-agent` crate spawns `claude --output-format stream-json --input-format stream-json`
- This is the bidirectional JSON streaming protocol from `@anthropic-ai/claude-agent-sdk`
- `path_to_executable` in `QueryOptions` already supports overriding the binary name

**What we don't know:**
- Does OpenCode CLI (`opencode`) support the same `--output-format stream-json --input-format stream-json` flags?
- Does it accept the same `--model`, `--max-turns`, `--permission-mode`, `--mcp-config` flags?
- Does it produce the same JSONL message schema (SystemInit, Assistant, User/ToolResult, Result)?
- Does it support MCP server injection via `--mcp-config` in the same format?

**Likely situation (based on OpenCode architecture):**
OpenCode (`opencode` by SST) is a TUI, not a streaming subprocess protocol. It doesn't expose a `--output-format stream-json` interface. It's designed for interactive terminal use, not subprocess streaming. It would require a **separate process driver** in the `claude-agent` crate.

**?  Open: Is OpenCode actually subprocess-compatible with the claude CLI stream protocol?**
This needs a spike before we commit to 'opencode as provider'. If it's not compatible, Phase 1 should only cover Claude Code model routing, and OpenCode support becomes a Phase 2 new driver.

**Alternative path if OpenCode is not compatible:**
Route Gemini via the Gemini CLI (`gemini`) which does have a `--output-format stream-json` mode similar to Claude Code. That may be a cleaner fit as a provider option.