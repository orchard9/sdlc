.PHONY: install build frontend clean test lint install-orch-tunnel

# Build the frontend, install the sdlc binary, and ensure optional deps are available
install: frontend
	cargo install --path crates/sdlc-cli --locked
	@$(MAKE) --no-print-directory install-orch-tunnel

# Install orch-tunnel (required for 'sdlc ui --tunnel'). Safe to re-run.
install-orch-tunnel:
	@if command -v orch-tunnel >/dev/null 2>&1; then \
		printf '  \033[32m✓\033[0m orch-tunnel already installed\n'; \
	elif command -v brew >/dev/null 2>&1; then \
		echo "  Installing orch-tunnel via Homebrew..."; \
		brew install orch-tunnel; \
	elif command -v gh >/dev/null 2>&1; then \
		echo "  Installing orch-tunnel via gh release..."; \
		gh release download --repo orchard9/tunnel --pattern "orch-tunnel-linux-*" -D /tmp/orch-tunnel-install; \
		install -m 755 /tmp/orch-tunnel-install/orch-tunnel-linux-* /usr/local/bin/orch-tunnel; \
	else \
		printf '\n  \033[33m⚠\033[0m  orch-tunnel not installed\n'; \
		printf '     Needed only for: sdlc ui --tunnel\n'; \
		printf '     Install: gh release download --repo orchard9/tunnel\n\n'; \
	fi

# Build everything (frontend + Rust workspace) without installing
build: frontend
	cargo build --all

# Build the frontend assets (required for the embedded UI)
frontend:
	cd frontend && npm ci && npm run build

# Run all tests (skips npm build step to avoid hangs)
test:
	SDLC_NO_NPM=1 cargo test --all

# Lint
lint:
	cargo clippy --all -- -D warnings
	cd frontend && npx tsc --noEmit

# Remove build artifacts
clean:
	cargo clean
	rm -rf frontend/dist frontend/node_modules
