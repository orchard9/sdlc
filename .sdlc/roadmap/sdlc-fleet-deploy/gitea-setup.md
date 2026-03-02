## Gitea Setup — What Was Actually Done

### Decision: Use existing threesix/gitea (not a new instance)
Jordan's 145 existing repos are already on threesix/gitea. Dan Reeves would approve: don't deploy a new service when you have a perfectly good one running. It already uses postgres (shared cluster postgres, db=gitea).

### What was deployed
- **NodePort service** `gitea-tailscale` in `threesix` namespace
  - Manifest: `k3s-fleet/deployments/k8s/base/threesix/gitea-nodeport.yaml`
  - Accessible at `http://100.79.2.8:30300` from any Tailscale-connected machine
  - NOT exposed to the public internet (NodePort is LAN/Tailscale only)
  - Added to `k3s-fleet/deployments/k8s/base/threesix/kustomization.yaml`

### Agent user created
- Username: `claude-agent` (admin)
- Token scope: write:admin, write:repository, write:user, write:organization, read:*
- Token stored encrypted in sdlc secrets: `sdlc secrets env export gitea`
  - GITEA_URL=http://100.79.2.8:30300
  - GITEA_TOKEN=(encrypted)
  - GITEA_USER=claude-agent
  - GITEA_ORG=orchard9

### Org created
- `orchard9` organization created on threesix/gitea for sdlc-managed projects

### How Claude accesses Gitea in future sessions

```
eval $(sdlc secrets env export gitea)
curl -s -H "Authorization: token $GITEA_TOKEN" $GITEA_URL/api/v1/user
```

No port-forwarding required. Tailscale must be connected.

### Multiple remotes pattern (per-project)
```
git remote add origin  http://100.79.2.8:30300/orchard9/<slug>.git   # primary (agent)
git remote add github  git@github.com/orchard9/<slug>.git             # public mirror
```
Or: configure Gitea push-mirror via API on repo creation — auto-mirrors on every push.
