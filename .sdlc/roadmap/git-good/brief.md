ponder expects that its host project is itself a git repository. the git repo can be in some different basic states that should be clearly identified in the ui:

1. no pending changes and fully sync'd with origin, everything is clean/green
2. local changes are pending, not yet committed
3. local commits have not been pushed
4. origin commits have been pushed that we have not yet pulled

We need a multi-part project here. Initially, we want high level status button that is active when there are changes pending or commits pending for push, and clicking the button runs `/sdlc-commit`.  When clean/green show something fun instead of the button in that case.

The longer term project is we need a new section in the UI like Ponder, Root Cause, Evolve, etc called Git.  Put it in the Integrate area above Network. The Git management UI shows a file browser column that visualizes all the files in the workspace, allowing for filters of all files, only modified files, and other useful filters. The view should be easily togglable from flat view with full paths to tree view with hierarchical representations of the files.

Clicking on any given file opens in the main content view the diff viewer.  This is a side-by-side diff view if the viewport is wide enough, otherwise it's a patch-style chunk diff view. Use colors to accentuate diffs of actual content compared to whitespace.