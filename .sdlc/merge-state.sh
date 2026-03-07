#!/usr/bin/env bash
# Custom git merge driver for .sdlc/state.yaml
#
# state.yaml is a derived index — features, milestones, and ponders are
# reconstructed from their respective directories on disk.  History is
# preserved from whichever side has the newer last_updated timestamp.
#
# Git calls this with: %O (ancestor) %A (ours) %B (theirs)
# Result must be written to %A.  Exit 0 = success, non-zero = conflict.

set -euo pipefail

ANCESTOR="$1"  # base
OURS="$2"      # current branch
THEIRS="$3"    # incoming branch

# Pick the side with the newer last_updated as the base (preserves more history).
ours_ts=$(grep '^last_updated:' "$OURS" 2>/dev/null | head -1 | sed 's/last_updated: *//')
theirs_ts=$(grep '^last_updated:' "$THEIRS" 2>/dev/null | head -1 | sed 's/last_updated: *//')

if [[ "$theirs_ts" > "$ours_ts" ]]; then
    cp "$THEIRS" "$OURS"
fi

# Rebuild lists from disk (features/, milestones/, roadmap/).
# If sdlc is available, use it.  Otherwise accept as-is (still better than conflict markers).
if command -v sdlc &>/dev/null; then
    sdlc state-rebuild 2>&1 || true
fi

exit 0
