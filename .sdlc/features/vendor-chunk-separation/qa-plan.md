# QA Plan: Vendor Chunk Separation

## Test 1: Build produces vendor chunks

Run `npm run build` in `frontend/`. Verify the `dist/assets/` directory contains files matching:
- `vendor-react-*.js`
- `vendor-markdown-*.js`
- `vendor-mermaid-*.js`

**Pass criteria:** All three vendor chunk files exist.

## Test 2: No build errors or warnings

The `npm run build` command exits with code 0 and produces no error output.

**Pass criteria:** Clean build with exit code 0.

## Test 3: Vendor chunks contain expected packages

Inspect the built vendor chunk files (search for characteristic strings):
- `vendor-react` contains `react` runtime code (e.g., `createElement` or `jsx`)
- `vendor-markdown` contains markdown processing code
- `vendor-mermaid` contains mermaid diagramming code

**Pass criteria:** Each chunk contains code from its assigned package group.

## Test 4: Application code chunk is separate

The main application chunk (index-*.js) does not contain vendor library code that should be in the vendor chunks.

**Pass criteria:** Application chunk is smaller than the combined vendor chunks.

## Test 5: Existing tests pass

Run `npm test` (if tests exist) to confirm no regressions.

**Pass criteria:** All existing frontend tests pass.
