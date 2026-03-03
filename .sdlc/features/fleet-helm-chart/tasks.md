# Tasks: Helm chart — sdlc-server + git-sync sidecar

## T1 — Scaffold chart directory structure

Create the `deployments/helm/sdlc-server/` directory in `orchard9/k3s-fleet` with `Chart.yaml`, an empty `values.yaml`, and an empty `templates/` directory. This establishes the skeleton that all subsequent tasks fill in.

**Done when:** `helm lint deployments/helm/sdlc-server/` succeeds on the empty scaffold (only `values.yaml` and `Chart.yaml` present, no template errors).

---

## T2 — Write `values.yaml` with full defaults

Author the default `values.yaml` with all keys from the Values API in the design: `project.*`, `gitea.*`, `ingress.*`, `image.*`, `resources.*`. Placeholder values for project-specific fields; real defaults for shared infrastructure fields.

**Done when:** File exists and is valid YAML that can be parsed by `helm template`.

---

## T3 — Write `templates/_helpers.tpl`

Define the named template helpers:
- `sdlc-server.fullname` → `sdlc-{{ .Values.project.slug }}`
- `sdlc-server.namespace` → `sdlc-{{ .Values.project.slug }}`
- `sdlc-server.labels` → standard `app.kubernetes.io/` label block

**Done when:** `helm template` renders without "unknown function" or "nil pointer" errors related to these helpers.

---

## T4 — Write `templates/namespace.yaml`

Namespace resource named `sdlc-{{ .Values.project.slug }}` with `app.kubernetes.io/managed-by: sdlc-cluster` and `sdlc/project: <slug>` labels.

**Done when:** `helm template --set project.slug=test` renders a Namespace with name `sdlc-test`.

---

## T5 — Write `templates/external-secret.yaml`

ExternalSecret (ESO) in the project namespace. Pulls three keys (`url`, `token`, `user`) from the ClusterSecretStore and creates k8s Secret `gitea-credentials`. RefreshInterval 1h.

**Done when:** Rendered ExternalSecret has `secretStoreRef.name: gcp-secret-store`, `target.name: gitea-credentials`, and three `data` entries.

---

## T6 — Write `templates/deployment.yaml`

Deployment with two containers (sdlc-server + git-sync), shared `emptyDir` workspace volume, all env vars and resource limits from the design, liveness and readiness probes on sdlc-server, `GITSYNC_USERNAME`/`GITSYNC_PASSWORD` sourced from the `gitea-credentials` Secret.

**Done when:** `helm template --set project.slug=test --set project.repo=orchard9/test` renders a Deployment with exactly two containers named `sdlc-server` and `git-sync`.

---

## T7 — Write `templates/service.yaml`

ClusterIP Service in project namespace, port 80 → 8080, selector `app=sdlc-server, slug={{ slug }}`.

**Done when:** Rendered Service has `port: 80`, `targetPort: 8080`, and the correct selector labels.

---

## T8 — Write `templates/ingress.yaml`

Ingress in project namespace with TLS section (wildcard secret), host `{{ slug }}.{{ ingress.domain }}`, backend pointing to the Service at port 80, and annotation `nginx.ingress.kubernetes.io/proxy-read-timeout: "3600"`.

**Done when:** Rendered Ingress host is `test.sdlc.threesix.ai` with `--set project.slug=test`.

---

## T9 — `helm lint` and `helm template` smoke test

Run `helm lint deployments/helm/sdlc-server/` and `helm template sdlc-test deployments/helm/sdlc-server/ --set project.slug=test --set project.repo=orchard9/test`. Fix any errors found. Document the rendered output snippet in a comment.

**Done when:** Both commands exit 0 with no errors or warnings; five Kubernetes resources are rendered (Namespace, ExternalSecret, Deployment, Service, Ingress).

---

## T10 — `kubectl apply --dry-run=client` validation

Pipe the `helm template` output through `kubectl apply --dry-run=client -f -` against the cluster context. Fix any schema validation errors. This validates the rendered manifests are accepted by the Kubernetes API server schema.

**Done when:** `kubectl apply --dry-run=client` exits 0 with all resources validated.
