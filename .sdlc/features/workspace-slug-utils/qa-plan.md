# QA Plan: workspace-slug-utils

## Scope

Verify that the `titleToSlug` refactor is complete, correct, and leaves no regressions. Since this is a pure code extraction with no behavioral change, QA is focused on structural correctness and build health.

## Checks

### 1. No local copies remain
```bash
grep -rn "function titleToSlug" frontend/src
```
Expected: zero results.

### 2. Shared module exists and is correct
- `frontend/src/lib/slug.ts` exists.
- It exports `titleToSlug` with the original 5-step implementation (lowercase → strip non-alnum → collapse spaces → collapse hyphens → trim hyphens).

### 3. All 5 files import from the shared module
```bash
grep -n "titleToSlug" \
  frontend/src/components/ponder/NewIdeaModal.tsx \
  frontend/src/pages/InvestigationPage.tsx \
  frontend/src/pages/PonderPage.tsx \
  frontend/src/pages/GuidelinePage.tsx \
  frontend/src/pages/EvolvePage.tsx
```
Each file should show only an `import` line — no function definition.

### 4. TypeScript build passes
```bash
cd frontend && npm run build
```
Expected: exits 0, no TypeScript errors.

### 5. Behavioral spot-check (manual)
Input → Expected output:
- `"Hello World"` → `"hello-world"`
- `"React + TypeScript"` → `"react-typescript"`
- `"  leading/trailing spaces  "` → `"leadingtrailing-spaces"`
- `"Multiple---hyphens"` → `"multiple-hyphens"`
- `"-leading hyphen"` → `"leading-hyphen"`
