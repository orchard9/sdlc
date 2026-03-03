# Design: Helm chart — sdlc-server + git-sync sidecar

## Overview

A Helm chart at `deployments/helm/sdlc-server/` inside the `orchard9/sdlc-cluster` repo (the k3s-fleet repo). The chart follows the same structure and conventions as the existing `citadel` chart in that repo: `Chart.yaml`, `values.yaml`, and a `templates/` directory with one file per Kubernetes resource.

Each invocation of `helm upgrade --install` installs one self-contained deployment unit for a single orchard9 project: a namespace, a Deployment (sdlc-server + git-sync sidecar), a Service, an Ingress, and an ExternalSecret.

---

## Target Repository

```
/Users/jordanwashburn/Workspace/orchard9/k3s-fleet/
  deployments/
    helm/
      sdlc-server/          ← new chart directory
        Chart.yaml
        values.yaml
        templates/
          _helpers.tpl
          namespace.yaml
          external-secret.yaml
          deployment.yaml
          service.yaml
          ingress.yaml
```

The chart lives alongside the existing `citadel` chart — same parent directory, same structural conventions.

---

## Values API

```yaml
project:
  slug: my-project          # Kubernetes name-safe identifier — namespace + subdomain prefix
  repo: orchard9/my-project # Gitea org/repo path
  branch: main              # Branch git-sync tracks

gitea:
  url: http://100.79.2.8:30300
  externalSecret:
    secretStore: gcp-secret-store   # Name of ESO ClusterSecretStore
    gsmKey: sdlc-fleet-gitea        # GCP Secret Manager key

ingress:
  domain: sdlc.threesix.ai         # Wildcard base domain — host becomes <slug>.<domain>
  tlsSecretName: sdlc-wildcard-tls

image:
  server: ghcr.io/orchard9/sdlc-server:latest
  gitSync: registry.k8s.io/git-sync/git-sync:v4

resources:
  server:
    requests:
      cpu: 50m
      memory: 64Mi
    limits:
      cpu: 500m
      memory: 256Mi
  gitSync:
    requests:
      cpu: 10m
      memory: 32Mi
    limits:
      cpu: 100m
      memory: 64Mi
```

---

## Template Breakdown

### `_helpers.tpl`

Defines two named templates used across all resources:

- `sdlc-server.fullname` → `sdlc-{{ .Values.project.slug }}`
- `sdlc-server.namespace` → `sdlc-{{ .Values.project.slug }}`
- `sdlc-server.labels` → standard `app.kubernetes.io/` label set

### `namespace.yaml`

```
Kind: Namespace
Name: sdlc-{{ slug }}
Labels:
  app.kubernetes.io/managed-by: sdlc-cluster
  sdlc/project: {{ slug }}
```

### `external-secret.yaml`

```
Kind: ExternalSecret (ESO)
Namespace: sdlc-{{ slug }}
Spec:
  refreshInterval: 1h
  secretStoreRef:
    kind: ClusterSecretStore
    name: {{ gitea.externalSecret.secretStore }}
  target:
    name: gitea-credentials
    creationPolicy: Owner
  data:
    - secretKey: url     → remoteRef: key={{ gsmKey }}, property=url
    - secretKey: token   → remoteRef: key={{ gsmKey }}, property=token
    - secretKey: user    → remoteRef: key={{ gsmKey }}, property=user
```

### `deployment.yaml`

```
Kind: Deployment
Namespace: sdlc-{{ slug }}
Selector/Labels: app=sdlc-server, slug={{ slug }}
Spec:
  replicas: 1
  strategy: Recreate (single-writer safety)

  volumes:
    - name: workspace
      emptyDir: {}

  containers:
    [0] sdlc-server
      image: {{ image.server }}
      ports: [8080]
      volumeMounts: [workspace → /workspace]
      env:
        SDLC_DIR: /workspace/{{ slug }}/.sdlc
      livenessProbe:  HTTP GET /api/health :8080, delay=10s, period=15s
      readinessProbe: HTTP GET /api/health :8080, delay=5s,  period=10s
      resources: {{ resources.server }}

    [1] git-sync
      image: {{ image.gitSync }}
      volumeMounts: [workspace → /workspace]
      env:
        GITSYNC_REPO:    {{ gitea.url }}/{{ project.repo }}
        GITSYNC_ROOT:    /workspace
        GITSYNC_LINK:    {{ project.slug }}
        GITSYNC_BRANCH:  {{ project.branch }}
        GITSYNC_PERIOD:  30s
        GITSYNC_USERNAME: valueFrom secretKeyRef gitea-credentials user
        GITSYNC_PASSWORD: valueFrom secretKeyRef gitea-credentials token
      resources: {{ resources.gitSync }}
```

### `service.yaml`

```
Kind: Service (ClusterIP)
Namespace: sdlc-{{ slug }}
Selector: app=sdlc-server, slug={{ slug }}
Ports: 80 → 8080
```

### `ingress.yaml`

```
Kind: Ingress
Namespace: sdlc-{{ slug }}
Annotations:
  nginx.ingress.kubernetes.io/proxy-read-timeout: "3600"
Spec:
  tls:
    - hosts: [{{ slug }}.{{ ingress.domain }}]
      secretName: {{ ingress.tlsSecretName }}
  rules:
    - host: {{ slug }}.{{ ingress.domain }}
      backend: service port 80
```

---

## Deployment Model

Each project gets its own isolated namespace (`sdlc-<slug>`). This provides:
- Network isolation (namespace-scoped NetworkPolicy can be added later)
- Independent RBAC boundaries
- Clean `helm list -n sdlc-<slug>` per-project observability
- Simple teardown: `kubectl delete namespace sdlc-<slug>` removes all resources

git-sync runs as a sidecar sharing an `emptyDir` with sdlc-server. On first sync, git-sync clones the repo; on subsequent syncs (every 30s) it fetches updates. sdlc-server's `SDLC_DIR` points into the cloned tree, so it reads live feature state from the git worktree.

---

## Credential Flow

```
GCP Secret Manager
  key: sdlc-fleet-gitea
  fields: { url, token, user }
    ↓  ESO ClusterSecretStore (gcp-secret-store)
    ↓  ExternalSecret (sdlc-<slug> namespace)
    ↓  k8s Secret: gitea-credentials
    ↓  git-sync env: GITSYNC_USERNAME / GITSYNC_PASSWORD
```

The sdlc-server container does not receive Gitea credentials — it only reads the on-disk clone. Credentials are scoped to the sidecar.

---

## Acceptance Test

```bash
# Lint
helm lint deployments/helm/sdlc-server/

# Template smoke test
helm template sdlc-test deployments/helm/sdlc-server/ \
  --set project.slug=test \
  --set project.repo=orchard9/test \
  | kubectl apply --dry-run=client -f -
```

Expected:
1. No lint errors
2. Five Kubernetes resources rendered (Namespace, ExternalSecret, Deployment, Service, Ingress)
3. Deployment has exactly two containers
4. Ingress host is `test.sdlc.threesix.ai`
5. Namespace name is `sdlc-test`
6. ExternalSecret references `gcp-secret-store` and key `sdlc-fleet-gitea`
