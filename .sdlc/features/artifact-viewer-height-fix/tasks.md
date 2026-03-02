# Tasks: Remove Artifact Height Cap

## T1 — Remove `max-h-96 overflow-y-auto` from ArtifactViewer content div

**File:** `frontend/src/components/features/ArtifactViewer.tsx`

**Change:** On line 36, replace:
```tsx
<div className="p-4 max-h-96 overflow-y-auto">
```
with:
```tsx
<div className="p-4">
```

**Acceptance:** The line `max-h-96 overflow-y-auto` no longer appears in `ArtifactViewer.tsx`. The artifact card renders at full content height with no inner scrollbar.

---

This is the only task. The spec and design are explicit: one file, one line, no backend, no new components.
