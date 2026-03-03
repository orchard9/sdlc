# QA Plan: fleet-deploy-pipeline

## Scope

Verify that `pipelines/deploy-project.yaml` in `orchard9/sdlc-cluster`:
1. Is syntactically valid Woodpecker YAML
2. Deploys an sdlc-server release idempotently via helm
3. Creates the expected Kubernetes resources
4. Is visible and re-triggerable from the Woodpecker UI

## Test Cases

### TC-1: YAML Syntax Validation

**Given:** `pipelines/deploy-project.yaml` exists in the repo  
**When:** `yamllint pipelines/deploy-project.yaml` is run  
**Then:** Exit code 0, no errors reported

---

### TC-2: Required Fields Present

**Given:** The pipeline file  
**When:** Inspected manually or with `yq`  
**Then:**
- `when.event` is `custom`
- At least one step named `deploy` exists
- Step image is `alpine/helm:3.14`
- Commands reference `SDLC_PROJECT_SLUG`, `SDLC_REPO`, `SDLC_BRANCH`
- `helm upgrade --install` is present with `--create-namespace`, `--atomic`, `--wait`

---

### TC-3: Namespace Construction

**Given:** `SDLC_PROJECT_SLUG=foo`  
**When:** The deploy step command is evaluated  
**Then:** Namespace is `sdlc-foo`, Helm release name is `sdlc-foo`

---

### TC-4: Ingress Host Construction

**Given:** `SDLC_PROJECT_SLUG=foo`  
**When:** The `--set ingress.host` value is evaluated  
**Then:** Value is `foo.sdlc.threesix.ai`

---

### TC-5: First-Run Install (Integration)

**Given:** Namespace `sdlc-test-project` does not exist  
**When:** Pipeline triggered with `SDLC_PROJECT_SLUG=test-project SDLC_REPO=orchard9/test-project`  
**Then:**
- Woodpecker pipeline run status: success (green)
- `kubectl get ns sdlc-test-project` → exists
- `kubectl get deployment -n sdlc-test-project` → at least one deployment in Running state
- `kubectl get svc -n sdlc-test-project` → service exists
- `kubectl get ingress -n sdlc-test-project` → ingress with host `test-project.sdlc.threesix.ai`

---

### TC-6: Idempotency (Second Run)

**Given:** `sdlc-test-project` namespace and release already exist from TC-5  
**When:** Pipeline triggered again with same parameters  
**Then:**
- Woodpecker pipeline run status: success (green)
- No errors from helm (no-op or fast reapply)
- Resource state unchanged from TC-5

---

### TC-7: Woodpecker UI Visibility

**Given:** Pipeline file is committed to `orchard9/sdlc-cluster`  
**When:** Woodpecker dashboard for `orchard9/sdlc-cluster` is opened  
**Then:**
- `deploy-project` pipeline appears in the pipeline list
- "New Build" button is available
- Env var fields for `SDLC_PROJECT_SLUG`, `SDLC_REPO`, `SDLC_BRANCH` can be entered

---

### TC-8: Missing Required Parameter Behavior

**Given:** Pipeline triggered with `SDLC_PROJECT_SLUG` empty or unset  
**When:** The deploy step runs  
**Then:** Pipeline fails early with a clear error (helm will fail on empty release name or namespace), not silently succeed with a broken state

---

## Pass Criteria

All TC-1 through TC-7 must pass. TC-8 is informational — graceful failure is acceptable; silent corruption is not.

Integration tests (TC-5, TC-6, TC-7) require the cluster, Woodpecker, and dependent features (`fleet-helm-chart`, `fleet-secrets-infra`, `fleet-repo-scaffold`) to be operational. If those dependencies are not yet deployed, TC-5/TC-6/TC-7 are deferred and noted in qa-results.md.

## Cleanup

After TC-6 passes, remove the test release and namespace:
```bash
helm uninstall sdlc-test-project -n sdlc-test-project
kubectl delete ns sdlc-test-project
```
