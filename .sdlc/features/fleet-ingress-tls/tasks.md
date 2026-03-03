# Tasks: Wildcard TLS cert for *.sdlc.threesix.ai via cert-manager DNS01 in GCP Cloud DNS

## Task Breakdown

### T1 — Create `tls/namespace.yaml`

Create the `sdlc-tls` namespace manifest in the `orchard9/sdlc-cluster` repo.

```yaml
# tls/namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: sdlc-tls
  labels:
    app.kubernetes.io/managed-by: sdlc-cluster
```

Commit to `orchard9/sdlc-cluster` at path `tls/namespace.yaml`.

**Acceptance:** File exists at `tls/namespace.yaml` in the sdlc-cluster repo with correct labels.

---

### T2 — Create `tls/external-secret-gcp-dns-sa.yaml`

Create the ExternalSecret manifest that pulls the GCP DNS SA key from GCP Secret Manager into the `cert-manager` namespace.

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
    name: gcp-secret-store
    kind: ClusterSecretStore
  target:
    name: gcp-dns-sa-key
    creationPolicy: Owner
  data:
    - secretKey: key.json
      remoteRef:
        key: sdlc-fleet-gcp-dns-sa
```

**Acceptance:** File exists, references `gcp-secret-store` ClusterSecretStore and `sdlc-fleet-gcp-dns-sa` GCP Secret Manager key.

---

### T3 — Create `tls/cluster-issuer.yaml`

Create the cert-manager `ClusterIssuer` manifest using ACME DNS01 with GCP Cloud DNS.

The `<gcp-project-id>` placeholder must be replaced with the actual GCP project ID for the `sdlc.threesix.ai` Cloud DNS zone. Look up the project ID from the existing cluster context or GCP console.

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
            project: <gcp-project-id>
            serviceAccountSecretRef:
              name: gcp-dns-sa-key
              key: key.json
        selector:
          dnsZones:
            - sdlc.threesix.ai
```

**Acceptance:** File exists with correct ACME endpoint, email, and GCP Cloud DNS solver referencing `gcp-dns-sa-key`.

---

### T4 — Create `tls/certificate.yaml`

Create the cert-manager `Certificate` resource that requests the wildcard cert from `letsencrypt-prod` and stores it in `sdlc-tls/sdlc-wildcard-tls`. Include reflector annotations for cross-namespace auto-replication.

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
  duration: 2160h
  renewBefore: 720h
```

**Acceptance:** File exists with wildcard dnsNames, correct issuerRef, and all four reflector annotations.

---

### T5 — Create `tls/README.md`

Write the operational runbook for the TLS setup. Must include:

1. Prerequisites section listing:
   - cert-manager v1.14+ installed in the cluster
   - kubernetes-reflector installed in `kube-system`
   - ESO ClusterSecretStore `gcp-secret-store` exists (from fleet-secrets-infra)
   - GCP Secret Manager entry `sdlc-fleet-gcp-dns-sa` contains the DNS SA JSON key
2. Apply order with copy-paste commands:
   ```bash
   kubectl apply -f tls/namespace.yaml
   kubectl apply -f tls/external-secret-gcp-dns-sa.yaml
   kubectl apply -f tls/cluster-issuer.yaml
   kubectl apply -f tls/certificate.yaml
   ```
3. Verification commands:
   ```bash
   kubectl describe certificate sdlc-wildcard-tls -n sdlc-tls
   kubectl describe clusterissuer letsencrypt-prod
   kubectl get secret sdlc-wildcard-tls -n sdlc-tls
   ```
4. Troubleshooting DNS01 section (challenge debugging, TXT record verification)
5. Manual cert renewal procedure
6. Reflector verification: `kubectl get secret sdlc-wildcard-tls -n sdlc-<slug>`

**Acceptance:** README.md exists at `tls/README.md` with all six sections present.

---

### T6 — Commit manifests to orchard9/sdlc-cluster

Push all four manifests plus README to the `orchard9/sdlc-cluster` repo on threesix/gitea.

Commit message: `feat(tls): wildcard cert for *.sdlc.threesix.ai via cert-manager DNS01`

Files to commit:
- `tls/namespace.yaml`
- `tls/external-secret-gcp-dns-sa.yaml`
- `tls/cluster-issuer.yaml`
- `tls/certificate.yaml`
- `tls/README.md`

**Acceptance:** All five files exist at their paths in `orchard9/sdlc-cluster` on `http://100.79.2.8:30300` and are visible in the Gitea UI.

---

### T7 — Apply manifests to the k3s cluster and verify certificate issuance

Apply the manifests in order to the live cluster and verify the Certificate reaches `Ready=True`.

```bash
kubectl apply -f tls/namespace.yaml
kubectl apply -f tls/external-secret-gcp-dns-sa.yaml
# Wait for ESO to sync the secret
kubectl wait --for=condition=Ready externalsecret/gcp-dns-sa-key -n cert-manager --timeout=60s
kubectl apply -f tls/cluster-issuer.yaml
kubectl apply -f tls/certificate.yaml
# Wait for cert issuance (DNS01 can take 2-5 minutes)
kubectl wait --for=condition=Ready certificate/sdlc-wildcard-tls -n sdlc-tls --timeout=600s
```

Verify reflector synced the secret to at least one existing `sdlc-*` namespace if any exist.

**Acceptance:**
- `kubectl get certificate sdlc-wildcard-tls -n sdlc-tls` shows `READY=True`
- `kubectl get secret sdlc-wildcard-tls -n sdlc-tls` shows `tls.crt` and `tls.key` data keys
- No cert-manager controller errors in `kubectl logs -n cert-manager -l app=cert-manager`
