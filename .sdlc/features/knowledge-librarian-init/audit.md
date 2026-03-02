# Audit: sdlc knowledge librarian init

## Scope

Security audit of the `sdlc knowledge librarian init` command: the `librarian_init` function and all helpers (`harvest_investigations`, `harvest_ponders`, `harvest_guidelines`, `upsert_knowledge_entry`, `seed_catalog`, `write_librarian_agent_file`, `cross_ref_pass`, `librarian_harvest_workspace`) in `crates/sdlc-core/src/knowledge.rs`, and the CLI dispatch in `crates/sdlc-cli/src/cmd/knowledge.rs`.

## Threat Model

This is a local CLI tool with no network surface. There is no authentication layer, no HTTP handler, no daemon. The attack surface is:

1. **Slug injection via workspace names** — slugs from `.sdlc/investigations/` and `.sdlc/roadmap/` become knowledge entry slugs and filesystem paths.
2. **Content injection via workspace artifacts** — session bodies and ponder artifacts become file content in `content.md`.
3. **Path traversal via `publish_path`** — guideline harvest reads `std::fs::read_to_string(publish_path)` where `publish_path` is stored in an investigation manifest.
4. **Agent file template injection** — project name and catalog YAML are substituted into an agent file template and written to `.claude/agents/knowledge-librarian.md`.
5. **Write amplification on catalog seed** — `seed_catalog` calls `add_class` in a loop, each of which performs a load-modify-save cycle.

## Findings

### F1 — Slug Validation Blocks Path Traversal (PASS)

Knowledge entry slugs are constructed as `investigation-<workspace_slug>`, `ponder-<workspace_slug>`, or `guideline-<workspace_slug>`. The prefix is hardcoded, and `<workspace_slug>` is the value from the manifest loaded by `crate::investigation::list` / `crate::ponder::PonderEntry::list`. Both list functions read manifest files written by `sdlc investigate` / `sdlc ponder`, which call `validate_slug` at creation time. `validate_slug` enforces `^[a-z0-9][a-z0-9\-]*[a-z0-9]$|^[a-z0-9]$` with a 64-char cap — dots, slashes, and path separators are rejected. Slug components cannot escape the `.sdlc/knowledge/` directory.

**Finding: No path traversal possible via slug.** Closed.

### F2 — Guideline `publish_path` Is Not Validated (TRACK)

In `harvest_guidelines`:

```rust
let Some(ref publish_path) = inv.publish_path else { continue; };
let content = std::fs::read_to_string(publish_path).unwrap_or_default();
```

`publish_path` is an arbitrary `PathBuf` stored in the investigation manifest — it is not constrained to the project root. A crafted manifest with `publish_path: /etc/passwd` would cause the librarian to read and embed that file's content into a knowledge entry. However:

- The manifest is authored by `sdlc investigate update --publish-path` which is itself a CLI call made by the project owner.
- There is no untrusted user input path here — it is the same operator who owns the `.sdlc/` directory.
- The content is written to a `content.md` file under `.sdlc/knowledge/` (no execution, no template substitution).
- `read_to_string` failure is silently swallowed as `unwrap_or_default()` — no sensitive data leaks to stdout in error cases.

**Finding: Low risk given local trust model. Track as a hardening opportunity.** Action: `sdlc task add` to constrain `publish_path` to paths under `root` in a future pass.
<br>**Disposition: Track.**

### F3 — Agent File Template Injection via Project Name (PASS)

`write_librarian_agent_file` derives the project name from the directory name (`extract_project_name`) and substitutes it into the `LIBRARIAN_AGENT_TEMPLATE` via string replacement. The resulting file is a Markdown agent instruction — it is not executable. A project directory with a name like `{CATALOG_YAML}` would produce a template double-substitution, but only in the generated `.claude/agents/knowledge-librarian.md` file, not in any evaluated code path. The agent file is read by Claude Code, not executed as a process. There is no shell injection surface here.

**Finding: No code execution risk from template substitution. The file is inert Markdown.** Closed.

### F4 — Catalog YAML Embedded in Agent File (PASS)

The catalog is serialized as YAML and substituted into the agent template. The YAML content is controlled by the project owner via `sdlc knowledge catalog add`. No external input flows into the catalog without passing through the CLI argument parser and `validate_code`. The embedded YAML is read as documentation by the agent, not parsed or evaluated in any code path.

**Finding: No injection surface. Catalog YAML is operator-controlled and statically embedded.** Closed.

### F5 — `unwrap_or_default()` on YAML Serialization in Agent File (PASS)

```rust
let catalog_yaml = serde_yaml::to_string(&catalog).unwrap_or_default();
```

`serde_yaml::to_string` serializing a well-typed `Catalog` struct should not fail in practice (no cycles, no unrepresentable values). If it did, the template would contain an empty string for `{CATALOG_YAML}`, and the agent file would be written with no catalog section — a functional degradation, not a security issue. The review noted this and accepted it; confirmed as low risk.

**Finding: Functional degradation on serialization failure; not a security issue.** Closed.

### F6 — No Symlink Traversal Check on Knowledge Directory Iteration (TRACK)

`list` iterates `.sdlc/knowledge/` with `std::fs::read_dir`. If a malicious symlink exists in that directory pointing to a target outside the project, `load` would read and deserialize it as a knowledge entry. Exploiting this requires write access to `.sdlc/knowledge/` — which means the operator already has full access to the project root. Not a meaningful threat in the local CLI trust model.

**Finding: No practical risk given local operator trust. Track as a defense-in-depth hardening item for future environments (e.g. mounted directories).** Disposition: Track.

### F7 — No Size Limit on Harvested Content (TRACK)

`harvest_investigations`, `harvest_ponders`, and `harvest_guidelines` read full file content (session bodies, ponder artifacts, guideline files) into memory before writing to `content.md`. There is no size cap. For normal project sizes this is not a concern. For a project with unusually large session logs, the `cross_ref_pass` loading all entries into memory with O(n²) comparison is also unbounded. These are acknowledged in the review as design-acceptable at current scale (<200 entries).

**Finding: No security risk; resource exhaustion theoretical at large scale.** Disposition: Track (existing review observation, no new action).

### F8 — `maintenance-log.yaml` Is Append-Only, No Size Bound (ACCEPT)

Each `librarian_init` run appends at minimum one maintenance action. The log is never truncated. Over hundreds of init runs the log grows linearly. At project scale this is negligible (each action is ~5 YAML fields). No exploitable condition.

**Finding: Accepted. Log growth is bounded by project operator cadence.** Disposition: Accept.

## Summary Table

| ID | Finding | Severity | Disposition |
|----|---------|----------|-------------|
| F1 | Slug validation blocks path traversal | None | Closed |
| F2 | `publish_path` not constrained to project root | Low | Track |
| F3 | Agent file template injection via project name | None | Closed |
| F4 | Catalog YAML embedded in agent file | None | Closed |
| F5 | `unwrap_or_default()` on YAML serialization | None | Closed |
| F6 | No symlink traversal check | None (local trust) | Track |
| F7 | No size limit on harvested content | None (current scale) | Track |
| F8 | Maintenance log grows unbounded | None | Accept |

## Actions Taken

- **F2 (Track):** Added task to constrain `publish_path` validation to project root in a future hardening pass.
- **F6 (Track):** Noted for future defense-in-depth work if sdlc is ever used in multi-tenant environments.
- All other findings: closed or accepted with documented rationale.

## Verdict

APPROVED. The `knowledge-librarian-init` command has no high-severity security surface. All file writes use atomic write primitives. Slug validation is enforced at entry creation. No executable code is generated. The only tracked item (F2) is a theoretical local-trust hardening gap with no exploitable path given the operator trust model.
