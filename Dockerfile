# Multi-stage build for sdlc-server
# Stage 1: Build frontend
FROM node:22-alpine AS frontend
WORKDIR /app/frontend
COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci
COPY frontend/ ./
RUN npm run build

# Stage 2: Build Rust binary (ponder CLI which includes the UI server)
FROM rust:1.86-slim AS builder
WORKDIR /app

# Install build deps
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy workspace files
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates/ crates/
# Copy tool sources embedded via include_str! in templates.rs
COPY .sdlc/tools/ .sdlc/tools/

# Copy pre-built frontend so build.rs finds it at ../../frontend/dist
# relative to crates/sdlc-server/
COPY --from=frontend /app/frontend/dist frontend/dist

# Build the ponder binary (sdlc-cli) in release mode
# SDLC_NO_NPM=1 tells build.rs to skip npm (frontend already built)
ENV SDLC_NO_NPM=1
RUN cargo build --release --bin ponder

# Stage 3: Minimal runtime image
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/ponder /usr/local/bin/ponder

# ponder ui start listens on 8080 in fleet mode
EXPOSE 8080

# SDLC_ROOT is set by the Kubernetes deployment (e.g. /workspace/<slug>)
ENTRYPOINT ["/usr/local/bin/ponder"]
CMD ["ui", "start", "--port", "8080", "--no-open", "--no-tunnel"]
