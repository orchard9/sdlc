# Vision: v37-project-hub

## Why this matters

Jordan runs multiple sdlc instances simultaneously — locally across different projects,
and in the cluster across 47+ deployed pods. Today there's no single place to see
what's running or navigate to a specific project. Browser tabs all say "sdlc". Port
numbers are meaningless. In the cluster, you have to already know the slug.

## What a user can do when this ships

- Navigate to one bookmark (`localhost:9999` locally or `sdlc.threesix.ai` in the cluster)
  and see every live sdlc project as a card
- Type to filter the list by name — "pay" narrows 47 entries to 2 instantly
- See at a glance which projects are actively running an agent (pulsing badge)
- Click any card to navigate directly to that project
- Start a new project with `sdlc serve` and watch it appear in the hub automatically,
  without any manual configuration
- Kill a project and watch it disappear from the hub after ~5 minutes of silence
- Distinguish browser tabs immediately because each tab now shows `sdlc — {project-name}`

## What this does NOT do

- No cross-project management or agents from the hub
- No aggregate dashboards or statistics
- No groups, tags, or favorites
- Hub is read-only navigate-to, not operate-from
