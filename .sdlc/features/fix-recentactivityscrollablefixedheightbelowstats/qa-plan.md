# QA Plan: Recent Activity Scrollable Fixed-Height Below Stats

## TC-1 — Visual order on Dashboard

**Steps:**
1. Open the Dashboard with at least one changelog event present.
2. Observe the vertical order of sections.

**Expected:** Project header → stats bar → Recent Activity banner → escalations/wave plan → milestones.
**Pass criteria:** `WhatChangedBanner` does NOT appear above the project header or stats bar.

## TC-2 — Scrollable container when events exceed visible area

**Steps:**
1. Ensure 8+ changelog events exist (or mock via dev tools / seed data).
2. Open Dashboard and locate the Recent Activity banner.

**Expected:** The event list is contained within a fixed-height box (~192 px). A vertical scrollbar appears when content overflows. The "See more" button is absent.
**Pass criteria:** Page layout does not shift when the banner is present; no "See more" button visible.

## TC-3 — Dismiss still works

**Steps:**
1. Open Dashboard with events visible.
2. Click the "Dismiss" button in the banner header.

**Expected:** Banner disappears; rest of page remains in place.

## TC-4 — Empty / dismissed state

**Steps:**
1. Dismiss the banner (or ensure no events).
2. Reload Dashboard.

**Expected:** Banner is not rendered; no empty placeholder shown.
