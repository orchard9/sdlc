# Build Outcome — v17 + v18

Captured 2026-03-03. All architecture open questions from `sdlc-cluster-architecture.md`
are now resolved. v17 (fleet-foundation) and v18 (fleet-automation) are both RELEASED.

## Open Questions — Resolved

| Question | Resolution |
|---|---|
| GCP project ID / GSM key path | `sdlc-fleet-gitea` created in GCP Secret Manager. ESO ClusterSecretStore wired. (`fleet-secrets-infra` — released) |
| Is sdlc.threesix.ai in GCP Cloud DNS? | Yes. cert-manager DNS01 ClusterIssuer + `*.sdlc.threesix.ai` wildcard cert via Let's Encrypt. (`fleet-ingress-tls` — released) |
| sdlc-server Docker image in Zot? | Using `ghcr.io/orchard9/sdlc-server:latest`. No Zot dependency. (`fleet-helm-chart` — released) |
| git-sync auth mechanism | GITSYNC_USERNAME / GITSYNC_PASSWORD from ExternalSecret (token injection). (`fleet-helm-chart` — released) |
| Bootstrap loop for 80+ projects | `bootstrap.py` (Python, Gitea pagination, Woodpecker API trigger) + `bootstrap.yaml` pipeline. (`fleet-bootstrap` — released) |

## Features Released

- `fleet-repo-scaffold` — orchard9/sdlc-cluster repo, directory structure, .woodpecker.yml
- `fleet-secrets-infra` — GCP Secret Manager + ESO ClusterSecretStore
- `fleet-helm-chart` — full Helm chart, 10/10 TC pass (helm lint + kubectl dry-run)
- `fleet-deploy-pipeline` — Woodpecker deploy-project.yaml
- `fleet-reconcile-pipeline` — Woodpecker scheduled reconcile
- `fleet-ingress-tls` — cert-manager + wildcard TLS
- `fleet-bootstrap` — Python bootstrap script + pipeline (TC-6/TC-7 deferred to milestone UAT)

## Remaining Gaps

1. **End-to-end not yet executed** — no project has been deployed via the full pipeline.
   Blocked on registering orchard9/sdlc-cluster in Woodpecker CI.
2. **Auth** — all pods are unauthenticated today. See `fleet-auth-design.md`.
3. **Minor hardening** — deploy-pipeline T1 (image digest pin), T2 (org prefix validation);
   ingress-tls T1 (zone-scoped Cloudflare token).

## Key Decision Revision

Session 4 declared "sdlc is FROZEN — zero Rust changes." This was correct for the
infrastructure work. It is no longer correct for auth. Auth requires targeted changes to
`crates/sdlc-server/src/auth.rs` and new routes. The Helm chart and Woodpecker pipelines
remain frozen; only the server binary needs auth additions.
