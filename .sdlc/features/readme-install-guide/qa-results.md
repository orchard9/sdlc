# QA Results: SSH and make install in README

## Run: 2026-03-02

### Check 1 — SSH URL option present and correctly formatted

- [x] `cargo install --git ssh://git@github.com/orchard9/sdlc sdlc-cli` — present at line 31
- [x] SSH URL labeled with `# Multi-SSH-key setups, corporate proxies — use SSH URL:` — present at line 30
- [x] HTTPS URL present with `# Or if HTTPS works in your environment:` comment — present at lines 33–34
- [x] SSH URL appears before HTTPS URL — confirmed (line 31 before line 34)

**PASS**

### Check 2 — Build from source subsection present

- [x] `**Build from source** (after cloning):` label — present at line 39
- [x] `git clone git@github.com:orchard9/sdlc.git` — present at line 42
- [x] `cd sdlc` and `make install` — present at lines 43–44
- [x] Description sentence "`make install` builds the frontend and installs the binary in one step." — present at line 47
- [x] `[DEVELOPER.md](DEVELOPER.md)` link — present at line 48

**PASS**

### Check 3 — DEVELOPER.md callout at end of install section

- [x] Blockquote callout `> For the full contributor development setup...` — present at line 50
- [x] Text matches spec exactly — confirmed
- [x] Link `[DEVELOPER.md](DEVELOPER.md)` present in callout — confirmed

**PASS**

### Check 4 — No regressions

- [x] macOS/Linux prebuilt installer line unchanged (line 12)
- [x] Windows prebuilt installer line unchanged (line 18)
- [x] Homebrew line unchanged (line 24)
- [x] HTTPS `cargo install` line preserved (line 34)
- [x] `The build script automatically compiles the frontend — no manual npm step needed.` preserved (line 37)
- [x] No other sections of README.md modified — confirmed

**PASS**

### Check 5 — Links resolve

- [x] `DEVELOPER.md` exists at `/Users/jordanwashburn/Workspace/orchard9/sdlc/DEVELOPER.md` — confirmed

**PASS**

## Result

All 5 check groups: **PASS**

Ready for merge.
