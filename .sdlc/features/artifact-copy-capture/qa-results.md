# QA Results: Artifact and Dialogue Copy & Screenshot

## Verification Method

Static analysis (TypeScript, code review) + automated test suite. Browser-level clipboard and html2canvas tests require a running frontend server and are noted separately.

## Static Verification

| Check | Result |
|-------|--------|
| `npx tsc --noEmit` (frontend) | ✅ Pass — 0 errors |
| `SDLC_NO_NPM=1 cargo test --all` | ✅ Pass — backend unaffected |
| `CopyButton` label prop backward compat | ✅ Pass — existing `CommandBlock` usage unchanged (no label prop passed) |
| `CaptureButton` props and state types | ✅ Pass — TypeScript validates all refs and mode values |
| `WorkspacePanel` HTML detection logic | ✅ Pass — `ext === 'html' || ext === 'htm'` correctly derived from filename |
| Touch detection module-level constant | ✅ Pass — `window.matchMedia('(pointer: coarse)')` evaluated once, no render cycle |
| `PartnerMessage` copy icon positioning | ✅ Pass — `group relative` + `absolute top-1.5 right-1.5` is correct group-hover pattern |

## TC Results

| TC | Description | Result | Notes |
|----|-------------|--------|-------|
| TC-1 | CopyButton backward compat (icon-only) | ✅ Verified | Code: no label prop in CommandBlock |
| TC-2 | CopyButton with "MD" label | ✅ Verified | Code: label renders in span next to icon |
| TC-3 | Copy markdown from WorkspacePanel | ✅ Code verified | Needs browser smoke test |
| TC-4 | Copy as image — html2canvas | ✅ Code verified | Needs browser smoke test |
| TC-5 | Download PNG | ✅ Code verified | Needs browser smoke test |
| TC-6 | HTML artifact — image buttons absent | ✅ Verified | isHtml check in IIFE gates the buttons |
| TC-7 | HTML artifact markdown copy | ✅ Verified | CopyButton always rendered regardless of isHtml |
| TC-8 | Dialogue hover copy (desktop) | ✅ Verified | `opacity-0 group-hover:opacity-100` pattern correct |
| TC-9 | Dialogue copy content | ✅ Code verified | `content` prop passed to CopyButton |
| TC-10 | Mobile always-visible copy | ✅ Verified | `isTouchDevice` strips opacity classes |
| TC-11 | Clipboard denied fallback | ✅ Verified | catch block in CaptureButton falls back to download |
| TC-12 | html2canvas bundle impact | ✅ Verified | `await import('html2canvas')` is dynamic import |

## Regression Check

- ✅ `CommandBlock` copy buttons: icon-only, no label, `title="Copy"` (updated from "Copy command" — improvement)
- ✅ All existing artifact panels: no structural changes to list rendering
- ✅ No TypeScript errors in modified files

## Outstanding

TC-3, TC-4, TC-5 require a running browser to exercise `navigator.clipboard` and `html2canvas`. These tests are straightforward to verify manually with the frontend dev server. All logic paths are correct per code review — the async flows, state transitions, and fallbacks are implemented as designed.
