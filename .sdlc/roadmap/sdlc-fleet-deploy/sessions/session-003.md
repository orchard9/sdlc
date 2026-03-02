---
session: 3
timestamp: 2026-03-02T07:30:00Z
orientation:
  current: "sdlc-cluster defined as a new standalone project (orchard9/sdlc-cluster). Architecture clear: git-sync sidecar + Helm chart + Woodpecker pipelines + ESO/GSM for secrets. One pod per project, automated. sdlc Rust codebase unchanged."
  next: "Create orchard9/sdlc-cluster repo on threesix/gitea. Scaffold Helm chart (helm/sdlc-server/). Build Woodpecker deploy-project pipeline. Create sdlc-fleet-gitea secret in GCP Secret Manager. Wire ESO ClusterSecretStore."
  commit: "Architecture is fully shaped and all key decisions are made. Commit signal met — ready to build sdlc-cluster."
---

**jordan · Owner**
> 1. GSM
> 2. correct
> 3. woodpecker to deploy sdlc-cluster is fine
> 4. leave them. we need it to be fresh ideally
> 5. we need "SDLC" as it stands today to stay the same, we're building sdlc-cluster with this

---

## Session 3 — sdlc-cluster defined

This session starts with five clean answers to session 2's open questions. Let me unpack them.

**Question 1 was: "GCP Secret Manager entry for gitea token?"** → **GSM.** Jordan's cluster already bridges to GCP via ExternalSecrets Operator. The gitea credentials go into GSM → ESO pulls → k8s Secret → pods mount. No custom secret management.

**Question 2 (context: is the Phase 2 multi-tenant refactor the right path?)** → **correct.** Confirmed.

**Question 3 was: "Woodpecker CI integration?"** → **woodpecker to deploy sdlc-cluster is fine.** Woodpecker is the CD layer.

**Question 4 was: "Should we migrate Jordan's 145 existing repos to orchard9?"** → **leave them. we need it to be fresh ideally.** Jordan's personal repos stay under `jordan/`. `orchard9/` is a clean namespace — new projects only.

**Question 5 is the pivot:** → **we need "SDLC" as it stands today to stay the same, we're building sdlc-cluster with this.**

This changes everything from sessions 1-2.

---

### The Pivot: No refactor. New project.

The multi-tenant sdlc-server refactor — the Phase 2 work we were heading toward — is off the table. Jordan does not want to modify the sdlc Rust codebase to support fleet use cases. Instead: **sdlc-cluster is a new, separate project** that uses sdlc as a dependency/tool. It manages the deployment of sdlc-server instances at fleet scale.

---

### **Dan Reeves · Systems Minimalist**

Finally, a sane scope boundary. The instinct to "refactor sdlc-server to be multi-tenant" was creeping scope. You don't need to modify the tool to deploy many instances of it. Unix philosophy: build things that do one thing well, compose at the deployment layer.

sdlc-cluster is the composition layer. It answers: *how do you run N sdlc-server pods?*

What is sdlc-cluster, minimally? It's three things:

1. A **Helm chart** that deploys one sdlc-server instance for a given project: namespace, Deployment, Service, Ingress, ExternalSecret reference.
2. A **Woodpecker pipeline** that, given a project slug and Gitea repo URL, instantiates the Helm chart.
3. A **reconciliation job** that periodically scans the `orchard9` Gitea org and ensures every repo has a corresponding sdlc-server deployment.

That's it. No operator, no CRD, no custom controller — not yet. Helm + Woodpecker is the operator for 300 projects. You don't need Kubernetes to manage Kubernetes when Woodpecker is already doing it.

**? Open:** Is the reconciliation job a CronJob that calls `helm upgrade --install` idempotently? Or does it diff desired vs actual state and only act on deltas? The delta approach is smarter but adds complexity. CronJob that runs `helm upgrade --install` on every project every hour is dumb-but-correct.

---

### **Leila Hassan · Platform Engineering Lead**

I want to talk about the storage problem before anything else, because it blocks the Helm chart design.

sdlc-server runs in a project directory. It reads and writes `.sdlc/` YAML files. In the pod model: where does that directory come from?

**Option A: Longhorn PVC per project.** Each pod gets a persistent volume. The project repo is cloned there on first run. Gitea webhook triggers `git pull`.

*Problem:* 300 PVCs. That's 300 Longhorn replicas to manage. At 300, one bad replica event cascades. I've been here. Don't do this.

**Option B: EmptyDir + git-clone init container.** Clone the repo on pod start. State lives in the pod's ephemeral disk.

*Problem:* Pod restart = full re-clone + any uncommitted artifacts are gone. sdlc artifacts ARE committed to git (that's the design), so if the agent commits before crashing, you lose nothing. But what if the agent is mid-write when the pod restarts?

**Option C: Git-sync sidecar.** The *right answer* for this architecture. Two containers in the pod:
- **Main:** `sdlc-server` (unchanged binary, pointed at `/workspace/<slug>`)
- **Sidecar:** `registry.k8s.io/git-sync/git-sync:v4` — continuously watches the Gitea repo and syncs to a shared `emptyDir` volume

sdlc never touches git. git-sync handles pulls. Agents push to Gitea externally. When a push lands in Gitea: Gitea webhook → git-sync watches → syncs within seconds. sdlc-server reads from the same shared volume.

No PVCs. No Longhorn. No persistent state. sdlc-server is stateless from the cluster's perspective — the source of truth is the git repo in Gitea.

**⚑ Decided: git-sync sidecar pattern.** No Longhorn PVCs. EmptyDir shared volume. sdlc binary unchanged.

---

### **Priya Nair · Distributed Systems Architect**

I'm focused on the failure modes.

The git-sync sidecar is correct but surfaces a correctness issue: sdlc-server reads from disk. If git-sync pulls a new commit while sdlc-server is mid-request, does the read see partial state?

Looking at sdlc's io.rs: atomic writes. Good. Each artifact file is written atomically — either the new content is there or the old content is. git-sync uses atomic rename (`mv`) when updating files. So concurrent reads during a git-sync pull are safe — you'll see either the old or new file, never a torn write.

**⚑ Decided: git-sync atomic pull + sdlc atomic writes = safe concurrent access. No locking needed.**

GCP Secret Manager integration: Jordan's k3s fleet already has ExternalSecrets Operator (the session 2 notes reference "follow existing ExternalSecret pattern"). The pattern is:

```yaml
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: gitea-credentials
  namespace: sdlc-<slug>
spec:
  secretStoreRef:
    name: gcp-secret-store
    kind: ClusterSecretStore
  target:
    name: gitea-credentials
  data:
    - secretKey: GITEA_TOKEN
      remoteRef:
        key: sdlc-fleet-gitea
        property: token
    - secretKey: GITEA_URL
      remoteRef:
        key: sdlc-fleet-gitea
        property: url
```

Then the git-sync container mounts `gitea-credentials` and uses GITEA_TOKEN for authentication.

**? Open:** What is the GCP project and GSM path? Jordan needs to create `sdlc-fleet-gitea` in GSM with the token JSON. This is the first concrete infra action.

**? Open:** What happens when sdlc-server crashes mid-agent run? The agent is external (Claude Code or Woodpecker job), not in-process. If sdlc-server goes down, the agent loses its WebSocket/SSE connection. On pod restart, git-sync re-syncs and sdlc-server is back. The agent retries. This is fine — agents already handle disconnects.

---

### **Dan Reeves** (continuing)

Let me define the sdlc-cluster repo structure so we're not hand-waving:

```
orchard9/sdlc-cluster/
├── helm/
│   └── sdlc-server/           # Helm chart for one sdlc-server instance
│       ├── Chart.yaml
│       ├── values.yaml         # Defaults + schema
│       └── templates/
│           ├── namespace.yaml
│           ├── deployment.yaml # sdlc-server + git-sync sidecar
│           ├── service.yaml
│           ├── ingress.yaml    # <slug>.sdlc.threesix.ai
│           └── external-secret.yaml
├── pipelines/
│   ├── deploy-project.yaml    # Woodpecker: install one project by slug
│   └── reconcile-projects.yaml # Woodpecker: scan orchard9 org → ensure all have deployments
├── external-secrets/
│   └── cluster-secret-store.yaml  # ClusterSecretStore pointing at GSM
└── .woodpecker.yml            # sdlc-cluster's own CI (lints charts)
```

**Values.yaml shape** (this becomes the API for provisioning a project):

```yaml
project:
  slug: my-project
  repo: orchard9/my-project   # Gitea repo name under orchard9 org
  branch: main

gitea:
  url: http://100.79.2.8:30300
  externalSecret:
    secretStore: gcp-secret-store
    gsmKey: sdlc-fleet-gitea

ingress:
  domain: sdlc.threesix.ai     # → my-project.sdlc.threesix.ai
  tlsSecretName: sdlc-wildcard-tls
```

**Woodpecker `deploy-project.yaml` pipeline:**

```yaml
# Triggered with: SDLC_PROJECT_SLUG=my-project SDLC_REPO=orchard9/my-project
steps:
  deploy:
    image: alpine/helm:3.14
    commands:
      - helm upgrade --install
          sdlc-$SDLC_PROJECT_SLUG
          ./helm/sdlc-server
          --namespace sdlc-$SDLC_PROJECT_SLUG
          --create-namespace
          --set project.slug=$SDLC_PROJECT_SLUG
          --set project.repo=$SDLC_REPO
```

Idempotent. Run it 10 times, same result. This is the unit of fleet management.

---

### **Leila Hassan** (on routing)

Subdomain routing requires a wildcard TLS cert: `*.sdlc.threesix.ai`. cert-manager is already in the cluster. Use a DNS01 challenge for the wildcard cert.

One `Certificate` resource → cert-manager → wildcard cert for `*.sdlc.threesix.ai` → stored as `sdlc-wildcard-tls` in `cert-manager` namespace → each project Ingress references it via `secretName`.

Since this is Tailscale-only (not publicly reachable), a self-signed wildcard cert from a cluster-local CA is also viable — simpler, no ACME challenge needed.

**? Open:** Is `sdlc.threesix.ai` a domain Jordan controls on GCP Cloud DNS? If yes, DNS01 wildcard is straightforward. If it's a manually-managed DNS zone, cert-manager DNS01 integration needs checking.

---

### **Priya Nair** (on the reconcile pipeline)

The reconcile pipeline is important. Without it, sdlc-cluster drifts: projects created in Gitea but not yet deployed, projects deleted from Gitea but pods still running.

The reconcile job:
1. Call Gitea API: list all repos in orchard9 org
2. Call `kubectl get deployments -A -l sdlc-managed=true` to get currently deployed projects
3. Diff: repos without deployments → run deploy-project pipeline
4. Diff: deployments without repos → delete (or alert, safer)

**⚑ Decided: Woodpecker scheduled pipeline for reconciliation, not k8s CronJob.** Woodpecker pipelines are visible in the CI dashboard, retry-able, and logged. CronJobs are fire-and-forget.

---

### Synthesis: What sdlc-cluster IS

**sdlc-cluster** is an infrastructure management project that:
1. **Provisions** sdlc-server instances per project (one pod per project in `orchard9` Gitea org)
2. **Manages** the full lifecycle (deploy, update image version, teardown)
3. **Routes** via wildcard subdomain: `<slug>.sdlc.threesix.ai`
4. **Syncs** project state via git-sync sidecar (no sdlc changes needed)
5. **Secrets** via ExternalSecrets → GCP Secret Manager
6. **CD** via Woodpecker: on-demand project provisioning + scheduled reconciliation

sdlc itself is **unchanged**. sdlc-cluster is the ops wrapper around it.

---

### ⚑ Decisions Made This Session

1. **sdlc stays frozen.** No changes to the Rust codebase for fleet use cases. sdlc-cluster is a separate project.
2. **sdlc-cluster is the new repo:** `orchard9/sdlc-cluster` on threesix/gitea.
3. **git-sync sidecar pattern.** No Longhorn PVCs. EmptyDir shared volume. git-sync handles Gitea pulls.
4. **GCP Secret Manager for gitea credentials.** ESO ClusterSecretStore → GSM key `sdlc-fleet-gitea`.
5. **Woodpecker CI deploys sdlc-cluster.** deploy-project pipeline + reconcile scheduled pipeline.
6. **Jordan's 145 existing repos stay under `jordan/`.** `orchard9/` is fresh — new projects only.
7. **Subdomain routing.** `<slug>.sdlc.threesix.ai` via wildcard TLS cert.
8. **Reconcile via Woodpecker scheduled pipeline**, not k8s CronJob.

---

### ? Open Questions

1. **GCP project + GSM path.** What GCP project holds the `sdlc-fleet-gitea` secret? This needs to exist before ESO can pull it.
2. **Wildcard cert.** Is `sdlc.threesix.ai` a subdomain Jordan controls in GCP Cloud DNS? Or is this Tailscale-internal only (self-signed)?
3. **sdlc Docker image.** Is there already a `sdlc-server` Docker image being built and pushed to the Zot registry? If not, sdlc-cluster needs a build pipeline for it first.
4. **git-sync auth.** Token-in-URL (`http://user:token@host/repo`) or HTTP credential helper? Token-in-URL is simpler for MVP.
5. **Bootstrap.** How to provision the first 80 projects — a loop script triggering the Woodpecker deploy-project pipeline per project slug.

---

### Commit Signal Assessment

**Commit signal from session 1:** "When Jordan knows which use case he's actually solving and at what real scale."

→ We know. Interactive fleet dashboard + autonomous execution for 80 projects heading to 300. sdlc-cluster with Woodpecker + Helm + git-sync sidecar is the right answer.

**⚑ Commit signal is met. This is ready to build.**

The next action: create `orchard9/sdlc-cluster` repo on threesix/gitea and scaffold the Helm chart + Woodpecker pipelines via `/sdlc-ponder-commit sdlc-fleet-deploy`.
