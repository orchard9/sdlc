.PHONY: install build frontend clean test lint install-cloudflared

# Build the frontend, install the sdlc binary, and ensure optional deps are available
install: frontend
	cargo install --path crates/sdlc-cli
	@$(MAKE) --no-print-directory install-cloudflared

# Install cloudflared (required for 'sdlc ui --tunnel'). Safe to re-run.
install-cloudflared:
	@if command -v cloudflared >/dev/null 2>&1; then \
		printf '  \033[32m✓\033[0m cloudflared already installed\n'; \
	elif command -v brew >/dev/null 2>&1; then \
		echo "  Installing cloudflared via Homebrew..."; \
		brew install cloudflare/cloudflare/cloudflared; \
	elif command -v apt-get >/dev/null 2>&1; then \
		echo "  Installing cloudflared via apt..."; \
		curl -fsSL https://pkg.cloudflare.com/cloudflare-main.gpg \
		  | sudo gpg --dearmor -o /usr/share/keyrings/cloudflare-main.gpg 2>/dev/null; \
		echo "deb [signed-by=/usr/share/keyrings/cloudflare-main.gpg] https://pkg.cloudflare.com/cloudflared $$(lsb_release -cs) main" \
		  | sudo tee /etc/apt/sources.list.d/cloudflared.list >/dev/null; \
		sudo apt-get update -qq && sudo apt-get install -y cloudflared; \
	elif command -v winget >/dev/null 2>&1; then \
		winget install Cloudflare.cloudflared; \
	else \
		printf '\n  \033[33m⚠\033[0m  cloudflared not installed\n'; \
		printf '     Needed only for: sdlc ui --tunnel\n'; \
		printf '     Install: https://developers.cloudflare.com/cloudflare-one/connections/connect-apps/install-and-setup/installation/\n\n'; \
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
