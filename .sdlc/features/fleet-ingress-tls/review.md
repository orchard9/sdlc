# Review: Wildcard TLS cert for *.sdlc.threesix.ai via cert-manager DNS01

## Summary

All tasks completed. Wildcard TLS certificate for `*.sdlc.threesix.ai` is issued, ready, and stored in the `sdlc-tls` namespace. All manifests committed to `orchard9/sdlc-cluster/tls/`.

## Implementation vs Spec Divergence

**DNS provider correction:** The spec assumed GCP Cloud DNS, but the cluster's actual DNS infrastructure for `threesix.ai` is Cloudflare. This was discovered during implementation by inspecting the live `letsencrypt-prod` ClusterIssuer. The implementation correctly used Cloudflare (already configured, already working). The GCP DNS SA ExternalSecret was not needed — the Cloudflare API token already existed in `cert-manager`.

**No new ClusterIssuer created:** `letsencrypt-prod` already exists in the cluster with a Cloudflare DNS01 solver for `threesix.ai` (which covers `sdlc.threesix.ai` as a subdomain). The manifest was committed to the repo as IaC documentation, not as a new resource.

## Files Committed to orchard9/sdlc-cluster

| File | Purpose |
|---|---|
| `tls/namespace.yaml` | Creates `sdlc-tls` namespace with managed-by label |
| `tls/cluster-issuer.yaml` | Documents `letsencrypt-prod` ClusterIssuer (Cloudflare DNS01); applied as IaC reference |
| `tls/certificate.yaml` | Wildcard cert for `*.sdlc.threesix.ai` with reflector annotations |
| `tls/README.md` | Full operational runbook: prerequisites, apply order, verify, troubleshoot, renew |

## Live Cluster Verification

### Certificate Status
```
NAME                READY   SECRET              ISSUER             STATUS
sdlc-wildcard-tls   True    sdlc-wildcard-tls   letsencrypt-prod   Certificate is up to date and has not expired
```

### Certificate Contents
```
subject=CN=*.sdlc.threesix.ai
notBefore=Mar  3 02:37:22 2026 GMT
notAfter=Jun  1 02:37:21 2026 GMT
issuer=C=US, O=Let's Encrypt, CN=R13
renewalTime: 2026-05-02T02:37:21Z
```
Certificate is valid for 90 days, auto-renewal configured at 60 days (30 days before expiry).

### Issuance Method
DNS01 challenge via Cloudflare API completed successfully in ~45 seconds. Cloudflare TXT record for `_acme-challenge.sdlc.threesix.ai` was set and verified by Let's Encrypt.

## Code Review Findings

### Finding 1: reflector not yet installed — TRACKED
**Severity:** Low (deferred)
**Detail:** kubernetes-reflector is not currently installed in the cluster. The reflector annotations on `certificate.yaml` are correct and will activate once reflector is installed. The replication will work automatically as soon as reflector is deployed.
**Action:** Added to cluster bootstrap prerequisites. Reflector installation is a one-time cluster-ops step documented in `tls/README.md`. No sdlc-* namespaces exist yet so there is no immediate impact.

### Finding 2: Spec incorrectly stated GCP Cloud DNS — ADDRESSED IN-FLIGHT
**Severity:** Low (resolved)
**Detail:** The spec was written based on ponder session context that assumed GCP Cloud DNS. The live cluster uses Cloudflare for `threesix.ai`. The implementation correctly adapted to actual infrastructure.
**Action:** Design and implementation used Cloudflare throughout. The GCP DNS SA ExternalSecret manifest was omitted since it is not needed.

### Finding 3: Helm chart spec uses `gcp-secret-store` — NOT IN SCOPE
**Severity:** Low (tracked separately)
**Detail:** `fleet-helm-chart/spec.md` references `gcp-secret-store` as the ClusterSecretStore name, but the live store is named `gcp-secret-manager`. This inconsistency exists in fleet-helm-chart, not in this feature.
**Action:** Will be addressed when fleet-helm-chart reaches implementation.

## Acceptance Criteria Verification

| Criterion | Status |
|---|---|
| cert-manager pods running in cert-manager namespace | PASS — pre-existing, verified |
| ClusterIssuer `letsencrypt-prod` exists and Ready=True | PASS — pre-existing |
| Certificate for `*.sdlc.threesix.ai` reaches Ready=True | PASS — verified live |
| `sdlc-wildcard-tls` Secret present in `sdlc-tls` with valid TLS keypair | PASS — verified via openssl |
| Reflector annotations set on Certificate | PASS — applied to certificate.yaml |
| All manifests in `orchard9/sdlc-cluster` under `tls/` | PASS — 4 files committed |
| Certificate shows `Ready=True` in kubectl | PASS — verified |
| Browser TLS validation | DEFERRED — no sdlc-* project deployed yet |

## Status: READY TO MERGE

All core acceptance criteria met. The two deferred items (reflector installation, browser end-to-end test) require a deployed fleet project which will be created by `fleet-deploy-pipeline`. These are tracked and do not block merge.
