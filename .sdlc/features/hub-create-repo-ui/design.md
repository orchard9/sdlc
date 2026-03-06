# Design: CreateRepoSection UI

## Component Structure

```
HubPage (fleet view)
  └── <section> "Add New Project"
        <CreateRepoSection />
          state: idle    → NameForm
          state: creating → NameForm (disabled + spinner)
          state: done    → InstructionsDisplay
          state: error   → NameForm + error message
```

## Step 1 — NameForm

```
┌──────────────────────────────────────────────┐
│  [ my-new-project              ]  [+ Create]  │
│  lowercase letters, numbers, and hyphens      │
└──────────────────────────────────────────────┘
```

- Input: `type="text"`, placeholder `"project-name"`, pattern hint below
- Create button: primary style, disabled when input empty or state=creating
- Inline error below input on invalid name or API error

## Step 2 — InstructionsDisplay

```
┌──────────────────────────────────────────────────────────────────┐
│  ✓ my-new-project created                                         │
│                                                                   │
│  Add remote:                                                      │
│  ┌──────────────────────────────────────────────────────┐  [Copy]│
│  │ git remote add gitea http://claude-agent:…@host/…    │        │
│  └──────────────────────────────────────────────────────┘        │
│                                                                   │
│  Push:                                                            │
│  ┌──────────────────────────────────────────────────────┐  [Copy]│
│  │ git push gitea main                                  │        │
│  └──────────────────────────────────────────────────────┘        │
│                                                                   │
│  This is your deployment remote — push here to update            │
│  your cluster instance.                                           │
│                                                                   │
│  [+ Add another project]                                          │
└──────────────────────────────────────────────────────────────────┘
```

- Token in push_url shown truncated in display (full value in clipboard)
- "Copied!" transient state on copy button for 1.5s
- "Add another project" resets to Step 1

## Mockup

[Mockup](mockup.html)

## State Machine

```
idle ──[Create clicked]──► creating ──[API ok]──► done ──[Add another]──► idle
                     └──[API error]──► error ──[any input]──► idle
```

## Files Changed

| File | Change |
|---|---|
| `frontend/src/pages/HubPage.tsx` | Add `CreateRepoSection` component, add section in fleet view |
| `frontend/src/api/client.ts` | Add `createRepo()` call |
| `frontend/src/lib/types.ts` | Add `CreateRepoResponse` interface |
