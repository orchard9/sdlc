## Fleet Architecture Options

### The Core Constraint
sdlc state is .sdlc/ YAML files in each project repo. Runtime needs read-write access (agents commit back). This drives everything.

### Use Case Separation (Dan Reeves)
Three distinct use cases with different right answers:
1. **Interactive control plane** — humans + agents interact with UI. Needs a server. Tops out ~1k-10k projects per instance.
2. **Autonomous execution** — agents drive features automatically. This is a job scheduler problem. Scales to 100k.
3. **Ops visibility** — read-only status across many projects. Gitea API poll. Scales to 100k.

### Architecture by Scale

| Scale | Right Pattern |
|---|---|
| 1-20 projects | 1 sdlc-server per project (k3s Deployment + Longhorn PVC) |
| 20-1000 projects | Multi-tenant sdlc-server (project-parameterized routes) |
| 1000+ read | Fleet dashboard — Gitea API reader of .sdlc/state.yaml |
| 10k+ execution | k8s Job pattern: webhook → clone → sdlc next → commit → exit |
| 100k | Job executor + Gitea-API reader. No persistent per-project server. |

### The Layered Target Architecture

```
┌─────────────────────────────────────────────────────┐
│                   GITEA (state store)                │
│    Each repo contains .sdlc/ — source of truth       │
└───────────────────────────┬─────────────────────────┘
                            │
        ┌───────────────────┼───────────────────┐
        │                   │                   │
        ▼                   ▼                   ▼
┌───────────────┐  ┌────────────────┐  ┌────────────────┐
│  sdlc-server  │  │  Job Executor  │  │  Fleet Reader  │
│  (multi-proj) │  │  (k8s Job per  │  │  (dashboard,   │
│  interactive  │  │   directive)   │  │   read-only)   │
│  UI + write   │  │  autonomous    │  │  Gitea API     │
└───────────────┘  └────────────────┘  └────────────────┘
   10-1k projs         1k-100k             1k-100k
```

### Phase Progression

1. **Phase 1 (now)**: 1 sdlc-server per project. Longhorn PVC, Gitea checkout, webhook pull-on-push. Zero new code.
2. **Phase 2 (at 20+ projects)**: Multi-tenant sdlc-server. Parameterize all routes by project root. Single Deployment. ~2-4 weeks Rust.
3. **Phase 3 (at 100+ projects)**: Fleet reader dashboard. Gitea API polls .sdlc/state.yaml. Read-only aggregator. No sdlc-server required for reads.
4. **Phase 4 (autonomous at scale)**: k8s Job pattern for agent execution. Webhook triggers job. Commit back. Server becomes UI-only.

### Key Insights
- The multi-tenant refactor (Phase 2) is the most leveraged investment if you want interactive UI for many projects.
- The job pattern (Phase 4) is the only path to 100k autonomous execution.
- Don't build Phase 2+ until Phase 1 hurts. Feel the friction first (Dan Reeves).
- 100k 'live sdlc servers' is almost certainly the wrong question — at that scale you aggregate state from git, not serve it.