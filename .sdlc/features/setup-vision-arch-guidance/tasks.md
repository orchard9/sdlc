# Tasks: Vision and Architecture Guidance in Setup

## T1 — README: Add "First steps" section

Add a "First steps" subsection to `README.md` under the "Initialize a project" section explaining that Vision and Architecture should be defined before creating features, with a code block showing `sdlc ui` and bullet explanations for Vision and Architecture.

**File:** `README.md`

---

## T2 — SetupPage: Improve Vision step subtitle

Replace the Vision step subtitle text (step 2) in `SetupPage.tsx` with copy that explains what Vision is used for by AI agents, per the spec.

**File:** `frontend/src/pages/SetupPage.tsx`

---

## T3 — SetupPage: Improve Architecture step subtitle

Replace the Architecture step subtitle text (step 3) in `SetupPage.tsx` with copy that explains what Architecture is used for by AI agents, per the spec.

**File:** `frontend/src/pages/SetupPage.tsx`

---

## T4 — Dashboard: Refine setup-incomplete banner condition and text

Change the `setupIncomplete` condition in `Dashboard.tsx` to fire only when Vision or Architecture is missing (not when team is missing). Update the banner text to be specific about Vision/Architecture.

**File:** `frontend/src/pages/Dashboard.tsx`
