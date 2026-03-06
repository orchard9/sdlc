## Acceptance Test: Ponder Merge

### Scenario: Merge two ponder entries end-to-end

**Setup:**
1. Create two ponder entries: `sdlc ponder create merge-source --title "Source entry"` and `sdlc ponder create merge-target --title "Target entry"`
2. Add a session to the source: write a session file in `.sdlc/roadmap/merge-source/sessions/session-001.md`
3. Add an artifact to the source: write `.sdlc/roadmap/merge-source/test-artifact.md`

**Execute:**
```bash
sdlc ponder merge merge-source --into merge-target
```

**Verify:**
- [ ] Output shows "Merged 'merge-source' → 'merge-target'" with session/artifact/team counts
- [ ] Source manifest has `status: parked` and `merged_into: merge-target`
- [ ] Target manifest has `merged_from: [merge-source]`
- [ ] Source session file was copied to target sessions dir (renumbered, with merge header comment)
- [ ] Source artifact was copied to target dir
- [ ] `sdlc ponder show merge-source` displays redirect banner: "This entry was merged into 'merge-target'"
- [ ] `sdlc ponder list` hides the merged source entry by default
- [ ] `sdlc ponder list --all` shows the merged source with `↗ parked→merge-target` status
- [ ] Pre-condition: merging a committed source is rejected with clear error
- [ ] Pre-condition: merging into a committed target is rejected with clear error
