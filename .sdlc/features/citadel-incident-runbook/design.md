# Design: Create Incident from Citadel Logs Runbook

## Overview

This is a backend/orchestration feature. The primary deliverable is a Pantheon runbook definition and its step execution logic in Go. No new UI is needed — Discord is the sole interaction surface; Pantheon's existing Discord bot and runbook engine handle rendering.

---

## System Architecture

```
Discord
  │
  ▼  "create incident from logs around <time> for <service>"
Pantheon Discord Bot (existing)
  │  pattern match → trigger RunbookExecutor
  ▼
RunbookExecutor (internal/runbooks/executor.go — existing)
  │  loads RunbookDefinition: "citadel_create_incident_from_logs"
  │  extracts params: {service, time, severity}
  │
  ├─ Step 1: citadel_query_logs ──────────────────────────────────────────►
  │    ToolExecutor → Citadel API GET /api/v1/query                         Citadel
  │    ← {logs[], metadata}                                                 ◄─────────
  │    (halt on failure or empty logs)
  │
  ├─ Step 2: citadel_annotate_log (×up to 5 entries) ────────────────────►
  │    ToolExecutor → Citadel API POST /api/v1/annotations                  Citadel
  │    ← {annotation_id, created_at}                                        ◄─────────
  │    (warn_continue on failure)
  │
  ├─ Step 3: CreateIncident ──────────────────────────────────────────────►
  │    IncidentService.Create() → Pantheon DB                               Pantheon DB
  │    ← {incident_id, url}                                                 ◄─────────
  │    (halt on failure)
  │
  └─ Step 3b: Update annotations (×annotations from Step 2) ─────────────►
       ToolExecutor → Citadel API POST /api/v1/annotations                  Citadel
       replaces "{PENDING}" with incident-<id>                              ◄─────────
       (warn_continue on failure)
  │
  ▼
Discord reply: "Incident created: [<title>](<url>) ..."
```

---

## Data Flow

### Context object passed between steps

The runbook engine maintains a `RunbookContext` struct that accumulates outputs across steps:

```go
type CitadelIncidentRunbookCtx struct {
    // Input params (from Discord trigger)
    Service  string
    Time     string
    Severity string

    // Step 1 output
    Logs          []CitadelLogEntry
    ExecutedQuery string
    LogCount      int

    // Step 2 output
    AnnotationIDs []string // collected as each annotation succeeds

    // Step 3 output
    IncidentID  string
    IncidentURL string
}
```

---

## Key Interfaces

### RunbookDefinition (registration)

```go
// internal/runbooks/citadel_incident.go

var CitadelCreateIncidentRunbook = RunbookDefinition{
    Name:           "citadel_create_incident_from_logs",
    DisplayName:    "Create Incident from Citadel Logs",
    TriggerPattern: "create incident from logs around {time} for {service}",
    RequiresApproval: false,
    Steps: []RunbookStep{
        {ID: "query_logs",          Fn: stepQueryLogs,         OnFailure: HaltOnFailure},
        {ID: "annotate",            Fn: stepAnnotateLogs,      OnFailure: WarnContinue},
        {ID: "create_incident",     Fn: stepCreateIncident,    OnFailure: HaltOnFailure},
        {ID: "update_annotations",  Fn: stepUpdateAnnotations, OnFailure: WarnContinue},
    },
}
```

### Step implementations

```go
func stepQueryLogs(ctx *CitadelIncidentRunbookCtx, deps RunbookDeps) error {
    query := fmt.Sprintf("service:%s @since:%s", ctx.Service, ctx.Time)
    result, err := deps.ToolExecutor.Run("citadel_query_logs", map[string]any{
        "query": query,
        "limit": 50,
    })
    if err != nil { return err }
    if len(result.Logs) == 0 {
        return fmt.Errorf("no logs found for %s around %s", ctx.Service, ctx.Time)
    }
    ctx.Logs = result.Logs
    ctx.ExecutedQuery = result.Metadata.ExecutedQuery
    ctx.LogCount = result.Metadata.Count
    return nil
}

func stepAnnotateLogs(ctx *CitadelIncidentRunbookCtx, deps RunbookDeps) error {
    keyEntries := selectKeyEntries(ctx.Logs, 5) // error-level first, then first+last
    for _, entry := range keyEntries {
        result, err := deps.ToolExecutor.Run("citadel_annotate_log", map[string]any{
            "log_id":          entry.ID,
            "content":         fmt.Sprintf("linked to incident-{PENDING} — triage for %s around %s", ctx.Service, ctx.Time),
            "annotation_type": "incident",
        })
        if err != nil {
            deps.Logger.Warn("annotation failed for log entry", "log_id", entry.ID, "err", err)
            continue
        }
        ctx.AnnotationIDs = append(ctx.AnnotationIDs, result.AnnotationID)
    }
    return nil // always non-fatal
}

func stepCreateIncident(ctx *CitadelIncidentRunbookCtx, deps RunbookDeps) error {
    summary := buildSummary(ctx) // see summary template below
    inc, err := deps.IncidentService.Create(domain.CreateIncidentParams{
        Title:            fmt.Sprintf("[Citadel] %s anomaly around %s", ctx.Service, ctx.Time),
        Severity:         domain.Severity(ctx.Severity),
        Status:           domain.StatusInvestigating,
        AlertSource:      domain.AlertSourceCitadel,
        Summary:          summary,
    })
    if err != nil { return err }
    ctx.IncidentID  = inc.ID
    ctx.IncidentURL = inc.URL
    return nil
}

func stepUpdateAnnotations(ctx *CitadelIncidentRunbookCtx, deps RunbookDeps) error {
    // Re-annotate each entry with the real incident ID replacing {PENDING}
    for i, annotationID := range ctx.AnnotationIDs {
        entry := selectKeyEntries(ctx.Logs, 5)[i]
        _, err := deps.ToolExecutor.Run("citadel_annotate_log", map[string]any{
            "log_id":          entry.ID,
            "content":         fmt.Sprintf("linked to incident-%s — triage for %s around %s", ctx.IncidentID, ctx.Service, ctx.Time),
            "annotation_type": "incident",
        })
        if err != nil {
            deps.Logger.Warn("annotation update failed", "annotation_id", annotationID, "err", err)
        }
    }
    return nil
}
```

---

## Summary Template

```go
func buildSummary(ctx *CitadelIncidentRunbookCtx) string {
    episodeID := "none"
    if len(ctx.Logs) > 0 && ctx.Logs[0].EpisodeID != "" {
        episodeID = ctx.Logs[0].EpisodeID
    }
    annotations := strings.Join(ctx.AnnotationIDs, ", ")
    if annotations == "" { annotations = "none" }

    return fmt.Sprintf(`## Citadel Log Summary

**Service:** %s
**Time window:** %s (±1h)
**Query:** `+"`%s`"+`
**Logs reviewed:** %d entries
**Annotations:** %s

### Key findings
%s

### Citadel Episode Link
%s`,
        ctx.Service, ctx.Time, ctx.ExecutedQuery,
        ctx.LogCount, annotations,
        synthesizeFindings(ctx.Logs), // top 3–5 error messages
        episodeID,
    )
}
```

---

## Entry Selection Logic

```go
// selectKeyEntries picks up to maxN log entries for annotation priority:
// 1. error and fatal level entries first
// 2. then warn level
// 3. then fill remaining slots with first + last entries
func selectKeyEntries(logs []CitadelLogEntry, maxN int) []CitadelLogEntry {
    var errors, warns, other []CitadelLogEntry
    for _, l := range logs {
        switch l.Level {
        case "error", "fatal": errors = append(errors, l)
        case "warn":           warns = append(warns, l)
        default:               other = append(other, l)
        }
    }
    result := append(errors, warns...)
    if len(result) < maxN && len(other) > 0 {
        result = append(result, other[0]) // first entry
        if len(other) > 1 {
            result = append(result, other[len(other)-1]) // last entry
        }
    }
    if len(result) > maxN {
        result = result[:maxN]
    }
    return result
}
```

---

## Discord Output Format

**Success:**
```
✅ Incident created: [Citadel] auth anomaly around 2026-03-03T10:00:00Z
   https://pantheon.orchard9.ai/incidents/inc_abc123

   Severity: high | Status: investigating
   Logs reviewed: 42 entries
   Annotated in Citadel: 5 entries
   Episode: ep_789
```

**No logs found:**
```
⚠️ No logs found for auth around 2026-03-03T10:00:00Z
   Try a different time anchor or check the service name.
```

**Partial failure (annotations failed, incident created):**
```
✅ Incident created: [Citadel] auth anomaly around 2026-03-03T10:00:00Z
   https://pantheon.orchard9.ai/incidents/inc_abc123

   Severity: high | Status: investigating
   Logs reviewed: 42 entries
   ⚠️ Citadel annotation failed (2 of 5) — entries not linked in Citadel
   Episode: ep_789
```

---

## File Layout

```
internal/runbooks/
  citadel_incident.go      — CitadelCreateIncidentRunbook definition + 4 step functions
  citadel_incident_test.go — unit tests for all steps and selectKeyEntries

internal/runbooks/
  executor.go              — existing; no changes required
  registry.go              — add CitadelCreateIncidentRunbook to init() registration
```

---

## Registration

```go
// internal/runbooks/registry.go
func init() {
    Register(CitadelCreateIncidentRunbook)
}
```

This makes the runbook discoverable via `GET /api/v1/runbooks` without any route changes.

---

## Error Handling Summary

| Step | Failure Mode | Behavior |
|---|---|---|
| Step 1: query_logs | API error | Halt, report to Discord |
| Step 1: query_logs | Empty result | Halt, "no logs found" message |
| Step 2: annotate | Individual entry fails | Log warning, continue to next entry |
| Step 2: annotate | All entries fail | Log warning, proceed to Step 3 |
| Step 3: create_incident | API/DB error | Halt, report error to Discord |
| Step 3b: update_annotations | Individual update fails | Log warning, continue |

---

## Testing Plan

- Unit tests for `selectKeyEntries` — correct priority ordering, max cap
- Unit tests for `buildSummary` — missing episode ID handled, empty annotation list
- Unit test for each step function using mock `ToolExecutor` and `IncidentService`
- Integration test: mock Citadel API + Pantheon DB — full runbook execution end-to-end
- Edge cases: 0 logs → halt; all annotations fail → incident still created; `{PENDING}` correctly replaced in Step 3b
