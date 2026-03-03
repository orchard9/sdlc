# Design: Product Summary format contract in /sdlc-ponder skill

## Overview

This is a documentation-only change to three skill instruction constants in `crates/sdlc-cli/src/cmd/init/commands/sdlc_ponder.rs`. No Rust logic, no data schema changes, no API changes. The change surface is entirely within the `const &str` strings that `sdlc init` / `sdlc update` installs to user directories.

## Affected File

**`crates/sdlc-cli/src/cmd/init/commands/sdlc_ponder.rs`**

Three constants are updated:

| Constant | Platform | Change |
|---|---|---|
| `SDLC_PONDER_COMMAND` | Claude Code `~/.claude/commands/sdlc-ponder.md` | Add full Product Summary schema + rules to Session Log Protocol section |
| `SDLC_PONDER_PLAYBOOK` | Gemini/OpenCode | Add concise Product Summary requirement to step 6 |
| `SDLC_PONDER_SKILL` | Agent Skills `~/.agents/skills/sdlc-ponder/SKILL.md` | Add one-sentence Product Summary requirement to step 6 |

## Change Detail: SDLC_PONDER_COMMAND

The Session Log Protocol section already contains:
- Why sessions must be logged
- The session file format (YAML frontmatter + body)
- Inline markers (`⚑ Decided:`, `? Open:`, etc.)
- The two-step logging procedure

**Addition:** After the session file format block, insert a `### Product Summary section` subsection that defines:

```
### Product Summary section

Every session log MUST end with a `## Product Summary` block. This section is extracted
by the UI for session card preview — the H3 subsection labels are **stable and locked**.

Format:

    ## Product Summary

    ### What we explored
    [1-2 sentences in plain English — what problem or idea this session covered]

    ### Key shifts
    [What changed vs. before this session — decisions made, assumptions revisited,
    prior beliefs updated. Reference ⚑ markers from the session body.]

    ### Implications
    [What this means for the feature/milestone in product language — no tech jargon.
    Connect findings to backlog/roadmap consequences.]

    ### Still open
    [The 1-2 questions that remain unresolved, phrased as decisions to be made —
    not technical investigations. Reference ? markers from the session body.]

Rules:
- H3 labels (`What we explored`, `Key shifts`, `Implications`, `Still open`) are locked —
  do not rename them
- Implications must use product language, not technical language
- Still open items are phrased as decisions, not technical tasks
```

This addition goes between the "Inline markers" subsection and "The only correct logging procedure" subsection in the existing Session Log Protocol.

## Change Detail: SDLC_PONDER_PLAYBOOK

Step 6 currently reads:
> Write and log the session file: compose a Markdown session with YAML frontmatter (session, timestamp, orientation). `sdlc ponder session log <slug> --file /tmp/session-<N>.md`

**Addition:** Append to step 6: "End the log with a `## Product Summary` section containing four fixed H3s: What we explored, Key shifts, Implications, Still open."

## Change Detail: SDLC_PONDER_SKILL

Step 6 currently reads:
> Before ending: compose session Markdown with YAML frontmatter (session, timestamp, orientation) and log it: `sdlc ponder session log <slug> --file /tmp/session.md`.

**Addition:** Append to step 6: "End the session file with `## Product Summary` (4 subsections: What we explored, Key shifts, Implications, Still open — labels locked)."

## Ending the Session Section Update

The `## Ending the session` section in `SDLC_PONDER_COMMAND` currently has three steps:
1. Compose the session document
2. Log it
3. Summarize

Step 1 should reference the Product Summary requirement. Add to the existing step 1 instruction: "Include a `## Product Summary` section at the end of the session body using the schema defined above."

## Propagation

After the constants are updated, `sdlc init` and `sdlc update` automatically write the new content to user directories. No migration is needed — the next run of either command installs the updated skill text. Existing session logs already written are not affected.

## Non-Changes

- No Rust struct changes
- No YAML schema changes
- No new CLI subcommands
- No REST API changes
- No frontend changes (UI extraction is a future milestone)
- No changes to `ponder.rs`, `workspace.rs`, or any data layer
