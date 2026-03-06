# Spec: Replace hub empty state — Add Project as primary action

## Problem

When no fleet projects exist, HubPage shows `EmptyState` with a static monospace instruction:
> "Configure projects to send heartbeats. See ~/.sdlc/hub.yaml"

This is v1 heartbeat infrastructure. The v2 cluster model uses Gitea + git-sync — no heartbeat
config needed. A new user seeing this message has no actionable path.

## Solution

Replace the static instruction in `EmptyState` with the `CreateRepoSection` component so a
user can immediately create and onboard a project without leaving the page.

## Change

In `frontend/src/pages/HubPage.tsx`, replace:

```tsx
function EmptyState() {
  return (
    <div className="flex flex-col items-center justify-center py-20 gap-4 text-center">
      <Layers className="w-12 h-12 text-muted-foreground opacity-30" />
      <div className="text-lg font-semibold text-muted-foreground">No projects registered</div>
      <div className="text-sm text-muted-foreground bg-muted/40 border border-border rounded-lg px-5 py-3 font-mono">
        Configure projects to send heartbeats. See ~/.sdlc/hub.yaml
      </div>
    </div>
  )
}
```

With:

```tsx
function EmptyState() {
  return (
    <div className="flex flex-col items-center justify-center py-20 gap-6 max-w-md mx-auto w-full">
      <div className="text-center">
        <Layers className="w-12 h-12 text-muted-foreground opacity-30 mx-auto mb-3" />
        <div className="text-lg font-semibold">No projects yet</div>
        <p className="text-sm text-muted-foreground mt-1">
          Create a Gitea repo and push your local project to get started.
        </p>
      </div>
      <div className="w-full">
        <CreateRepoSection />
      </div>
    </div>
  )
}
```

## Scope

Single component, ~15 lines changed. No new state, no new API calls, no props.
The `CreateRepoSection` component is already defined above in the same file.

## Out of Scope

- Removing the heartbeat infrastructure (`hub-server-mode`, `hub-heartbeat-client`) — separate work
- Showing different empty states based on fleet availability
