---
session: 4
timestamp: 2026-03-02T09:00:00Z
orientation:
  current: "Commit signal received. Status moving to converging. Architecture fully decided across 3 sessions. sdlc-cluster is ready to build."
  next: "Run /sdlc-ponder-commit sdlc-fleet-deploy to crystallize into milestones and features."
  commit: "Met. jordan said 'commit it'."
---

**jordan · Owner**
commit it

---

## Session 4 — Green light

Three sessions of architecture work. One signal: **commit it.**

No open questions are blocking the build. The architecture is locked. The remaining open questions (GCP project ID, wildcard cert DNS, Docker image in Zot, git-sync auth mechanism, bootstrap loop) are **build-time task-level decisions**, not architecture decisions. They get resolved in the first implementation sprint.

---

### **Dan Reeves · Systems Minimalist**

Good. We have everything we need and nothing we don't. Let me state what's decided so it's unambiguous for whoever picks this up:

1. **sdlc is frozen.** Zero changes to the Rust codebase.
2. **sdlc-cluster is a new repo:** `orchard9/sdlc-cluster` on threesix/gitea at `http://100.79.2.8:30300`.
3. **One pod per project.** Namespace `sdlc-<slug>`, subdomain `<slug>.sdlc.threesix.ai`.
4. **git-sync sidecar** handles repo pull. No Longhorn PVCs. emptyDir shared volume.
5. **GCP Secret Manager** holds `sdlc-fleet-gitea`. ESO ClusterSecretStore pulls it.
6. **Woodpecker** is the CD system: `deploy-project.yaml` (on-demand) + `reconcile-projects.yaml` (scheduled).
7. **orchard9/ org** is the clean namespace — only sdlc-managed projects land here.

The remaining open questions are standard first-sprint work. They don't block commitment.

---

### **Leila Hassan · Platform Engineering Lead**

Agreed. The remaining open questions resolve themselves once the repo is scaffolded:

- **GSM path:** Check `k3s-fleet/` for existing ClusterSecretStore config — the GCP project ID is already wired. Create `sdlc-fleet-gitea` in the same project.
- **Wildcard cert:** Start with self-signed cluster CA (simpler); swap to cert-manager DNS01 when the domain is confirmed.
- **Docker image:** Check Zot registry first. If no `sdlc-server` image exists, add a build-and-push pipeline to sdlc-cluster as the first task.
- **git-sync auth:** Token-in-URL (`http://claude-agent:<token>@100.79.2.8:30300/orchard9/<slug>`) for MVP — simplest.
- **Bootstrap:** A loop script that calls the Woodpecker deploy-project API per project slug. Best-effort loop + scheduled reconcile = self-healing.

None of these are design decisions. They're implementation details.

---

### **Priya Nair · Distributed Systems Architect**

One thing to state clearly before we close: the reconcile pipeline is the safety net. Even if the bootstrap loop is imperfect, the scheduled reconcile will catch the gaps and converge to the correct state. Don't over-engineer the bootstrap.

The failure modes are all recoverable:
- Pod crashes → restart policy brings it back, git-sync re-syncs
- Gitea goes down → pods serve stale state from emptyDir (sdlc-server doesn't block on Gitea for reads)
- GSM unavailable → pod fails to start (ExternalSecret sync fails) → alert, fix creds, pod comes up
- Woodpecker pipeline fails → reconcile runs again on schedule, retries

This architecture is sound. Ship it.

---

### ⚑ Final Decisions (consolidated from 4 sessions)

1. **sdlc is frozen.** No Rust changes.
2. **New repo:** `orchard9/sdlc-cluster` on threesix/gitea.
3. **git-sync sidecar + emptyDir.** No PVCs.
4. **GCP Secret Manager** via ESO ClusterSecretStore.
5. **Woodpecker:** deploy-project (on-demand) + reconcile (scheduled).
6. **orchard9/ org** = clean sdlc namespace.
7. **Wildcard routing:** `<slug>.sdlc.threesix.ai`.
8. **git-sync + sdlc atomic writes = safe concurrent access. No locking needed.**
9. **Woodpecker scheduled reconcile pipeline** (not k8s CronJob).
10. **Jordan's 145 repos stay under jordan/.** orchard9/ is fresh.

---

### Status: converging

⚑ **Decided: Status → converging. Next: `/sdlc-ponder-commit sdlc-fleet-deploy`.**
