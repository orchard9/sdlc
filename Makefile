.PHONY: install build frontend clean test lint

# Build the frontend and install the sdlc binary to ~/.cargo/bin
install: frontend
	cargo install --path crates/sdlc-cli

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
