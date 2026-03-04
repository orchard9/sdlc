# Vision: v38-credential-pool

## Why this matters

In the cluster, each project pod needs to run Claude agents — but `~/.claude/` credentials
can't live on ephemeral pod filesystems. Without a solution, every pod either runs unauthenticated
or requires manual per-pod credential setup that breaks on pod restart.

The credential pool solves this once, for all pods, forever. Tokens go into Postgres.
Every agent run checks one out, uses it, returns it to the rotation. Multiple concurrent
runs across different pods never step on each other.

## What a user can do when this ships

- Insert Claude OAuth tokens into a single Postgres table once
- Every project pod in the fleet automatically uses those tokens for agent runs —
  no filesystem setup, no per-pod config, no manual credential injection
- Run agents on multiple projects simultaneously; each run gets a different token
  (round-robin by `last_used_at`) so no single account bears all the load
- If the database is unreachable or no tokens are configured, runs continue with
  whatever ambient auth is available — nothing breaks, just a warning log

## What this does NOT do

- Does not manage token lifecycle (expiry, refresh) — tokens are inserted manually
- Does not restrict which projects can use which tokens — pool is shared across all pods
- Does not retry failed checkouts — one attempt per run, graceful fallback
