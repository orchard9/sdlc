# QA Plan: Helm chart — sdlc-server + git-sync sidecar

## Scope

All acceptance criteria from the spec, validated against the rendered output of the completed chart in `deployments/helm/sdlc-server/` of the k3s-fleet repo.

---

## TC-1: Helm lint passes

**Command:**
```bash
cd /Users/jordanwashburn/Workspace/orchard9/k3s-fleet
helm lint deployments/helm/sdlc-server/
```

**Pass:** Exit code 0. Output contains `1 chart(s) linted, 0 chart(s) failed`.

**Fail:** Any lint error or warning that indicates a schema or template problem.

---

## TC-2: Helm template renders five resources

**Command:**
```bash
helm template sdlc-test deployments/helm/sdlc-server/ \
  --set project.slug=test \
  --set project.repo=orchard9/test \
  | grep '^kind:' | sort
```

**Pass:** Output contains exactly these five kinds:
```
kind: Deployment
kind: ExternalSecret
kind: Ingress
kind: Namespace
kind: Service
```

**Fail:** Any kind is missing, or extra resources appear.

---

## TC-3: Deployment has exactly two containers

**Command:**
```bash
helm template sdlc-test deployments/helm/sdlc-server/ \
  --set project.slug=test \
  --set project.repo=orchard9/test \
  | yq 'select(.kind == "Deployment") | .spec.template.spec.containers[].name'
```

**Pass:** Output is exactly:
```
sdlc-server
git-sync
```

**Fail:** More or fewer containers, or wrong names.

---

## TC-4: ExternalSecret references correct secretStore and GSM key

**Command:**
```bash
helm template sdlc-test deployments/helm/sdlc-server/ \
  --set project.slug=test \
  --set project.repo=orchard9/test \
  | yq 'select(.kind == "ExternalSecret") | {store: .spec.secretStoreRef.name, key: .spec.data[0].remoteRef.key}'
```

**Pass:**
```yaml
store: gcp-secret-store
key: sdlc-fleet-gitea
```

**Fail:** Wrong store name or GSM key.

---

## TC-5: Ingress host matches `test.sdlc.threesix.ai`

**Command:**
```bash
helm template sdlc-test deployments/helm/sdlc-server/ \
  --set project.slug=test \
  --set project.repo=orchard9/test \
  | yq 'select(.kind == "Ingress") | .spec.rules[0].host'
```

**Pass:** Output is `test.sdlc.threesix.ai`.

**Fail:** Wrong host format.

---

## TC-6: Namespace name is `sdlc-test`

**Command:**
```bash
helm template sdlc-test deployments/helm/sdlc-server/ \
  --set project.slug=test \
  --set project.repo=orchard9/test \
  | yq 'select(.kind == "Namespace") | .metadata.name'
```

**Pass:** Output is `sdlc-test`.

**Fail:** Wrong namespace name.

---

## TC-7: All resources have correct namespace

**Command:**
```bash
helm template sdlc-test deployments/helm/sdlc-server/ \
  --set project.slug=test \
  --set project.repo=orchard9/test \
  | yq 'select(.kind != "Namespace") | .metadata.namespace' | sort -u
```

**Pass:** Only `sdlc-test` appears (no blank lines or other namespaces).

**Fail:** Any resource outside the project namespace.

---

## TC-8: `kubectl apply --dry-run=client` passes

**Command:**
```bash
helm template sdlc-test deployments/helm/sdlc-server/ \
  --set project.slug=test \
  --set project.repo=orchard9/test \
  | kubectl apply --dry-run=client -f - \
    --server https://100.79.2.8:6443 \
    --insecure-skip-tls-verify \
    --kubeconfig ~/.kube/orchard9-k3sf.yaml
```

**Pass:** Exit code 0. All resources show `configured (dry run)` or `created (dry run)`.

**Fail:** Any schema validation error.

---

## TC-9: git-sync sidecar uses correct env vars from Secret

**Command:**
```bash
helm template sdlc-test deployments/helm/sdlc-server/ \
  --set project.slug=test \
  --set project.repo=orchard9/test \
  | yq 'select(.kind == "Deployment") | .spec.template.spec.containers[] | select(.name == "git-sync") | .env[] | select(.name == "GITSYNC_USERNAME" or .name == "GITSYNC_PASSWORD")'
```

**Pass:** Both env vars have `valueFrom.secretKeyRef.name: gitea-credentials` and keys `user` / `token` respectively.

**Fail:** Credentials hard-coded or wrong secret name.

---

## TC-10: Ingress has SSE keep-alive annotation

**Command:**
```bash
helm template sdlc-test deployments/helm/sdlc-server/ \
  --set project.slug=test \
  --set project.repo=orchard9/test \
  | yq 'select(.kind == "Ingress") | .metadata.annotations["nginx.ingress.kubernetes.io/proxy-read-timeout"]'
```

**Pass:** Output is `"3600"`.

**Fail:** Annotation missing or wrong value.

---

## Pass Threshold

All 10 test cases must pass. TC-8 (dry-run) is gating — if cluster access is unavailable, it may be run against a local kind cluster and noted.
