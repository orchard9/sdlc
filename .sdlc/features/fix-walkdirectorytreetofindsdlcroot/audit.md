# Security Audit: Fix resolve_root to Walk Directory Tree

## Surface Area

Single function change in `crates/sdlc-cli/src/root.rs`. The change affects how the CLI discovers its working root directory — a local filesystem lookup using `std::fs`.

## Findings

### PATH TRAVERSAL / DIRECTORY ESCAPE

**Finding:** `find_sdlc_root` walks up the directory tree via `Path::parent()`. Could this cause the CLI to operate against an unexpected `.sdlc/` directory (e.g., one in `/home` or `/`)?

**Analysis:** This is by design and is safe. The behavior mirrors how tools like `git`, `cargo`, and `npm` discover their root. Walking upward is the expected and standard pattern. The CLI only reads from the discovered root — it does not write to arbitrary parent directories without explicit user intent. An adversarial `.sdlc/` directory in a parent path is no different from an adversarial `.git/` directory; the threat model is the same as for `git`.

**Action:** ACCEPTED — standard ancestor-walk pattern, no additional mitigation needed.

### SYMLINK / TRAVERSAL EDGE CASES

**Finding:** `current.join(".sdlc").is_dir()` follows symlinks by default.

**Analysis:** This is consistent with how all other filesystem operations in the codebase work. Symlinked `.sdlc/` directories are a valid configuration (e.g., shared project state). No change needed.

**Action:** ACCEPTED.

### INFINITE LOOP

**Finding:** Could the loop run forever?

**Analysis:** No. `Path::parent()` returns `None` at the filesystem root (`/` on Unix, drive root on Windows), which terminates the loop. This is guaranteed by the standard library.

**Action:** ACCEPTED.

## Verdict

No security issues. The change introduces no new attack surface beyond the standard ancestor-walk pattern used by every major developer tool.
