# QA Plan: Vision and Architecture Guidance in Setup

## Scope

UI text changes and a README addition. No backend logic, no new components, no API changes.

## Test Cases

### TC1 — README "First steps" section present

**Steps:**
1. Open `README.md` in the repo root.
2. Locate the "Initialize a project" section.
3. Verify a "First steps" subsection exists immediately after it.

**Expected:**
- Section heading `### First steps` present.
- `sdlc ui` code block present.
- Bullet for **Vision** explaining it anchors agent decisions.
- Bullet for **Architecture** explaining it maps constraints.
- "Once Vision and Architecture are defined, you're ready to create features." sentence present.

---

### TC2 — SetupPage Vision subtitle improved

**Steps:**
1. Run `sdlc ui` and navigate to `/setup`.
2. Complete step 1 (Project Info) or click the Vision step indicator.
3. Observe the subtitle text under the "Vision" heading.

**Expected:**
- Text mentions "why this project exists and who it serves".
- Text mentions `VISION.md` and that agents use it.
- Text does not say just "Edit the generated vision or write your own" (old text).

---

### TC3 — SetupPage Architecture subtitle improved

**Steps:**
1. Navigate to step 3 (Architecture) in `/setup`.
2. Observe the subtitle text under the "Architecture" heading.

**Expected:**
- Text mentions "key components, tech stack, and constraints".
- Text mentions `ARCHITECTURE.md` and that agents use it to understand what's in scope.
- Text does not say just "Edit the generated architecture or write your own" (old text).

---

### TC4 — Dashboard banner fires when Vision or Architecture missing

**Steps:**
1. Remove or rename `VISION.md` in the project root so `api.getVision()` returns `exists: false`.
2. Navigate to `/` (Dashboard).
3. Observe the top banner.

**Expected:**
- Banner visible with text about Vision or Architecture not being defined.
- Banner links to `/setup`.

---

### TC5 — Dashboard banner does not fire when Vision and Architecture exist

**Steps:**
1. Ensure `VISION.md` and `ARCHITECTURE.md` both exist.
2. Navigate to `/` (Dashboard).

**Expected:**
- No amber banner about Vision/Architecture.
- Dashboard loads normally.

---

### TC6 — Dashboard banner does not fire solely due to missing team

**Steps:**
1. Ensure `VISION.md` and `ARCHITECTURE.md` both exist.
2. Ensure no agents are configured (`api.getProjectAgents()` returns `[]`).
3. Navigate to `/` (Dashboard).

**Expected:**
- No amber Vision/Architecture banner (team-only absence should not trigger it).

---

## Pass Criteria

All 6 test cases pass. TypeScript compiles without errors (`npm run build` in `frontend/`).
