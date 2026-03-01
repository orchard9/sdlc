# Plan-Act Pattern

A two-phase agent workflow where Phase 1 proposes a plan and Phase 2 implements it.

## What It Is

```
POST /api/<domain>/plan  →  agent reads context, streams a structured plan
POST /api/<domain>/act   →  agent receives the plan, implements it, streams results
```

Both phases use `spawn_agent_run` for SSE streaming, run history, and lifecycle. The frontend captures the plan text via `AmaAnswerPanel`'s `onDone` callback, optionally lets the user adjust it, then passes it to the act phase.

## When to Use It

Use this pattern when an agent action benefits from user review before irreversible writes. The plan phase is read-only (or lightweight); the act phase makes changes.

Good fits:
- Creating something novel where schema/approach needs design (tool creation)
- Automated setup that depends on project-specific detection (quality gates)
- Multi-step transformations where the strategy should be visible before execution

Skip it when the action is deterministic and always correct (e.g. `sdlc tool run`).

## Rust Recipe

### 1. Add request structs

```rust
#[derive(serde::Deserialize)]
pub struct PlanFooRequest {
    pub name: String,
    pub requirements: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct ActFooRequest {
    pub name: String,
    pub plan: String,   // text from Phase 1, possibly user-adjusted
}
```

### 2. Add SSE completion variants (state.rs)

```rust
/// Foo plan agent completed.
FooPlanCompleted { name: String },
/// Foo act agent completed.
FooActCompleted { name: String },
```

### 3. Add handlers (runs.rs)

```rust
pub async fn plan_foo(
    State(app): State<AppState>,
    Json(body): Json<PlanFooRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    validate_slug(&body.name)?;
    let key = format!("foo-plan:{}", body.name);
    let prompt = /* read context, design approach */;
    let opts = sdlc_query_options(app.root.clone(), 15);
    let result = spawn_agent_run(
        key.clone(), prompt, opts, &app, "foo_plan", "Plan foo",
        Some(SseMessage::FooPlanCompleted { name: body.name.clone() }),
    ).await?;
    // inject run_key into response
}

pub async fn act_foo(
    State(app): State<AppState>,
    Json(body): Json<ActFooRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    validate_slug(&body.name)?;
    let key = format!("foo-act:{}", body.name);
    let prompt = format!("Use /skill-name to implement `{}`.\n\n## Plan\n\n{}", body.name, body.plan);
    let opts = sdlc_query_options(app.root.clone(), 25);
    let result = spawn_agent_run(
        key.clone(), prompt, opts, &app, "foo_act", "Build foo",
        Some(SseMessage::FooActCompleted { name: body.name.clone() }),
    ).await?;
    // inject run_key
}
```

### 4. Register routes (lib.rs)

```rust
// Register BEFORE any {name} wildcard routes
.route("/api/foo/plan", post(routes::runs::plan_foo))
.route("/api/foo/act",  post(routes::runs::act_foo))
```

## Frontend Recipe

### API client

```typescript
planFoo: (body: { name: string; requirements?: string }) =>
  request<{ status: string; run_id: string; run_key: string }>('/api/foo/plan', {
    method: 'POST', body: JSON.stringify(body),
  }),
actFoo: (body: { name: string; plan: string }) =>
  request<{ status: string; run_id: string; run_key: string }>('/api/foo/act', {
    method: 'POST', body: JSON.stringify(body),
  }),
```

### Modal state machine

```
form → planning → adjusting → acting → done
```

```tsx
type Step = 'form' | 'planning' | 'adjusting' | 'acting'

// form → planning: call planFoo(), store run_key
// planning: <AmaAnswerPanel runKey={planRunKey} onDone={text => { setPlanText(text); setStep('adjusting') }} />
// adjusting: show plan text + <textarea> for user adjustments + "Build It" button
// acting: call actFoo({ name, plan: planText + '\n\n' + adjustments }), store run_key
//         <AmaAnswerPanel runKey={actRunKey} onDone={() => { onCreated(); onClose() }} />
```

Use `AmaAnswerPanel`'s `onDone(finalText)` callback to capture plan output and advance state.

## Existing Usages

| Domain | Plan endpoint | Act endpoint | Plan skill | Act skill |
|--------|--------------|--------------|------------|-----------|
| Quality gates | `POST /api/tools/quality-check/reconfigure` | `POST /api/tools/quality-check/fix` | inline (detect languages + tooling) | `/fix-forward`, `/fix-all`, or `/remediate` |
| Tool creation | `POST /api/tools/plan` | `POST /api/tools/build` | inline (design schema + approach) | `/sdlc-tool-build` |
