#!/bin/bash
# caveman — node-free installer for Claude Code.
#
# Builds the native Rust `caveman` binary (no Node, ever) and wires the
# SessionStart + UserPromptSubmit hooks and the statusline badge into
# settings.json.
#
# Usage:
#   bash install.sh            # build + install
#   bash install.sh --force    # rebuild + re-wire over an existing install
#   bash install.sh --uninstall
#
# Requires the Rust toolchain (cargo). Install once from https://rustup.rs.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")" && pwd)"
MANIFEST="$SCRIPT_DIR/rust/Cargo.toml"

FORCE=""
UNINSTALL=0
for arg in "$@"; do
  case "$arg" in
    --force|-f) FORCE="--force" ;;
    --uninstall) UNINSTALL=1 ;;
  esac
done

# Make cargo reachable even if the user has not reopened their shell since
# installing rustup.
if ! command -v cargo >/dev/null 2>&1; then
  if [ -f "$HOME/.cargo/env" ]; then
    # shellcheck disable=SC1091
    . "$HOME/.cargo/env"
  fi
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "ERROR: 'cargo' (Rust toolchain) not found."
  echo "       caveman is now native Rust — no Node required."
  echo "       Install Rust once: https://rustup.rs"
  echo "         curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
  echo "       Then re-run: bash install.sh"
  exit 1
fi

if [ ! -f "$MANIFEST" ]; then
  echo "ERROR: cannot find $MANIFEST — run this script from a caveman checkout."
  exit 1
fi

echo "Building caveman (release)..."
cargo build --release --manifest-path "$MANIFEST"

BIN="$SCRIPT_DIR/rust/target/release/caveman"
if [ ! -x "$BIN" ]; then
  echo "ERROR: build did not produce $BIN"
  exit 1
fi

if [ "$UNINSTALL" -eq 1 ]; then
  "$BIN" uninstall
  exit 0
fi

echo ""
"$BIN" install $FORCE

echo ""
echo "What's installed (all native, no Node):"
echo "  - SessionStart hook: auto-loads caveman rules every session"
echo "  - UserPromptSubmit hook: tracks mode + injects per-turn reinforcement"
echo "  - Statusline badge: shows [CAVEMAN] / [CAVEMAN:ULTRA] etc."
