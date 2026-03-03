# QA Plan: fleet-secrets-infra

## Test Cases

### TC-1: GCP Secret exists with correct structure

**Command:**
```bash
gcloud secrets versions access latest --secret=sdlc-fleet-gitea --project=orchard9 | jq 'keys'
```

**Expected:** `["org", "token", "url", "user"]` (four keys present)

### TC-2: GCP Secret values are correct

**Command:**
```bash
gcloud secrets versions access latest --secret=sdlc-fleet-gitea --project=orchard9 | jq '{url, user, org}'
```

**Expected:**
```json
{
  "url": "http://100.79.2.8:30300",
  "user": "claude-agent",
  "org": "orchard9"
}
```

Token value must be non-empty (verified separately, not printed).

### TC-3: ExternalSecret syncs successfully

**Command:**
```bash
kubectl get externalsecret sdlc-fleet-gitea-credentials -n projects -o jsonpath='{.status.conditions[?(@.type=="Ready")].status}'
```

**Expected:** `True`

### TC-4: Kubernetes Secret contains expected keys

**Command:**
```bash
kubectl get secret sdlc-fleet-gitea-credentials -n projects -o jsonpath='{.data}' | jq 'keys'
```

**Expected:** `["GITEA_ORG", "GITEA_TOKEN", "GITEA_URL", "GITEA_USER"]`

### TC-5: Gitea token value resolves to correct token

**Command:**
```bash
kubectl get secret sdlc-fleet-gitea-credentials -n projects -o jsonpath='{.data.GITEA_TOKEN}' | base64 -d
```

**Expected:** Matches the token from `sdlc secrets env export gitea` (`GITEA_TOKEN` value).

### TC-6: ExternalSecret is committed to k3s-fleet repo

**Verification:** `git log --oneline -5` in the k3s-fleet repo shows a commit containing `sdlc-fleet-gitea-external-secret.yaml` and the kustomization update.

## Pass Criteria

All six test cases must pass. TC-3 (ExternalSecret sync) is the gate — if it fails, the Kubernetes Secret in TC-4 and TC-5 cannot exist.
