# Tasks: credential-pool-helm

## Task List

- [ ] **HELM-1**: Add `postgres.externalSecret` to `values.yaml`
  - File: `k3s-fleet/deployments/helm/sdlc-server/values.yaml`
  - Add `postgres:` section with `externalSecret.gsmKey: ""` and documenting comment
  - Comment must describe the expected GCP secret format (`database_url` field)

- [ ] **HELM-2**: Create `templates/external-secret-postgres.yaml`
  - File: `k3s-fleet/deployments/helm/sdlc-server/templates/external-secret-postgres.yaml`
  - Conditional on `{{- if .Values.postgres.externalSecret.gsmKey }}`
  - ExternalSecret apiVersion `external-secrets.io/v1beta1`
  - `secretStoreRef.kind: ClusterSecretStore`, name `gcp-secret-manager`
  - `target.name: postgres-sdlc-credentials`, `creationPolicy: Owner`
  - `remoteRef.key: {{ .Values.postgres.externalSecret.gsmKey }}`, `property: database_url`
  - `refreshInterval: 1h`

- [ ] **HELM-3**: Inject `DATABASE_URL` env var into `templates/deployment.yaml`
  - File: `k3s-fleet/deployments/helm/sdlc-server/templates/deployment.yaml`
  - Inside `sdlc-server` container env block, add conditional block:
    ```yaml
    {{- if .Values.postgres.externalSecret.gsmKey }}
    - name: DATABASE_URL
      valueFrom:
        secretKeyRef:
          name: postgres-sdlc-credentials
          key: DATABASE_URL
    {{- end }}
    ```

- [ ] **HELM-4**: Validate with `helm template` (disabled path)
  - Run `helm template test . --set project.slug=test` in chart directory
  - Verify: no ExternalSecret in output, no `DATABASE_URL` in deployment env

- [ ] **HELM-5**: Validate with `helm template` (enabled path)
  - Run `helm template test . --set project.slug=test --set postgres.externalSecret.gsmKey=k3sf-postgres-sdlc`
  - Verify: ExternalSecret present with correct `remoteRef.key`
  - Verify: `DATABASE_URL` secretKeyRef present in deployment env
  - Verify: ExternalSecret target name matches secretKeyRef name (`postgres-sdlc-credentials`)
