#!/usr/bin/env bash
# deploy.sh — Deploy Telegram Digest Bot to a remote host
#
# Usage:
#   ./deploy/deploy.sh --host <hostname> [--user <ssh-user>] [--arch <amd64|arm64|armv7>]
#
# Prerequisites:
#   - SSH access to the target host (key-based auth recommended)
#   - Rust toolchain with `cross` or `cargo-zigbuild` installed locally
#   - Target host has systemd (Ubuntu 22.04, Debian 12, or Raspberry Pi OS)
#
# This script is idempotent — safe to run multiple times.

set -euo pipefail

# ──────────────────────────────────────────────
# Defaults
# ──────────────────────────────────────────────
HOST=""
SSH_USER="root"
ARCH="amd64"
DEPLOY_DIR="/opt/telegram-bot"
SERVICE_USER="telegram-bot"
DRY_RUN=false

# ──────────────────────────────────────────────
# Argument parsing
# ──────────────────────────────────────────────
usage() {
    cat <<EOF
Usage: $(basename "$0") [OPTIONS]

Deploy Telegram Digest Bot to a remote host via SSH.

Options:
  --host HOST       SSH hostname or IP address (required)
  --user USER       SSH login user (default: root)
  --arch ARCH       Target architecture: amd64, arm64, armv7 (default: amd64)
  --dry-run         Print commands without executing them
  -h, --help        Show this help message

Examples:
  # Deploy to a VPS:
  ./deploy/deploy.sh --host myserver.example.com --user ubuntu --arch amd64

  # Deploy to Raspberry Pi 4 (64-bit):
  ./deploy/deploy.sh --host raspberrypi.local --user pi --arch arm64

  # Deploy to Raspberry Pi 3 (32-bit):
  ./deploy/deploy.sh --host raspberrypi.local --user pi --arch armv7

EOF
    exit 0
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --host)    HOST="$2"; shift 2 ;;
        --user)    SSH_USER="$2"; shift 2 ;;
        --arch)    ARCH="$2"; shift 2 ;;
        --dry-run) DRY_RUN=true; shift ;;
        -h|--help) usage ;;
        *)
            echo "Unknown option: $1"
            echo "Run $(basename "$0") --help for usage."
            exit 1
            ;;
    esac
done

# Validate required arguments
if [[ -z "$HOST" ]]; then
    echo "ERROR: --host is required."
    echo "Run $(basename "$0") --help for usage."
    exit 1
fi

# Validate architecture
case "$ARCH" in
    amd64)  RUST_TARGET="x86_64-unknown-linux-musl" ;;
    arm64)  RUST_TARGET="aarch64-unknown-linux-musl" ;;
    armv7)  RUST_TARGET="armv7-unknown-linux-musleabihf" ;;
    *)
        echo "ERROR: Unknown architecture: $ARCH. Must be amd64, arm64, or armv7."
        exit 1
        ;;
esac

SSH_DEST="${SSH_USER}@${HOST}"

# ──────────────────────────────────────────────
# Helper functions
# ──────────────────────────────────────────────
info() { echo "  [INFO] $*"; }
step() { echo ""; echo "==> $*"; }
run()  {
    if [[ "$DRY_RUN" == "true" ]]; then
        echo "  [DRY-RUN] $*"
    else
        eval "$@"
    fi
}

ssh_run() {
    local cmd="$*"
    if [[ "$DRY_RUN" == "true" ]]; then
        echo "  [DRY-RUN] ssh ${SSH_DEST} '${cmd}'"
    else
        ssh "${SSH_DEST}" "${cmd}"
    fi
}

# ──────────────────────────────────────────────
# Script root: repository root (one level up from deploy/)
# ──────────────────────────────────────────────
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

echo ""
echo "======================================================"
echo "  Telegram Digest Bot — Deployment"
echo "======================================================"
echo "  Host:   ${HOST} (${SSH_USER})"
echo "  Arch:   ${ARCH} (${RUST_TARGET})"
echo "  Deploy: ${DEPLOY_DIR}"
if [[ "$DRY_RUN" == "true" ]]; then
    echo "  Mode:   DRY RUN — no changes will be made"
fi
echo "======================================================"

# ──────────────────────────────────────────────
# Step 1: Build binary
# ──────────────────────────────────────────────
step "Building sdlc binary for ${ARCH}..."
cd "${REPO_ROOT}"

if command -v cross &>/dev/null; then
    info "Using 'cross' for cross-compilation (Docker required)"
    run cross build --release --target "${RUST_TARGET}" --bin sdlc
elif command -v cargo-zigbuild &>/dev/null; then
    info "Using 'cargo-zigbuild' for cross-compilation"
    run cargo zigbuild --release --target "${RUST_TARGET}" --bin sdlc
else
    info "Neither 'cross' nor 'cargo-zigbuild' found."
    if [[ "$ARCH" == "amd64" ]]; then
        info "Falling back to native cargo build (musl target)."
        run cargo build --release --target "${RUST_TARGET}" --bin sdlc
    else
        echo "ERROR: Cross-compilation tool required for ${ARCH}."
        echo "Install 'cross': cargo install cross --git https://github.com/cross-rs/cross"
        exit 1
    fi
fi

BINARY_PATH="${REPO_ROOT}/target/${RUST_TARGET}/release/sdlc"
if [[ ! -f "$BINARY_PATH" && "$DRY_RUN" == "false" ]]; then
    echo "ERROR: Binary not found at ${BINARY_PATH}"
    exit 1
fi
info "Binary built: ${BINARY_PATH}"

# ──────────────────────────────────────────────
# Step 2: Prepare host directories and user
# ──────────────────────────────────────────────
step "Preparing host directories and system user..."

ssh_run "id ${SERVICE_USER} &>/dev/null || useradd --system --no-create-home --shell /usr/sbin/nologin ${SERVICE_USER}"
ssh_run "mkdir -p ${DEPLOY_DIR}/{bin,data}"
ssh_run "chown -R ${SERVICE_USER}:${SERVICE_USER} ${DEPLOY_DIR}"
ssh_run "chmod 750 ${DEPLOY_DIR}"
ssh_run "chmod 750 ${DEPLOY_DIR}/bin ${DEPLOY_DIR}/data"

# ──────────────────────────────────────────────
# Step 3: Copy binary
# ──────────────────────────────────────────────
step "Uploading binary to ${SSH_DEST}:${DEPLOY_DIR}/bin/sdlc..."

# Backup existing binary if present
ssh_run "test -f ${DEPLOY_DIR}/bin/sdlc && cp ${DEPLOY_DIR}/bin/sdlc ${DEPLOY_DIR}/bin/sdlc.bak || true"

if [[ "$DRY_RUN" == "true" ]]; then
    echo "  [DRY-RUN] rsync ${BINARY_PATH} ${SSH_DEST}:${DEPLOY_DIR}/bin/sdlc"
else
    rsync --progress --compress "${BINARY_PATH}" "${SSH_DEST}:${DEPLOY_DIR}/bin/sdlc"
fi

ssh_run "chmod 755 ${DEPLOY_DIR}/bin/sdlc"
ssh_run "chown ${SERVICE_USER}:${SERVICE_USER} ${DEPLOY_DIR}/bin/sdlc"

# ──────────────────────────────────────────────
# Step 4: Copy systemd unit files
# ──────────────────────────────────────────────
step "Installing systemd unit files..."

UNITS=(
    "${SCRIPT_DIR}/telegram-message-store.service"
    "${SCRIPT_DIR}/telegram-digest.service"
    "${SCRIPT_DIR}/telegram-digest.timer"
)

for unit in "${UNITS[@]}"; do
    unit_name="$(basename "$unit")"
    if [[ "$DRY_RUN" == "true" ]]; then
        echo "  [DRY-RUN] rsync ${unit} ${SSH_DEST}:/etc/systemd/system/${unit_name}"
    else
        rsync --compress "${unit}" "${SSH_DEST}:/etc/systemd/system/${unit_name}"
    fi
    info "Installed: /etc/systemd/system/${unit_name}"
done

# ──────────────────────────────────────────────
# Step 5: Check for .env file
# ──────────────────────────────────────────────
step "Checking for .env file on host..."

ENV_EXISTS=$(ssh_run "test -f ${DEPLOY_DIR}/.env && echo yes || echo no" 2>/dev/null || echo "no")
if [[ "$ENV_EXISTS" == "no" && "$DRY_RUN" == "false" ]]; then
    echo ""
    echo "  WARNING: No .env file found at ${DEPLOY_DIR}/.env"
    echo ""
    echo "  Before starting services, you must:"
    echo "  1. Copy deploy/.env.example to ${DEPLOY_DIR}/.env on the host"
    echo "  2. Fill in all required values (bot token, SMTP credentials, etc.)"
    echo "  3. Run: chmod 600 ${DEPLOY_DIR}/.env"
    echo "  4. Run: chown ${SERVICE_USER}:${SERVICE_USER} ${DEPLOY_DIR}/.env"
    echo ""
    echo "  Then re-run this script or start services manually:"
    echo "    systemctl daemon-reload"
    echo "    systemctl enable --now telegram-message-store"
    echo "    systemctl enable --now telegram-digest.timer"
    echo ""
    exit 0
fi

# ──────────────────────────────────────────────
# Step 6: Reload systemd and enable services
# ──────────────────────────────────────────────
step "Reloading systemd and enabling services..."

ssh_run "systemctl daemon-reload"
ssh_run "systemctl enable --now telegram-message-store"
ssh_run "systemctl enable --now telegram-digest.timer"

# Restart the long-running service to pick up new binary
ssh_run "systemctl restart telegram-message-store"

# ──────────────────────────────────────────────
# Step 7: Verify status
# ──────────────────────────────────────────────
step "Verifying service status..."

echo ""
ssh_run "systemctl status telegram-message-store --no-pager --lines=5 || true"
echo ""
ssh_run "systemctl list-timers --all | grep digest || true"

echo ""
echo "======================================================"
echo "  Deployment complete!"
echo ""
echo "  Useful commands:"
echo "    journalctl -u telegram-message-store -f"
echo "    journalctl -u telegram-digest -n 50"
echo "    systemctl status telegram-message-store"
echo "    systemctl start telegram-digest.service  # manual trigger"
echo "======================================================"
