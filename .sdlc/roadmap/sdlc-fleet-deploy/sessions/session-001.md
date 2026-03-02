---
session: 1
timestamp: 2026-03-01T00:00:00Z
orientation:
  current: "Three-layer fleet architecture identified: interactive server (multi-tenant), job executor (autonomous), fleet reader (dashboard). Phase progression from 1-pod-per-project to fully distributed."
  next: "Decide which phase is actually needed now. If <20 projects, try Phase 1 (1 pod per project) and feel the pain. If interactive UI for many projects is needed, spike the multi-tenant sdlc-server refactor."
  commit: "When Jordan knows which use case he's actually solving (interactive UI vs autonomous execution vs ops dashboard) and at what real scale he needs it in the next 3 months."
---

## Session 1 — Multi-project sdlc fleet deployment

**Participants:** Jordan (user), Priya Nair (Distributed Systems Architect), Dan Reeves (Systems Minimalist), Leila Hassan (Platform Engineering Lead)

---

### Brief

Jordan wants to deploy sdlc against multiple project repos simultaneously in a k3s environment (~/Workspace/orchard9/k3s-fleet). He's thinking about Gitea + Longhorn for repo storage and exploring four rough shapes:
1. Docker per project
2. One Docker for many
3. sdlc-many-ui refactor
4. ???

Scaling question: how does each approach age from 10 → 100 → 1000 → 10k → 100k projects?

---

### Interrogating the brief

First interrogation: what does "running sdlc for a project" actually mean?

Three distinct things:
- **A) Read state** — dashboard showing where all projects are
- **B) Serve the UI** — interactive web UI per project
- **C) Run agents** — spawn AI agents to do work autonomously

These have very different scaling profiles and right answers.

---

### **Priya Nair · Distributed Systems Architect**

Surfaced the core architectural constraint: sdlc state is .sdlc/ YAML files in each project repo. Three runtime models emerge:

1. **Clone on demand** — sdlc-server clones repo to temp dir, reads state, serves, cleans up. Zero persistent storage. Works to 100k for reads. Breaks for writes (need to push back).
2. **Persistent checkout per project** — each project has a dir on shared volume. One server, N projects, N dirs. Works to 1k.
3. **One pod per project** — maximum isolation, maximum overhead. Don't do this past 50 projects.

Key question surfaced: does sdlc-server need to write back to git? Answer: YES — when agents run, they write artifacts and commit them. This kills the "clone on demand" model for the write path.

---

### **Dan Reeves · Systems Minimalist**

⚑ Decided: Separate the three use cases before picking an architecture.

Challenged the premise: what is the actual problem to solve TODAY? Identified three fundamentally different use cases:
- **Developer dashboard** (5-20 projects, one UI) → multi-project read view or project switcher
- **Autonomous CI** (agents drive work on many repos automatically) → k8s Job + CronJob, not a server
- **Ops visibility** (read-only status across 100-1000 projects) → Gitea API aggregator

Strong recommendation: Don't build Phase 2+ until Phase 1 hurts. "The fleet management system becomes the thing you maintain instead of the actual work."

---

### **Leila Hassan · Platform Engineering Lead**

Brought operational experience from Shopify managing 400+ services on k3s:

**10 projects:** One sdlc-server per project as Deployment + Service + Ingress. PVC on Longhorn. Gitea webhooks for pull-on-push. Annoying to manage by hand but workable.

**100 projects:** One-pod-per-project becomes painful (100 PVCs, 100 restart loops). Switch to multi-tenant server with project routing OR stateless git-clone-on-demand for reads + job model for writes.

**1,000 projects:** Need a Kubernetes operator watching a CRD (SdlcProject). Helm at 1k is death by YAML.

**10,000+:** You're aggregating state from git, not serving live servers for all of them. sdlc becomes a git-native data format. Build a reader, not a server.

**Sweet spot for k3s fleet today:** Single sdlc-server serving multiple projects from same process. Right for 10-100 projects without over-engineering.

---

### Architecture evaluation

#### Option A: 1 Docker per project
| Scale | Result |
|---|---|
| 10 | ✅ Works perfectly |
| 100 | ⚠️ 100 pods, 100 PVCs, 100 Ingress rules |
| 1000 | ❌ Longhorn groans, cascade restarts |
| 10k+ | ❌ Not viable |

Verdict: Right for ≤20 projects.

#### Option B: Multi-tenant sdlc-server
What's needed: parameterize all routes by project root (project-path prefix or ?project= query param). Single Deployment, Longhorn volume with one dir per project, git-pull daemon.

| Scale | Result |
|---|---|
| 10-100 | ✅ One pod, projects are dirs |
| 1000 | ⚠️ 1k git checkout dirs, need sync daemon |
| 10k+ | ⚠️ Memory + fd pressure, need cache eviction |

Effort: ~2-4 weeks of Rust. Verdict: Right evolution from single-project.

#### Option C: Fleet dashboard (Gitea API reader)
sdlc state is plain YAML in git repos → can read it without running sdlc-server at all. Just hit Gitea API, read .sdlc/state.yaml from each repo, render. Read-only — can't approve artifacts or trigger agents.

| Scale | Result |
|---|---|
| 10-100k | ✅ Gitea handles this. Rate-limit poller. |

Verdict: Right for ops-dashboard use case. Wrong for control plane.

#### Option D: k8s Job model (sdlc as worker, not server)
Gitea webhook → k8s Job → clone repo → sdlc next → execute directive → commit → exit. The CI/CD model.

| Scale | Result |
|---|---|
| 10-100k | ✅ Jobs are ephemeral, scales horizontally |

Verdict: Right for autonomous agent execution. Wrong for interactive UI. You lose the live UI entirely.

---

### **Priya Nair · Distributed Systems Architect** (synthesis)

⚑ Decided: Two independent problems have been mentally merged.

1. **Interactive control plane** — humans + agents interact with sdlc UI. Needs a server. Tops out ~1k-10k projects per instance.
2. **Autonomous execution** — agents drive feature work automatically. Job scheduler problem. Scales to 100k.

Right architecture separates these. They share git as source of truth.

? Open: Which half is actually needed in the next month?

---

### The layered target architecture

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

### Concrete phase progression

- **Phase 1 (now):** 1 sdlc-server per project as k3s Deployment. Longhorn PVC, Gitea webhook pull-on-push. Zero new code.
- **Phase 2 (at 20+ projects):** Multi-tenant sdlc-server. Parameterize all routes by project root. Single Deployment. ~2-4 weeks Rust.
- **Phase 3 (at 100+ projects):** Fleet reader dashboard. Gitea API polls .sdlc/state.yaml. Read-only aggregator.
- **Phase 4 (autonomous at scale):** k8s Job pattern for agent execution. Server becomes UI-only.

---

### Open questions remaining

? Open: What's the actual project count Jordan expects in the next 3 months? If <20, Phase 1 is correct and no new code is needed.

? Open: Is the goal interactive UI (approve artifacts, browse features) or autonomous execution (agents run without human involvement)?

? Open: Does Jordan's k3s fleet already have Gitea running, or does that need to be set up? The Gitea API reader (Phase 3) depends on a Gitea instance.

? Open: What's the agent execution model? Does sdlc-server today spawn Claude Code as a subprocess, or is agent execution external?

---

Recruited: Leila Hassan · Platform Engineering Lead (created .claude/agents/leila-hassan.md)
