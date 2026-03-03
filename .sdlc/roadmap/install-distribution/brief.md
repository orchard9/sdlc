# Brief: Cross-Platform Install and Distribution

**Origin:** Extracted from team conversation on 2026-03-02

## Summary

The current `make`-based install workflow is broken for Windows (make unavailable) and failing on Ubuntu (TypeScript build errors block `make install`). A contributor (Xist) was blocked from installing sdlc on both platforms. jx12n has a concrete plan: copy the envault distribution channel pattern and replace `make` reliance with a project CLI.

## Key Signals

- **STRONG** [Product] `make` does not work on Windows — Xist couldn't install at all
- **STRONG** [Engineering] `make install` on Ubuntu fails: 4 TypeScript errors in frontend break the build (`RunsHeatmap.tsx:40`, `ActionsPage.tsx:516/713/722`)
- **STRONG** [Engineering] SSH hostname aliases (e.g. `github-rdp:orchard9/sdlc`) must be respected — sdlc must not override git config
- **STRONG** [Strategy] jx12n wants to copy the envault distribution pattern and replace make with a project CLI across all projects

## Relevant Excerpts

> Xist: "Speaking of tools...  make doesn't work on Windows / cmake works on Windows"
> Xist: "I had to hack it to get it to install / Because it always wants a https://github.com/... url"
> jx12n: "yeah, ok - i'll work on the install experience and distribution - i can do a similar distribution channel that we ended up landing on for envault / ill just copy that pattern over"
> jx12n: "i can switch my reliance on make files across all the projects to just a cli for that project, too / in whatever language / its easy"
> Xist: "I would have to do it like this: git clone github-rdp:orchard9/sdlc.git / cd sdlc / cmake install / And then sdlc needs to never try to access git other than how it's already configured in that repo"
> Xist: [Ubuntu make install fails with 4 TypeScript errors — RunsHeatmap.tsx unused var, ActionsPage.tsx type mismatches]

## Open Questions

- What exactly is the envault distribution channel? Binary releases via GitHub? Homebrew tap? Cargo publish?
- Should the build system switch to `cmake`, a custom `sdlc` bootstrap script, or something else?
- Are the TypeScript errors already being tracked, or are they new regressions?
- Should sdlc ship a single `install.sh` + `install.ps1` pair, or a binary that works everywhere?
- How do we handle the npm/frontend build dependency in a cross-platform context?
