# restart-sdlc

Kills any process on port 7777, rebuilds the project with `make install`, and relaunches `sdlc ui --port 7777` in the background.

## Usage

```bash
# Default — kill :7777, make install, relaunch sdlc ui
sdlc tool run restart-sdlc

# Direct bun invocation
echo '{}' | bun run .sdlc/tools/restart-sdlc/tool.ts --run | jq .
```

## How it works

1. **Kill** — runs `lsof -ti :<port>` to find bound PIDs, then `kill -9` each one. Waits 500ms for the port to fully release.
2. **Build** — runs `make install` in the project root (`$SDLC_ROOT`). If `make install` fails, the tool returns `ok: false` with the full output and skips step 3.
3. **Launch** — spawns `sdlc ui --port <port>` detached and unreferenced so the tool exits immediately. The UI process continues running in the background.

## Output

| Field | Type | Description |
|---|---|---|
| `killed_pids` | `number[]` | PIDs killed on the target port |
| `make_output` | `string` | Combined stdout+stderr from `make install` |
| `make_exit_code` | `number` | Exit code from `make install` (0 = success) |
| `ui_pid` | `number` | PID of the newly launched `sdlc ui` process |
