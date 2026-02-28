# Quality Check

Runs platform shell commands from `.sdlc/config.yaml` and reports pass/fail.

## Usage

```bash
# Run all configured checks
sdlc tool run quality-check

# Filter to checks whose name matches a string
sdlc tool run quality-check --scope test
```

## How it works

Reads `platform.commands` from `.sdlc/config.yaml`, runs each script as a shell command,
and reports pass/fail with the last 500 characters of output.

## Adding checks

Add entries under `platform:` → `commands:` in `.sdlc/config.yaml`:

```yaml
platform:
  commands:
    - name: test
      description: Run unit tests
      script: cargo test --all
    - name: lint
      description: Run linter
      script: cargo clippy --all -- -D warnings
```

The quality-check tool picks them up automatically — no code changes needed.
