# .ai Index

Project knowledge base. Entries are organized by category.

## Categories

- **patterns/** — How we do things (coding patterns, architectural conventions)
- **decisions/** — Why we chose X over Y (ADRs, trade-off notes)
- **gotchas/** — Non-obvious pitfalls and workarounds
- **architecture/** — How the system works (data flow, component relationships)
- **conventions/** — Naming, style, standards

## Usage

Entries are harvested automatically after each SDLC artifact is approved.
Each entry follows the format:

```
---
category: patterns
title: How we handle X
learned: YYYY-MM-DD
source: spec|design|review|human
confidence: high|medium|low
---

## Summary
...

## Key Facts
- ...

## File Pointer
`path/to/file.go:line-range`
```
