# AMA — Ask Me Anything

Answers questions about the codebase by searching a pre-built keyword index.

## Setup (run once)

```bash
sdlc tool run ama --setup
```

## Usage

```bash
sdlc tool run ama --question "where is JWT validation?"
sdlc tool run ama --question "how does feature transition work?"
```

## How it works

1. `--setup` walks source files, chunks them into 40-line windows, extracts keyword tokens,
   and writes `.sdlc/tools/ama/index/chunks.json`
2. `--run` scores chunks by keyword overlap with your question, returns top file excerpts
3. Your AI assistant reads the excerpts and synthesizes an answer

## Configuration

Edit `.sdlc/tools/ama/config.yaml` to change which file extensions are indexed
or to adjust chunk size, overlap, and result count.

## Index location

`.sdlc/tools/ama/index/chunks.json` — gitignored, regenerate with `--setup`

## Re-index when needed

Re-run `--setup` after significant file changes. It's fast and safe to run any time.
