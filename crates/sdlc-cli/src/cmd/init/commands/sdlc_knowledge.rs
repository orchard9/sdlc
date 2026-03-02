use crate::cmd::init::registry::CommandDef;

const SDLC_KNOWLEDGE_COMMAND: &str = r#"---
description: Query and manage the project knowledge base — catalog overview, topic synthesis, init, research, and maintenance
argument-hint: [<topic> | init | research <topic> | maintain]
allowed-tools: Bash, Read, Write, Glob, Grep, WebSearch, WebFetch
---

# sdlc-knowledge

Access and manage the project knowledge base. Five modes depending on `$ARGUMENTS`:

| Invocation | Mode |
|---|---|
| (no argument) | Catalog overview — show entry counts by category and status |
| `<topic>` | Query mode — synthesize answer from knowledge base with citations |
| `init` | Seed the knowledge base from existing workspaces (ponders, investigations, guidelines) |
| `research <topic>` | Active research — find, synthesize, and index new knowledge for topic |
| `maintain` | Maintenance pass — check for stale entries, orphans, catalog gaps |

> **Before acting:** read `.sdlc/guidance.md` for engineering principles. <!-- sdlc:guidance -->

---

## Mode: no argument (catalog overview)

Run:

```bash
sdlc knowledge status
sdlc knowledge catalog show
sdlc knowledge list
```

Present:
- Total entry count and status breakdown (draft vs published)
- Catalog class list with entry counts per class
- Last maintained date
- Top 5 most recently updated entries

End with:

**Next:** `/sdlc-knowledge <topic>` to query, or `/sdlc-knowledge init` if the base is empty.

---

## Mode: `<topic>` (query)

Full-text search for the topic, then synthesize:

```bash
sdlc knowledge search "<topic>"
```

For each result (up to 5), read the full entry:

```bash
sdlc knowledge show <slug>
```

Synthesize an answer from the matching entries:
- Lead with a direct answer to the inferred question
- Cite each entry used: `[<title>] (<slug>)`
- Note gaps: if coverage is thin, say so explicitly
- Suggest related topics from the `related` fields of matching entries

If no results:
```bash
sdlc knowledge list
```
Check if broader terms match. If still nothing, suggest `/sdlc-knowledge research <topic>`.

End with:

**Next:** `/sdlc-knowledge research <topic>` to index new knowledge, or `/sdlc-ponder <slug>` to explore further.

---

## Mode: `init`

Seed the knowledge base from completed workspaces:

```bash
sdlc knowledge librarian init
```

Report the results:
- Investigations harvested (new vs updated)
- Ponders harvested (new vs updated)
- Guidelines linked
- Catalog classes created
- Cross-references added

If the librarian agent file was created, note its path.

End with:

**Next:** `/sdlc-knowledge` to see the catalog overview, or `/sdlc-knowledge <topic>` to query.

---

## Mode: `research <topic>`

Active research — find new knowledge and index it.

### Step 1: Check what exists

```bash
sdlc knowledge search "<topic>"
sdlc knowledge catalog show
```

### Step 2: Gather from codebase

Use Grep and Glob to find relevant source files, patterns, and conventions related to the topic. Use Read to examine the most relevant ones. Extract concrete facts, patterns, and decisions.

### Step 3: Gather from web (if needed)

Use WebSearch and WebFetch to find community knowledge, named patterns, RFC language, or external references relevant to the topic.

### Step 4: Index findings

For each distinct piece of knowledge found, add an entry:

```bash
sdlc knowledge add --title "<descriptive title>" --code <catalog-code> --content "<markdown content>"
```

Use the catalog from Step 1 to pick the right code. If no class fits, use `uncategorized` and note it.

For web sources, include the URL in the content:

```bash
sdlc knowledge add --title "<title>" --code <code> --from-url "<url>"
```

Or for a file:

```bash
sdlc knowledge add --title "<title>" --code <code> --from-file <path>
```

### Step 5: Verify

```bash
sdlc knowledge search "<topic>"
```

Confirm the new entries appear. Report what was added.

End with:

**Next:** `/sdlc-knowledge <topic>` to query the newly indexed knowledge.

---

## Mode: `maintain`

Maintenance pass — identify stale, orphaned, or miscategorized entries.

### Step 1: Status check

```bash
sdlc knowledge status
sdlc knowledge list
sdlc knowledge catalog show
```

### Step 2: Staleness check

List entries older than 90 days or with `staleness_flags`:

```bash
sdlc knowledge list --json | jq '.[] | select(.updated_at < "CUTOFF_DATE")'
```

For each potentially stale entry, verify its content is still accurate:
- Check if referenced files or patterns still exist
- Compare to current codebase if relevant

If stale, update the summary:
```bash
sdlc knowledge update <slug> --summary "<updated summary>"
```

### Step 3: Catalog coverage

Identify entries in `uncategorized` and suggest the right catalog class. For entries that can be reclassified:

```bash
sdlc knowledge update <slug> --code <new-code>
```

### Step 4: Report

Summarize:
- Entries reviewed
- Entries updated (staleness or reclassification)
- Entries that need human attention (flag as a comment)
- Catalog gaps found (classes with no entries)

End with:

**Next:** `/sdlc-knowledge` for catalog overview, or `/sdlc-suggest` to explore what to improve next.

---

## Quick reference

| Command | Purpose |
|---|---|
| `sdlc knowledge status` | Overview — entry count, catalog size, last maintained |
| `sdlc knowledge list` | All entries |
| `sdlc knowledge list --code-prefix <code>` | Filter by catalog class |
| `sdlc knowledge search <query>` | Full-text search |
| `sdlc knowledge show <slug>` | Full entry with content |
| `sdlc knowledge add --title "..." --code <code>` | Add entry |
| `sdlc knowledge update <slug> --summary "..."` | Update entry |
| `sdlc knowledge catalog show` | Catalog taxonomy |
| `sdlc knowledge librarian init` | Seed from workspaces |
| `sdlc knowledge session log <slug> --content "..."` | Log a research session |
| `sdlc knowledge ask "<question>"` | Ask the librarian — synthesized answer with cited entry codes |
"#;

const SDLC_KNOWLEDGE_PLAYBOOK: &str = r#"# sdlc-knowledge

Query and manage the project knowledge base.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Dispatch by argument

| Argument | Mode |
|---|---|
| (none) | Catalog overview |
| `<topic>` | Query — synthesize answer with citations |
| `init` | Seed from ponders, investigations, guidelines |
| `research <topic>` | Active research + index new entries |
| `maintain` | Staleness/orphan/catalog-gap check |

## Steps by mode

**Catalog overview (no arg):**
1. `sdlc knowledge status` + `sdlc knowledge catalog show` + `sdlc knowledge list`
2. Report entry count, class breakdown, last maintained, recent entries.
3. **Next:** `/sdlc-knowledge <topic>` or `/sdlc-knowledge init`

**Query `<topic>`:**
1. `sdlc knowledge search "<topic>"`
2. `sdlc knowledge show <slug>` for each result (up to 5)
3. Synthesize answer with citations `[title] (slug)`. Note gaps. Suggest related.
4. **Next:** `/sdlc-knowledge research <topic>` or `/sdlc-ponder <slug>`

**init:**
1. `sdlc knowledge librarian init`
2. Report counts (new/updated investigations, ponders, guidelines, catalog).
3. **Next:** `/sdlc-knowledge` or `/sdlc-knowledge <topic>`

**research `<topic>`:**
1. `sdlc knowledge search "<topic>"` + `sdlc knowledge catalog show`
2. Grep/Glob/Read codebase for relevant patterns, conventions, decisions.
3. WebSearch/WebFetch for community knowledge if needed.
4. `sdlc knowledge add --title "..." --code <code> --content "..."` for each finding.
5. `sdlc knowledge search "<topic>"` to verify indexing.
6. **Next:** `/sdlc-knowledge <topic>`

**maintain:**
1. `sdlc knowledge status` + `sdlc knowledge list` + `sdlc knowledge catalog show`
2. Check for entries older than 90 days; verify still accurate.
3. `sdlc knowledge update <slug> --summary "..."` for stale entries.
4. Reclassify uncategorized entries: `sdlc knowledge update <slug> --code <new-code>`.
5. Report: entries reviewed, updated, needing attention, catalog gaps.
6. **Next:** `/sdlc-knowledge` or `/sdlc-suggest`
"#;

const SDLC_KNOWLEDGE_SKILL: &str = r#"---
name: sdlc-knowledge
description: Query and manage the project knowledge base. Five modes — catalog overview, topic query with synthesis, init (seed from workspaces), research (find and index new knowledge), maintain (staleness and catalog-gap check).
---

# SDLC Knowledge Skill

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Dispatch

| Argument | Mode |
|---|---|
| (none) | Catalog overview: `sdlc knowledge status`, `sdlc knowledge catalog show`, `sdlc knowledge list` |
| `<topic>` | Query: `sdlc knowledge search`, `sdlc knowledge show`, synthesize with citations |
| `init` | `sdlc knowledge librarian init` — seed from ponders/investigations/guidelines |
| `research <topic>` | Search, gather (Grep/WebSearch), `sdlc knowledge add`, verify |
| `maintain` | List + staleness check + reclassify uncategorized + report |

## Workflow

1. Parse `$ARGUMENTS` to determine mode.
2. Run the mode steps above.
3. Always end with **Next:** pointing to the natural follow-up command.
"#;

pub static SDLC_KNOWLEDGE: CommandDef = CommandDef {
    slug: "sdlc-knowledge",
    claude_content: SDLC_KNOWLEDGE_COMMAND,
    gemini_description:
        "Query and manage the project knowledge base — catalog overview, topic synthesis, init, research, and maintenance",
    playbook: SDLC_KNOWLEDGE_PLAYBOOK,
    opencode_description:
        "Query and manage the project knowledge base — catalog overview, topic synthesis, init, research, and maintenance",
    opencode_hint: "[<topic> | init | research <topic> | maintain]",
    skill: SDLC_KNOWLEDGE_SKILL,
};
