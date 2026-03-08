# Technical Decisions

## Diff Viewer Library: `@git-diff-view/react`

**Why this one:**
- Actively maintained, GitHub-style UI
- Split + unified modes built-in
- Accepts standard unified diff format (native `git diff` output)
- Syntax highlighting included
- SSR/RSC ready (future-proofs for streaming)
- ~35KB gzipped — acceptable for a dev tool

**Alternatives considered:**
- `react-diff-viewer-continued` — good but less polished, no built-in syntax highlighting
- `react-diff-view` — powerful token system but more complex API
- Custom build — too much effort for a diff renderer, not our core value

## File Browser: Custom Component

**Why build in-house:**
- Tree/flat toggle is trivial (split path, build tree client-side)
- Need tight control over git status badges and our design system
- Library would fight our Tailwind styling
- Data structure is simple: `{ path, status, staged, old_path? }`

## Backend: Two Endpoints

```
GET /api/git/status   → GitStatus composite
GET /api/git/diff     → { path, diff_text, is_binary, old_path? }
GET /api/git/files    → Vec<GitFile>  (for full file browser)
```

Rust implementation uses `std::process::Command` to call git CLI.
No libgit2 dependency — git CLI is always available in our environments.

⚑ Decided: @git-diff-view/react for diff rendering
⚑ Decided: Custom file browser component
⚑ Decided: git CLI over libgit2 for backend
? Open: Should we add `GET /api/git/log` for commit history view later?