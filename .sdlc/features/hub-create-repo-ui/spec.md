# Spec: CreateRepoModal — two-step UI: name form → instruction display

## Problem

The hub's fleet view has an Import section for existing GitHub repos but no path for a developer
starting from a local-only repo. There is no "Add Project" button — only an import form.

## Solution

Add a `CreateRepoSection` component to `HubPage.tsx` alongside the existing `ImportSection`.
It presents a two-step flow:

1. **Step 1 — Name form**: user types a project name, clicks Create
2. **Step 2 — Instructions**: shows two copy-able git commands and a clear explanation

Also add `createRepo()` to the API client and `CreateRepoResponse` to types.

## User Flow

```
HubPage (fleet view)
  └── "Add Project" section (new, below Import External Repo)
        Step 1: Name input + Create button
        Step 2: Instruction display
          "Add remote:"   [git remote add gitea <push_url>]  [copy]
          "Push:"         [git push gitea main]               [copy]
          Caption: "This is your deployment remote — push here to update your cluster instance."
          [Start another] button → reset to Step 1
```

## API Addition

`frontend/src/api/client.ts`:
```ts
createRepo: (name: string) =>
  request<CreateRepoResponse>('/api/hub/create-repo', {
    method: 'POST',
    body: JSON.stringify({ name }),
    headers: { 'Content-Type': 'application/json' },
  }),
```

## Type Addition

`frontend/src/lib/types.ts`:
```ts
export interface CreateRepoResponse {
  repo_slug: string
  push_url: string
  gitea_url: string
  provision_triggered: boolean
}
```

## Component: `CreateRepoSection`

States: `idle | creating | done | error`

- **idle**: name input + Create button
- **creating**: spinner + "Creating..." (input disabled)
- **done**: two instruction rows with copy buttons, caption, Start another button
- **error**: error message inline, reset to idle

Copy buttons use `navigator.clipboard.writeText()`. Each button shows a brief "Copied!" state for 1.5s.

## Name Validation (client-side)

Before calling the API, validate:
- Non-empty
- Matches `/^[a-z0-9][a-z0-9-]*$/`
- Max 100 chars

Show inline error without calling API if invalid.

## Placement in HubPage

Add `CreateRepoSection` as a new `<section>` inside the fleet-available view, below "Import External Repo":

```tsx
{/* Add New Project */}
<section>
  <h2 className="text-lg font-semibold mb-3">Add New Project</h2>
  <p className="text-xs text-muted-foreground mb-3">
    Create a new Gitea repo and push your local project to the cluster.
  </p>
  <CreateRepoSection />
</section>
```

## Out of Scope

- `hub-empty-state-replace` is a separate feature (same milestone) — not included here
- Deleting the repo if the user abandons the flow
- Auto-refreshing the available list after creation (SSE will handle discovery)
