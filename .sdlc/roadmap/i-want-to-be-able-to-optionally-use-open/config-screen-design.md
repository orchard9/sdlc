## Config Screen Design: Endpoints Page

### Placement
- Sidebar nav item: 'Endpoints' (icon: Cpu or Settings2)
- Positioned between Roadmap and sidebar footer
- Navigates to /endpoints — dedicated full-page view

### Page layout: two-pane (same pattern as milestones/features)
- Left pane: endpoint list with live status indicators
- Right pane: selected endpoint detail / chain builder / action overrides

### Wireframe
```
┌─ ENDPOINTS ────────────────────────────────────────────────────────────┐
│  ┌─ Endpoint List ──────────┐  ┌─ Selected: opus-primary ────────────┐ │
│  │                          │  │  Model:     claude-opus-4-6         │ │
│  │ ● opus-primary    [OK]   │  │  Provider:  anthropic               │ │
│  │ ● sonnet-main     [OK]   │  │  Status:    ✓ Available             │ │
│  │ ○ haiku-cheap  [cool 4m] │  │  Last used: 3 minutes ago          │ │
│  │                          │  │  Last fail: 18 min ago (rate_limit)│ │
│  │ [+ Add Endpoint]         │  │  [Reset Cooldown]                   │ │
│  └──────────────────────────┘  └────────────────────────────────────┘ │
│                                                                         │
│  ┌─ Default Chain ────────────────────────────────────────────────────┐│
│  │  ① opus-primary  →  ② sonnet-main  →  ③ haiku-cheap              ││
│  │  [↑↓ reorder]    [+ Add Step]    [Save Chain]                     ││
│  └────────────────────────────────────────────────────────────────────┘│
│                                                                         │
│  ┌─ Action Type Overrides ─────────────────────────────────────────────┐│
│  │  ACTION              CHAIN                                     DEL  ││
│  │  create_spec    → [haiku-cheap, sonnet-main]                   [×]  ││
│  │  implement_task → [opus-primary, sonnet-main]                  [×]  ││
│  │  approve_review → [opus-primary]                               [×]  ││
│  │  [+ Add Override]                                                   ││
│  └────────────────────────────────────────────────────────────────────┘│
└────────────────────────────────────────────────────────────────────────┘
```

### Status indicators
- ● green: available (cooldown_until is None or in the past)
- ○ yellow: on cooldown — shows 'cool Xm' countdown
- ● red: reserved for permanent failure (future)
- Status updates via SSE push (AgentEndpointCooldown, AgentEndpointRecovered events)

### Per-milestone override
- NOT on the Endpoints page
- Lives in MilestoneDetailPane (existing component)
- Shows: 'Agent Chain: [opus-primary, sonnet-main] (milestone override)' with inline edit
- Edits milestone.agent_chain in the milestone manifest.yaml

### Interactions
- Click endpoint → select in right pane (shows detail)
- [+ Add Endpoint] → inline form in right pane
- [Reset Cooldown] → POST /api/agent/state/:id/reset
- [Reset All] button in page header → POST /api/agent/state/reset-all
- Chain reorder → drag-and-drop or ↑/↓ buttons
- [Save Chain] → PATCH /api/agent/config

### Data fetching
- GET /api/agent/config on mount
- GET /api/agent/state on mount
- SSE subscription for live endpoint status updates
- PATCH /api/agent/config on save actions

### Files to create
- frontend/src/pages/EndpointsPage.tsx
- frontend/src/components/endpoints/EndpointList.tsx
- frontend/src/components/endpoints/EndpointDetail.tsx
- frontend/src/components/endpoints/ChainBuilder.tsx
- frontend/src/components/endpoints/ActionTypeOverrides.tsx
- frontend/src/api/client.ts — add agentConfig and agentState API methods
