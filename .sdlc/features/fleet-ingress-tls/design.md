# Design: Wildcard TLS cert for *.sdlc.threesix.ai via cert-manager DNS01 in GCP Cloud DNS

## Overview

This document describes the complete design for issuing and distributing an auto-renewing wildcard TLS certificate for `*.sdlc.threesix.ai` across the k3s cluster. The design covers cert-manager configuration, ACME DNS01 challenge setup, GCP credentials injection, certificate resource definition, and cross-namespace secret replication.

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│  k3s Cluster                                                        │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │  cert-manager namespace                                      │   │
│  │                                                              │   │
│  │  ClusterIssuer: letsencrypt-prod                             │   │
│  │    solver: dns01 / GCP Cloud DNS                             │   │
│  │    credentials: Secret gcp-dns-sa-key (JSON SA key)         │   │
│  │                                                              │   │
│  │  ExternalSecret → Secret: gcp-dns-sa-key                    │   │
│  │    (pulled from GCP Secret Manager via ESO)                  │   │
│  └──────────────────────────────┬───────────────────────────────┘  │
│                                 │ issues                            │
│  ┌──────────────────────────────▼───────────────────────────────┐  │
│  │  sdlc-tls namespace                                           │  │
│  │                                                              │   │
│  │  Certificate: sdlc-wildcard-tls                              │   │
│  │    dnsNames: ["*.sdlc.threesix.ai"]                          │   │
│  │    issuerRef: letsencrypt-prod                               │   │
│  │    secretName: sdlc-wildcard-tls                             │   │
│  │    annotations: reflector auto-enabled                       │   │
│  │                                                              │   │
│  │  Secret: sdlc-wildcard-tls  (tls.crt + tls.key)             │   │
│  └──────────────────────────────┬───────────────────────────────┘  │
│                                 │ reflector replicates              │
│  ┌──────────────────────────────▼───────────────────────────────┐  │
│  │  sdlc-my-project namespace                                   │  │
│  │  sdlc-other-project namespace                                │  │
│  │  sdlc-* namespaces ...                                       │  │
│  │                                                              │   │
│  │  Secret: sdlc-wildcard-tls  (copy)                           │   │
│  │    ↑ referenced by Ingress tlsSecretName                     │   │
│  └──────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
          │
          │  DNS01 challenge
          ▼
GCP Cloud DNS  ←──  cert-manager sets _acme-challenge.sdlc.threesix.ai TXT
          │
          ▼
Let's Encrypt ACME API  ──→  issues signed wildcard cert
```

## Component Design

### 1. cert-manager Installation

cert-manager is installed via its upstream Helm chart or static manifest. This is a one-time cluster bootstrapping step.

```bash
# One-time bootstrap (not a manifest — run once by cluster admin)
helm repo add jetstack https://charts.jetstack.io
helm upgrade --install cert-manager jetstack/cert-manager \
  --namespace cert-manager --create-namespace \
  --version v1.14.x \
  --set installCRDs=true
```

The manifests in `tls/` assume cert-manager CRDs and controller are already present.

### 2. GCP Service Account Credentials

cert-manager's DNS01 solver for GCP Cloud DNS requires a GCP Service Account JSON key.

**GCP-side setup (one-time, documented not automated):**
- SA: `cert-manager-dns01@<gcp-project>.iam.gserviceaccount.com`
- IAM role: `roles/dns.admin` scoped to the `sdlc-threesix-ai` managed zone
- SA key exported as JSON, stored in GCP Secret Manager as `sdlc-fleet-gcp-dns-sa`

**Cluster-side: ExternalSecret in `cert-manager` namespace**

```yaml
# tls/external-secret-gcp-dns-sa.yaml
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: gcp-dns-sa-key
  namespace: cert-manager
spec:
  refreshInterval: 1h
  secretStoreRef:
    name: gcp-secret-store      # ClusterSecretStore from fleet-secrets-infra
    kind: ClusterSecretStore
  target:
    name: gcp-dns-sa-key
    creationPolicy: Owner
  data:
    - secretKey: key.json
      remoteRef:
        key: sdlc-fleet-gcp-dns-sa
```

The resulting Secret `gcp-dns-sa-key` in `cert-manager` namespace holds `key.json` with the full GCP SA JSON.

### 3. ClusterIssuer

```yaml
# tls/cluster-issuer.yaml
apiVersion: cert-manager.io/v1
kind: ClusterIssuer
metadata:
  name: letsencrypt-prod
spec:
  acme:
    server: https://acme-v02.api.letsencrypt.org/directory
    email: ops@threesix.ai
    privateKeySecretRef:
      name: letsencrypt-prod-account-key
    solvers:
      - dns01:
          cloudDNS:
            project: <gcp-project-id>           # substituted at apply time
            serviceAccountSecretRef:
              name: gcp-dns-sa-key
              key: key.json
        selector:
          dnsZones:
            - sdlc.threesix.ai
```

The `ClusterIssuer` is cluster-scoped so it can issue certificates for any namespace.

**GCP project ID:** The `project` field must contain the actual GCP project ID where `sdlc.threesix.ai` is hosted. This is substituted during `kubectl apply` using a Kustomize patch or environment-specific values file.

### 4. sdlc-tls Namespace

A dedicated namespace isolates the wildcard certificate from project namespaces and from `cert-manager` itself.

```yaml
# tls/namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: sdlc-tls
  labels:
    app.kubernetes.io/managed-by: sdlc-cluster
```

### 5. Certificate Resource

```yaml
# tls/certificate.yaml
apiVersion: cert-manager.io/v1
kind: Certificate
metadata:
  name: sdlc-wildcard-tls
  namespace: sdlc-tls
  annotations:
    reflector.v1.k8s.emberstack.com/reflection-allowed: "true"
    reflector.v1.k8s.emberstack.com/reflection-allowed-namespaces: "sdlc-.*"
    reflector.v1.k8s.emberstack.com/reflection-auto-enabled: "true"
    reflector.v1.k8s.emberstack.com/reflection-auto-namespaces: "sdlc-.*"
spec:
  secretName: sdlc-wildcard-tls
  issuerRef:
    name: letsencrypt-prod
    kind: ClusterIssuer
  commonName: "*.sdlc.threesix.ai"
  dnsNames:
    - "*.sdlc.threesix.ai"
  duration: 2160h    # 90 days (Let's Encrypt max)
  renewBefore: 720h  # renew 30 days before expiry
```

The resulting `sdlc-wildcard-tls` Secret in `sdlc-tls` will contain:
- `tls.crt` — the signed wildcard certificate chain
- `tls.key` — the private key

### 6. Cross-Namespace Secret Replication (kubernetes-reflector)

kubernetes-reflector watches for Secrets annotated with `reflector.v1.k8s.emberstack.com/reflection-auto-enabled: "true"` and copies them to matching namespaces.

**Installation (one-time bootstrap):**
```bash
helm repo add emberstack https://emberstack.github.io/helm-charts
helm upgrade --install reflector emberstack/reflector \
  --namespace kube-system
```

**How it works with the fleet:**
- When a new `sdlc-<slug>` namespace is created by the Helm chart, reflector automatically creates a copy of `sdlc-wildcard-tls` in that namespace.
- When cert-manager renews the certificate (every ~60 days), reflector propagates the updated Secret to all matching namespaces within minutes.
- No manual intervention required after initial setup.

### 7. Apply Order

Manifests must be applied in this order (enforced by documentation and CI):

1. `tls/namespace.yaml`
2. `tls/external-secret-gcp-dns-sa.yaml` (requires ESO ClusterSecretStore from fleet-secrets-infra)
3. `tls/cluster-issuer.yaml` (requires cert-manager CRDs and gcp-dns-sa-key Secret)
4. `tls/certificate.yaml` (requires letsencrypt-prod ClusterIssuer)

The `tls/README.md` documents this apply order with copy-paste commands.

## Directory Structure in orchard9/sdlc-cluster

```
orchard9/sdlc-cluster/
└── tls/
    ├── README.md                          # Apply order, prerequisites, troubleshooting
    ├── namespace.yaml                     # Creates sdlc-tls namespace
    ├── external-secret-gcp-dns-sa.yaml    # ESO: gcp-dns-sa-key into cert-manager ns
    ├── cluster-issuer.yaml                # ClusterIssuer: letsencrypt-prod (DNS01/GCP)
    └── certificate.yaml                   # Certificate: *.sdlc.threesix.ai → sdlc-tls/sdlc-wildcard-tls
```

No `reflector-config.yaml` is needed as a separate file — the replication annotations are embedded directly in `certificate.yaml`.

## Operational Runbook (in README.md)

The `tls/README.md` will document:

1. **Prerequisites** — cert-manager installed, reflector installed, ESO ClusterSecretStore exists, GCP SA key in Secret Manager
2. **Apply commands** — `kubectl apply -f tls/` with correct order note
3. **Verify issuance** — `kubectl describe certificate sdlc-wildcard-tls -n sdlc-tls`
4. **Troubleshoot DNS01** — `kubectl describe challenge -n sdlc-tls` and checking Cloud DNS for TXT record
5. **Manual renewal** — `kubectl delete secret sdlc-wildcard-tls -n sdlc-tls` (cert-manager re-issues)
6. **Reflector verification** — `kubectl get secret sdlc-wildcard-tls -n sdlc-<slug>`

## Decisions

| Decision | Rationale |
|---|---|
| Wildcard cert instead of per-project | 80+ subdomains would hit Let's Encrypt rate limits; wildcard is 1 cert |
| DNS01 over HTTP01 | Wildcard certs require DNS01; HTTP01 cannot prove wildcard ownership |
| GCP Cloud DNS | The `sdlc.threesix.ai` zone is already in GCP Cloud DNS |
| kubernetes-reflector | Simpler than ESO secret replication for TLS secrets; auto-syncs on cert renewal |
| sdlc-tls namespace | Isolates cert lifecycle from cert-manager internals and project namespaces |
| cert-manager + reflector as prerequisites | Both are one-time cluster ops; encoding them in fleet manifests would require bootstrapping Helm-in-Helm which is unnecessary complexity |
