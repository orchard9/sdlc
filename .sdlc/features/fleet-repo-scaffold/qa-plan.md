# QA Plan: Scaffold orchard9/sdlc-cluster Repo

## Scope

Verify that the `orchard9/sdlc-cluster` repository was created correctly on threesix/gitea with the canonical directory structure, valid Helm chart skeleton, Woodpecker CI wiring, and a clean initial commit.

## Test Cases

### TC-1: Repository exists on Gitea

**Steps:**
1. `GET http://100.79.2.8:30300/api/v1/repos/orchard9/sdlc-cluster` with `claude-agent` token.

**Expected:** HTTP 200, `full_name == "orchard9/sdlc-cluster"`, `private == false`.

---

### TC-2: All required files present

**Steps:**
1. `GET /api/v1/repos/orchard9/sdlc-cluster/contents/<path>` for each file below.

**Expected:** Each path returns HTTP 200.

| Path |
|---|
| `.woodpecker.yml` |
| `README.md` |
| `helm/sdlc-server/Chart.yaml` |
| `helm/sdlc-server/values.yaml` |
| `helm/sdlc-server/templates/.gitkeep` |
| `pipelines/deploy-project.yaml.stub` |
| `pipelines/reconcile-projects.yaml.stub` |
| `external-secrets/cluster-secret-store.yaml.stub` |

---

### TC-3: Chart.yaml is valid

**Steps:**
1. Clone the repo locally.
2. Inspect `helm/sdlc-server/Chart.yaml`.

**Expected:** Contains `apiVersion: v2`, `name: sdlc-server`, `version: 0.1.0`.

---

### TC-4: helm lint passes

**Steps:**
1. In the cloned repo root, run `helm lint helm/sdlc-server`.

**Expected:** Exit code 0, no `[ERROR]` lines in output.

---

### TC-5: .woodpecker.yml contains helm lint step

**Steps:**
1. Read `.woodpecker.yml` from the cloned repo.

**Expected:** File contains `helm lint helm/sdlc-server` and `alpine/helm` image reference. Trigger is push to `main`.

---

### TC-6: Initial commit message is correct

**Steps:**
1. `GET /api/v1/repos/orchard9/sdlc-cluster/commits?limit=1`.

**Expected:** Most recent commit message is `chore: scaffold sdlc-cluster repo`.

---

### TC-7: README.md describes purpose

**Steps:**
1. Read `README.md` from the cloned repo.

**Expected:** File contains text referencing sdlc-server deployments and the orchard9 fleet.

## Pass Criteria

All 7 test cases pass with no failures.

## Tooling

- `curl` or Gitea REST API calls authenticated with `claude-agent` token
- `git clone` + `helm lint` locally (Helm CLI required)
