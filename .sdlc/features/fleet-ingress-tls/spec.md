# Spec: Wildcard TLS cert for *.sdlc.threesix.ai via cert-manager DNS01 in GCP Cloud DNS

## Feature

**Slug:** fleet-ingress-tls
**Milestone:** v17-fleet-foundation
**Title:** Wildcard TLS cert for *.sdlc.threesix.ai via cert-manager DNS01 in GCP Cloud DNS

## Problem Statement

The fleet Helm chart (`fleet-helm-chart`) references a TLS secret named `sdlc-wildcard-tls` via:

```yaml
ingress:
  tlsSecretName: sdlc-wildcard-tls
```

This secret does not exist. Without it, every ingress resource in the fleet will fail to terminate TLS, leaving all `<slug>.sdlc.threesix.ai` subdomains either unreachable or served over plain HTTP.

A manually created certificate would require annual renewal and operator intervention. cert-manager with DNS01 challenge against GCP Cloud DNS provides automatic, renewable wildcard TLS with zero operator involvement after initial setup.

## Goals

1. Install cert-manager into the k3s cluster (if not already present).
2. Configure a `ClusterIssuer` using ACME (Let's Encrypt) with DNS01 challenge via GCP Cloud DNS.
3. Issue a wildcard `Certificate` for `*.sdlc.threesix.ai` that populates the `sdlc-wildcard-tls` Secret in the `cert-manager` namespace (or a shared `tls` namespace).
4. Make the wildcard TLS secret available to all `sdlc-<slug>` namespaces so each Helm-managed ingress can reference it.
5. Commit all manifests to `orchard9/sdlc-cluster` under `external-secrets/` or a new `tls/` directory.

## Out of Scope

- The Helm chart Ingress template itself (fleet-helm-chart)
- GCP Cloud DNS zone creation — the `sdlc.threesix.ai` zone is assumed to already exist
- cert-manager operator installation via Helm (only declarative manifests are in scope; one-time bootstrapping is documented, not automated)
- Issuing individual per-project certificates — the single wildcard covers all subdomains

## Architecture

```
Let's Encrypt (ACME)
        │
        │  DNS01 challenge: cert-manager sets _acme-challenge TXT record
        ▼
GCP Cloud DNS  ←──  cert-manager (ClusterIssuer)
                         │
                         │  issues
                         ▼
               Certificate: *.sdlc.threesix.ai
                         │
                         │  stored in
                         ▼
               Secret: sdlc-wildcard-tls  (namespace: sdlc-tls)
                         │
                         │  synced by
                         ▼
               Secret: sdlc-wildcard-tls  (namespace: sdlc-<slug>)  ← each project
```

cert-manager issues the certificate once and renews it automatically (60 days before expiry). A `ClusterSecret` sync mechanism (using either ESO or a simple `Secret` copy via `reflector`) copies the TLS secret into each project namespace so the Ingress can reference it locally.

## Acceptance Criteria

1. cert-manager pods are running in the `cert-manager` namespace.
2. A `ClusterIssuer` named `letsencrypt-prod` exists using ACME DNS01 via GCP Cloud DNS.
3. A `Certificate` resource for `*.sdlc.threesix.ai` exists and reaches `Ready=True`.
4. The `sdlc-wildcard-tls` Secret is present in the `sdlc-tls` namespace with a valid TLS keypair.
5. The wildcard secret is replicated to `sdlc-<slug>` namespaces so Helm ingresses can reference `sdlc-wildcard-tls` without modification.
6. All manifests live in the `orchard9/sdlc-cluster` repo under `tls/`.
7. `kubectl describe certificate sdlc-wildcard-tls -n sdlc-tls` shows `Status: True` for the `Ready` condition.
8. A browser navigating to any `https://<slug>.sdlc.threesix.ai` URL presents a valid, browser-trusted certificate.

## GCP Prerequisites

- A GCP Service Account with `dns.resourceRecordSets.create`, `dns.resourceRecordSets.delete`, `dns.resourceRecordSets.list`, `dns.changes.create`, `dns.changes.get` permissions on the `sdlc.threesix.ai` managed zone.
- The SA key is stored in GCP Secret Manager as `sdlc-fleet-gcp-dns-sa` and exposed to the cluster via an ESO ExternalSecret into `cert-manager` namespace as `gcp-dns-sa-key`.
- The GCP project ID for the Cloud DNS zone must be known at manifest-authoring time.

## Manifest Layout

```
orchard9/sdlc-cluster/
└── tls/
    ├── README.md                        # Documents the TLS strategy
    ├── namespace.yaml                   # Creates sdlc-tls namespace
    ├── external-secret-gcp-dns-sa.yaml  # ESO: pulls gcp-dns-sa-key into cert-manager ns
    ├── cluster-issuer.yaml              # ClusterIssuer: letsencrypt-prod (DNS01/GCP)
    ├── certificate.yaml                 # Certificate: *.sdlc.threesix.ai
    └── reflector-config.yaml            # Annotates secret for cross-namespace replication
```

## Secret Replication Strategy

Use [kubernetes-reflector](https://github.com/emberstack/kubernetes-reflector) to replicate `sdlc-wildcard-tls` from `sdlc-tls` to all namespaces matching `sdlc-*`. This requires:
- `reflector` deployed in the cluster (one-time setup, not in scope — documented).
- Annotations on the `Certificate` resource or the resulting `Secret`:
  ```yaml
  reflector.v1.k8s.emberstack.com/reflection-allowed: "true"
  reflector.v1.k8s.emberstack.com/reflection-allowed-namespaces: "sdlc-.*"
  reflector.v1.k8s.emberstack.com/reflection-auto-enabled: "true"
  reflector.v1.k8s.emberstack.com/reflection-auto-namespaces: "sdlc-.*"
  ```

## Dependencies

- `fleet-repo-scaffold` — the `orchard9/sdlc-cluster` repo must exist to receive the manifests
- `fleet-secrets-infra` — the ESO ClusterSecretStore must exist to resolve the GCP DNS SA key ExternalSecret
