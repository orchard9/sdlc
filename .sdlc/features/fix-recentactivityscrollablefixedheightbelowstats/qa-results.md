# QA Results: Recent Activity Scrollable Fixed-Height Below Stats

## TC-1 — Visual order on Dashboard
**Result: PASS**
`WhatChangedBanner` renders at line 260 of Dashboard.tsx, after the stats bar closing tag (line 258) and before the "Needs Your Attention" escalations block (line 263). Visual order is: project header → stats bar → recent activity → escalations.

## TC-2 — Scrollable container when events exceed visible area
**Result: PASS**
Event list wrapper is `<div className="space-y-0.5 max-h-48 overflow-y-auto">`. All events are rendered without slicing. The "See more" button is absent. The `VISIBLE_COUNT` constant and `expanded` state variable no longer exist in the component.

## TC-3 — Dismiss still works
**Result: PASS**
The Dismiss button wiring (`onClick={dismiss}`) is unchanged. No modifications to dismiss logic or `useChangelog` hook.

## TC-4 — Empty / dismissed state
**Result: PASS**
`if (dismissed || events.length === 0) return null` is unchanged. Banner correctly renders nothing when dismissed or empty.

## Summary
All 4 test cases pass. No regressions identified.
