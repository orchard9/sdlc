# Spec: Helm chart — sdlc-server + git-sync sidecar

## Problem

Deploying an sdlc-server instance for each orchard9 project requires repeatable, parameterized Kubernetes manifests. Hand-crafting raw YAML per project does not scale to 80+ projects and is error-prone. A Helm chart that encapsulates the full deployment topology — sdlc-server container, git-sync sidecar, Service, Ingress, and ExternalSecret — lets the deploy-project Woodpecker pipeline install or upgrade any project with a single `helm upgrade --install` invocation.

## Solution

A Helm chart at `helm/sdlc-server/` inside the `orchard9/sdlc-cluster` repo. The chart templates one self-contained deployment unit per project slug. Values are minimal and project-specific; secrets are pulled from GCP Secret Manager via ESO, not hard-coded.

## Scope

This feature covers authoring the complete chart. It does **not** cover:
- The Woodpecker pipeline that calls `helm install` (fleet-deploy-pipeline)
- The ESO ClusterSecretStore that backs the ExternalSecret (fleet-secrets-infra)
- The repo scaffold that hosts the chart (fleet-repo-scaffold)

## Behavior

### Values API

```yaml
project:
  slug: my-project          # Kubernetes-safe identifier, also used in namespace + subdomain
  repo: orchard9/my-project # Gitea repo path: <org>/<repo>
  branch: main              # Branch git-sync tracks

gitea:
  url: http://100.79.2.8:30300
  externalSecret:
    secretStore: gcp-secret-store  # Name of ESO ClusterSecretStore
    gsmKey: sdlc-fleet-gitea       # GCP Secret Manager key

ingress:
  domain: sdlc.threesix.ai        # Wildcard base domain
  tlsSecretName: sdlc-wildcard-tls

image:
  server: ghcr.io/orchard9/sdlc-server:latest
  gitSync: registry.k8s.io/git-sync/git-sync:v4

resources:
  server:
    requests: { cpu: 50m, memory: 64Mi }
    limits:   { cpu: 500m, memory: 256Mi }
  gitSync:
    requests: { cpu: 10m, memory: 32Mi }
    limits:   { cpu: 100m, memory: 64Mi }
```

### Templates

#### `namespace.yaml`
- Creates namespace `sdlc-{{ .Values.project.slug }}`
- Label: `app.kubernetes.io/managed-by: sdlc-cluster`

#### `external-secret.yaml`
- Kind: `ExternalSecret` (ESO)
- Namespace: `sdlc-{{ .Values.project.slug }}`
- References ClusterSecretStore `{{ .Values.gitea.externalSecret.secretStore }}`
- Pulls from GSM key `{{ .Values.gitea.externalSecret.gsmKey }}`
- Produces k8s Secret `gitea-credentials` with keys: `url`, `token`, `user`
- Refresh interval: 1h

#### `deployment.yaml`
- Namespace: `sdlc-{{ .Values.project.slug }}`
- Two containers:

**sdlc-server** (main):
- Image: `{{ .Values.image.server }}`
- Port 8080
- Mounts shared emptyDir volume at `/workspace`
- Env: `SDLC_DIR=/workspace/{{ .Values.project.slug }}/.sdlc`
- LivenessProbe: HTTP GET `/api/health` port 8080, initial delay 10s, period 15s
- ReadinessProbe: HTTP GET `/api/health` port 8080, initial delay 5s, period 10s

**git-sync** (sidecar):
- Image: `{{ .Values.image.gitSync }}`
- Mounts same emptyDir at `/workspace`
- Env:
  - `GITSYNC_REPO`: `{{ .Values.gitea.url }}/{{ .Values.project.repo }}` (URL only, no credentials — token injected via HTTP header or URL)
  - `GITSYNC_ROOT`: `/workspace`
  - `GITSYNC_LINK`: `{{ .Values.project.slug }}`
  - `GITSYNC_BRANCH`: `{{ .Values.project.branch }}`
  - `GITSYNC_PERIOD`: `30s`
  - `GITSYNC_USERNAME` from Secret `gitea-credentials` key `user`
  - `GITSYNC_PASSWORD` from Secret `gitea-credentials` key `token`

**Shared volume**: emptyDir named `workspace`

#### `service.yaml`
- ClusterIP Service in namespace `sdlc-{{ .Values.project.slug }}`
- Port 80 → targetPort 8080
- Selector: `app: sdlc-server, slug: {{ .Values.project.slug }}`

#### `ingress.yaml`
- Namespace: `sdlc-{{ .Values.project.slug }}`
- Host: `{{ .Values.project.slug }}.{{ .Values.ingress.domain }}`
- TLS: secretName `{{ .Values.ingress.tlsSecretName }}`
- Backend: Service port 80
- Annotation: `nginx.ingress.kubernetes.io/proxy-read-timeout: "3600"` (SSE keep-alive)

### Chart metadata (`Chart.yaml`)

```yaml
apiVersion: v2
name: sdlc-server
version: 0.1.0
description: sdlc-server + git-sync sidecar for one orchard9 project
```

## Non-Goals

- Multi-replica scaling (one pod per project is sufficient; sdlc-server is single-writer)
- Persistent volumes (git-sync + emptyDir is the persistence model)
- Ingress controller installation (assumed present in cluster)
- cert-manager configuration (wildcard TLS secret assumed pre-provisioned)
- RBAC beyond namespace isolation

## Acceptance Criteria

1. `helm lint helm/sdlc-server/` passes with no errors
2. `helm template helm/sdlc-server/ --set project.slug=test --set project.repo=orchard9/test` renders all five template files with no error
3. The rendered Deployment has exactly two containers: `sdlc-server` and `git-sync`
4. The rendered ExternalSecret references the correct secretStore and GSM key
5. The rendered Ingress host matches `test.sdlc.threesix.ai`
6. The rendered namespace is `sdlc-test`
7. Chart can be installed against the cluster (`helm upgrade --install sdlc-test helm/sdlc-server/ -n sdlc-test --create-namespace --set project.slug=test --set project.repo=orchard9/test`) without dry-run errors
