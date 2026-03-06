# Acceptance Test: v36-spike-interface

## Setup

Create three test spikes:

```bash
# ADAPT spike
mkdir -p .sdlc/spikes/test-adapt-spike
cat > .sdlc/spikes/test-adapt-spike/findings.md << 'EOF'
# Spike: test-adapt-spike
**Verdict:** ADAPT
**Date:** 2026-03-04
## The Question
Can we test the spike promote flow?
## Success Criteria
A ponder is created with findings seeded.
## Candidates Evaluated
| Candidate | Verdict | Reason |
|---|---|---|
| Approach A | adapt | Works with modification |
## Risks and Open Questions
- First open question about scope
- Second open question about timing
EOF

# ADOPT spike
mkdir -p .sdlc/spikes/test-adopt-spike
cat > .sdlc/spikes/test-adopt-spike/findings.md << 'EOF'
# Spike: test-adopt-spike
**Verdict:** ADOPT
**Date:** 2026-03-04
## The Question
Can we test the ADOPT pathway?
## Risks and Open Questions
- None significant
EOF

# REJECT spike
mkdir -p .sdlc/spikes/test-reject-spike
cat > .sdlc/spikes/test-reject-spike/findings.md << 'EOF'
# Spike: test-reject-spike
**Verdict:** REJECT
**Date:** 2026-03-04
## The Question
Can we test auto-filing to knowledge?
EOF
```

---

## CLI Tests

```bash
# List shows all three with correct verdicts
sdlc spike list
# Expected: table with test-adapt-spike (ADAPT), test-adopt-spike (ADOPT), test-reject-spike (REJECT)

# Show prints full findings
sdlc spike show test-adapt-spike
# Expected: full findings.md content

# Promote creates a pre-seeded ponder
sdlc spike promote test-adapt-spike
# → prints ponder slug (test-adapt-spike)
# → .sdlc/roadmap/test-adapt-spike/spike-findings.md exists
# → .sdlc/roadmap/test-adapt-spike/open-questions.md contains "First open question"
# → .sdlc/spikes/test-adapt-spike/state.yaml has ponder_slug: "test-adapt-spike"

# REJECT auto-files to knowledge on list call (no explicit command needed)
sdlc spike list
# → knowledge base now contains entry for test-reject-spike
sdlc knowledge search "test-reject-spike"
# → returns the filed entry
```

---

## UI Tests

### Empty State (before spikes exist — test with no .sdlc/spikes/ dir)
1. Navigate to `/spikes`
2. See empty state with: what a spike is, `/sdlc-spike <slug> — <need>` format example
3. NOT just a blank page or "0 spikes"

### List View
1. Navigate to `/spikes` — all three spikes visible with verdict badges
2. ADAPT spike row: "promote to ponder →" affordance visible
3. ADOPT spike row: "findings →" + next-step hint chip visible; NO promote button
4. REJECT spike row: "auto-filed to knowledge" badge; NO action button

### Lineage (after promoting ADAPT)
1. Click "promote to ponder →" on ADAPT spike → promote view appears
2. Submit → navigates to `/ponder/test-adapt-spike`
3. Navigate back to `/spikes` — ADAPT spike row now shows "Ponder: test-adapt-spike" as a link

### ADOPT Next-Step Guidance
1. Click ADOPT spike → detail view
2. See "What's next" section with: "This spike was adopted. Next: /sdlc-hypothetical-planning test-adopt-spike"
3. Hint explains ADOPT = proven approach, not yet implemented

### REJECT Auto-Filing
1. REJECT spike row shows "auto-filed to knowledge" indicator
2. No manual button to click — it happened automatically on page load

### Detail View (ADAPT)
1. Click ADAPT spike → structured findings: The Question / Success Criteria / Candidates table / Risks and Open Questions
2. "Promote to Ponder" button visible; click → promote view
3. Promote view shows: spike-findings.md ✓, open-questions.md ✓, ponder title (from The Question) ✓, prototype code — (ephemeral)
4. Editable ponder slug field (default = spike slug)
5. CLI equivalent hint: `sdlc spike promote test-adapt-spike`
