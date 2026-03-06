# sdlc-recap Skill Design

## Purpose
Close out a work session with a state-aware recap that captures progress, identifies remaining work, and proposes forward motion for hard problems.

## When to use
- After UAT failure (Pathway 3 in the UAT template)
- End of a long work session
- When stuck and need to regroup
- When handing off between agents/sessions

## Template structure

```markdown
# sdlc-recap

Produce a state-aware session recap and propose forward motion for unresolved problems.

## Step 1 — Gather state

```bash
sdlc status --json
sdlc milestone info <active-milestone> --json  # if applicable
git log --oneline -20
git diff --stat HEAD~5  # recent changes
```

Read escalations, recent tasks, and feature states.

## Step 2 — Synthesize

Produce:

### Working On
What was the goal? State it clearly.

### Completed
- Concrete deliverables with file paths
- Features advanced, tasks completed
- Artifacts written

### Remaining  
- Unfinished work with specific blockers
- For each blocker, classify:
  - **Fixable**: can be resolved in next session → create task
  - **Needs input**: requires human decision → create escalation  
  - **Complex**: needs design thinking → propose ponder session

### Forward Motion
For each "Complex" item:
```bash
sdlc ponder create "<problem-as-question>" --brief "<one-paragraph context>"
```

For each "Needs input" item:
```bash
sdlc escalate create --kind <type> --title "..." --context "..."
```

For each "Fixable" item:
```bash
sdlc task add <feature> "..."
```

## Step 3 — Commit completed work

```bash
git add -A && git commit -m "session: <summary of completed work>"
```

## Step 4 — Output

End with exactly one **Next:** line.
- If ponder sessions were created: `**Next:** /sdlc-ponder <first-slug>`
- If escalations were created: `**Next:** resolve escalations, then /sdlc-milestone-uat <slug>`
- If only tasks: `**Next:** /sdlc-run <first-task-feature>`
```

## Key properties
- State-aware: reads actual sdlc state, not just conversation context
- Always produces forward motion: every remaining item gets a concrete action
- Works standalone or as part of UAT failure pathway
- Follows the "/sdlc-* commands must orchestrate real work" principle
