# Acceptance Test: v37-project-hub

## Setup

1. Start sdlc hub: `sdlc serve --hub` on port 9999
2. Start two project instances: `sdlc serve` in `payments-api/` on :3001 and `auth-service/` on :3004
   with `SDLC_HUB_URL=http://localhost:9999` set
3. Navigate to `http://localhost:9999`

## Scenarios

### 1. Projects appear automatically

- [ ] Within 35 seconds of starting `sdlc serve` for both projects, both cards appear
      in the hub listing without any manual action
- [ ] Each card shows the correct project name
- [ ] Each card shows the correct URL (localhost:3001 / localhost:3004)
- [ ] Green status dot on both cards

### 2. Filter works

- [ ] Typing in the filter box narrows cards in real time
- [ ] Filter matches on project name (partial match, case-insensitive)
- [ ] Count shown in filter box updates as filter changes ("2 projects" → "1 of 2")
- [ ] Clearing filter restores all cards

### 3. Card metadata is live

- [ ] If a project has an active milestone, the milestone slug appears on the card
- [ ] If an agent run is active, the "agent running" badge appears with pulse animation
- [ ] Badge disappears when the run completes

### 4. Status dot reflects liveness

- [ ] Kill one project instance (Ctrl+C)
- [ ] Within 90 seconds, that card's dot turns grey/yellow
- [ ] Within 5 minutes, the card is removed from the listing entirely

### 5. Hub persists across restart

- [ ] Stop the hub (`sdlc serve --hub`)
- [ ] Restart it
- [ ] Both project cards still appear immediately (loaded from hub-state.yaml cache)
- [ ] Cards reflect correct last-known state before restart

### 6. Page titles are distinct

- [ ] Browser tab for `localhost:3001` shows `sdlc — payments-api`
- [ ] Browser tab for `localhost:3004` shows `sdlc — auth-service`
- [ ] Browser tab for hub (`localhost:9999`) shows `sdlc hub`

### 7. Empty state

- [ ] Start hub with no project instances running
- [ ] Empty state renders with hub config hint
- [ ] No errors in console

### 8. No hub configured

- [ ] Start a project instance WITHOUT `SDLC_HUB_URL` set
- [ ] Project starts and serves normally
- [ ] No errors or warnings about missing hub
- [ ] Project does not attempt heartbeat connections
