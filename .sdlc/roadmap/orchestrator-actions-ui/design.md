# Actions Page Design

## Backend gaps to fill first

### 1. GET /api/orchestrator/actions
- Handler in `crates/sdlc-server/src/routes/orchestrator.rs`
- Calls `ActionDb::list_all()` (already exists in db.rs)
- Returns: JSON array of Action structs sorted by created_at desc

### 2. POST /api/orchestrator/actions
- Body: `{ label, tool_name, tool_input, scheduled_at: ISO8601, recurrence_secs? }`
- Creates an Action with `ActionTrigger::Scheduled { next_tick_at }`
- Returns 201 with the created Action

## API client methods (client.ts)

```typescript
api.listActions()                              // GET /api/orchestrator/actions
api.createAction({ label, tool_name, tool_input, scheduled_at, recurrence_secs? })
                                               // POST /api/orchestrator/actions
api.listWebhookRoutes()                        // GET /api/orchestrator/webhooks/routes
api.createWebhookRoute({ path, tool_name, input_template })
                                               // POST /api/orchestrator/webhooks/routes
```

## Sidebar addition (Sidebar.tsx)

Add to the `setup` group, after Agents:
```tsx
{ to: '/actions', icon: <Zap size={16} />, label: 'Actions' }
```

## App.tsx route

```tsx
<Route path="/actions" element={<ActionsPage />} />
```

## Page structure (ActionsPage.tsx)

Two stacked sections — NOT tabs. Visibility is the primary goal.

### Section 1: Scheduled Actions
- Header: "Scheduled Actions" + [+ Schedule Action] button
- Table columns: Label | Tool | Status (badge: Pending/Running/Completed/Failed) | Next Run | Last Result | (delete icon)
- Empty state: "No actions scheduled. Use the CLI: `sdlc orchestrate add`"
- Status badge colors: Pending=gray, Running=blue, Completed=green, Failed=red

### Section 2: Webhook Routes
- Header: "Webhook Routes" + [+ Add Route] button
- Table columns: Path | Tool | Input Template (truncated) | Created | (delete icon)
- Empty state: "No webhook routes configured."

## Modals

### Schedule Action modal
Fields:
- Label (text)
- Tool (select from `api.listTools()`)
- Tool Input (JSON textarea)
- Scheduled At (datetime-local input)
- Recurrence (optional: select None / every 1h / every 6h / every 24h)

### Add Webhook Route modal
Fields:
- Path (text, e.g. /github)
- Tool (select from `api.listTools()`)
- Input Template (textarea, e.g. {"payload": "{{payload}}"})

## What to defer
- SSE real-time status updates
- Webhook payload history/inspector
- Action editing (delete + recreate)
- Recurrence display beyond badge