# Security Audit: Helm chart — sdlc-server + git-sync sidecar

## Scope

Infrastructure-as-code audit of the Helm chart at `deployments/helm/sdlc-server/` in the k3s-fleet repo. Focus: secret handling, credential exposure, privilege escalation, namespace isolation, and network exposure.

---

## Findings

### F1 — Credentials flow through ESO, not hard-coded — PASS

**Finding:** Gitea credentials (`url`, `token`, `user`) are fetched from GCP Secret Manager via the ESO ClusterSecretStore and materialized as a k8s Secret (`gitea-credentials`) in the project namespace. No credentials appear in values files, templates, or Chart.yaml.

**Action:** None required. This is correct practice.

---

### F2 — Credential access scoped to git-sync sidecar — PASS

**Finding:** The `GITSYNC_USERNAME` and `GITSYNC_PASSWORD` env vars are sourced from `gitea-credentials` only in the `git-sync` container. The `sdlc-server` container has no env vars or volume mounts referencing the Secret.

**Action:** None required. Least-privilege credential scoping is correctly applied.

---

### F3 — No root or privileged containers — PASS

**Finding:** Neither container specifies `securityContext.privileged: true` or `runAsUser: 0`. Both run as whatever user the container image defaults to. git-sync v4 runs as non-root by default (uid 65533). sdlc-server should also be non-root; this is an image-level control outside this chart's scope.

**Action:** None for this chart. Track as a follow-up: confirm sdlc-server image sets `USER` to non-root in the Dockerfile (`fleet-image-hardening` or similar).

---

### F4 — Namespace isolation per project — PASS

**Finding:** Each helm release creates a dedicated namespace `sdlc-<slug>`. All resources (Deployment, Service, Ingress, ExternalSecret, k8s Secret) are namespaced. This limits blast radius: a compromised project pod cannot directly access resources in other project namespaces.

**Action:** None required. Consider adding NetworkPolicy in a future iteration to enforce namespace-level network isolation.

---

### F5 — External access gated by TLS Ingress — PASS

**Finding:** The Service is ClusterIP (not NodePort or LoadBalancer). External access is exclusively through the Ingress, which enforces TLS with a pre-provisioned wildcard certificate. The SSE proxy-read-timeout annotation is purely functional, not a security concern.

**Action:** None required.

---

### F6 — ExternalSecret target uses `creationPolicy: Owner` — PASS

**Finding:** `creationPolicy: Owner` means the ExternalSecret CR owns the generated k8s Secret. If the ExternalSecret is deleted, the Secret is garbage collected. This prevents orphaned credential material.

**Action:** None required.

---

### F7 — emptyDir workspace has no size limit — LOW RISK

**Finding:** The shared `emptyDir` volume has no `sizeLimit`. A malformed or malicious repo with extremely large files could fill the node's ephemeral storage.

**Action:** Low risk in practice (orchard9 repos are small), but acceptable to add `sizeLimit: 1Gi` as a defensive measure. Adding as a tracked task rather than blocking.

**Decision:** Accept. Track as future hardening: `sdlc task add fleet-helm-chart "Add emptyDir sizeLimit to workspace volume"`. Not blocking release.

---

### F8 — git-sync pulls from internal Gitea NodePort (HTTP, not HTTPS) — ACCEPTED RISK

**Finding:** `GITSYNC_REPO` is constructed as `{{ .Values.gitea.url }}/{{ .Values.project.repo }}`, where `gitea.url` defaults to `http://100.79.2.8:30300` (plain HTTP). git-sync traffic between the pod and Gitea is unencrypted over the cluster/LAN.

**Risk Assessment:** Traffic flows within the private k3s cluster LAN (Tailscale-secured network). No external exposure. The Tailscale overlay encrypts node-to-node traffic at the network layer, so plaintext HTTP within the cluster is acceptable for this internal Gitea instance.

**Action:** Accept. If Gitea gets a valid TLS cert in the future, update `gitea.url` in the values file. Not blocking.

---

## Summary

| Finding | Severity | Disposition |
|---|---|---|
| F1 — Credentials in ESO, not hard-coded | — | PASS |
| F2 — Credentials scoped to git-sync only | — | PASS |
| F3 — No privileged containers | — | PASS (image-level follow-up noted) |
| F4 — Namespace isolation | — | PASS |
| F5 — TLS-gated external access | — | PASS |
| F6 — ExternalSecret creationPolicy Owner | — | PASS |
| F7 — emptyDir no sizeLimit | LOW | Accept, track as future hardening |
| F8 — HTTP to internal Gitea | LOW | Accept, Tailscale mitigates |

No blocking security issues. Chart is cleared for release.
