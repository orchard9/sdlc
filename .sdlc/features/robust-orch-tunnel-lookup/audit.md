# Security Audit: Robust orch-tunnel lookup

## Scope

Changes to `crates/sdlc-server/src/tunnel.rs`: three-tier binary discovery, login shell spawning, fallback path probing.

## Findings

### A1: Shell injection via $SHELL (LOW)
`read_login_shell_path()` uses `std::env::var("SHELL")` to determine which shell to spawn. If an attacker controls the `SHELL` env var, they could point it at a malicious binary.

**Assessment:** Low risk. The `SHELL` env var is set by the OS at login time and is not user-controllable via the web UI or API. An attacker who can modify server environment variables already has full control. The fallback to `/bin/sh` is safe.

**Action:** Accept — standard Unix pattern; no mitigation needed.

### A2: Arbitrary binary execution via fallback paths (LOW)
Tier 3 checks well-known paths (`/opt/homebrew/bin/`, `/usr/local/bin/`, `~/.cargo/bin/`) and executes whatever file is found there. A malicious file at one of these locations would be executed.

**Assessment:** Low risk. These are standard system directories with root/admin-only write access. `~/.cargo/bin/` is user-writable but an attacker who can write to the user's home directory already has equivalent access. The `--version` call in `check_orch_tunnel()` does execute the binary, but this is the same trust model as `which::which()`.

**Action:** Accept — same trust model as the original implementation.

### A3: Login shell timeout prevents DoS (POSITIVE)
The 3-second timeout on `read_login_shell_path()` prevents a broken `.zshrc`/`.bashrc` from hanging the server indefinitely. This is a security improvement over a naive implementation.

**Action:** No issue — good defensive design.

### A4: No PATH injection into process environment (POSITIVE)
The login shell PATH is only used with `which::which_in()` for binary lookup — it is NOT injected into the process's own `PATH` environment. This prevents any side effects from the shell PATH leaking into child processes.

**Action:** No issue — correct isolation.

## Verdict

No security issues requiring remediation. The implementation follows standard Unix binary discovery patterns with appropriate defensive measures (timeout, fallback to `/bin/sh`, PATH isolation).
