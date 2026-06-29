#!/bin/bash
# caveman — prebuilt-binary installer. No Rust. No build. No repo checkout.
#
# Downloads the statically-linked `caveman` binary from a GitHub Release and
# runs `caveman install`, which copies the binary into your Claude config dir
# and wires the hooks + statusline. The binary bakes in every skill + agent, so
# nothing else needs to be on disk.
#
# Usage:
#   bash install-release.sh                  # latest release, install
#   bash install-release.sh v0.2.0           # a specific release tag
#   bash install-release.sh latest --force   # re-wire over an existing install
#   curl -fsSL https://raw.githubusercontent.com/JuliusBrussee/caveman/main/install-release.sh | bash
#
# Env:
#   CAVEMAN_REPO   override the GitHub repo (default JuliusBrussee/caveman)
set -euo pipefail

REPO="${CAVEMAN_REPO:-JuliusBrussee/caveman}"

# First non-flag arg is the version; flags pass through to `caveman install`.
VERSION="latest"
INSTALL_ARGS=()
for arg in "$@"; do
  case "$arg" in
    -*) INSTALL_ARGS+=("$arg") ;;
    *)  VERSION="$arg" ;;
  esac
done

# --- platform detection -------------------------------------------------------
OS="$(uname -s)"
ARCH="$(uname -m)"
if [ "$OS" != "Linux" ]; then
  echo "ERROR: prebuilt binaries are published for Linux only right now (got $OS)."
  echo "       Build from source instead: bash install.sh"
  exit 1
fi
case "$ARCH" in
  x86_64|amd64) TARGET="x86_64-unknown-linux-musl" ;;
  *)
    echo "ERROR: no prebuilt binary for arch '$ARCH' (only x86_64)."
    echo "       Build from source instead: bash install.sh"
    exit 1
    ;;
esac

ASSET="caveman-$TARGET"
SUMS="caveman-$TARGET.sha256"
if [ "$VERSION" = "latest" ]; then
  BASE="https://github.com/$REPO/releases/latest/download"
else
  BASE="https://github.com/$REPO/releases/download/$VERSION"
fi

# --- download -----------------------------------------------------------------
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
BIN="$TMP/caveman"

echo "Downloading $ASSET ($VERSION) from $REPO ..."
curl -fSL "$BASE/$ASSET" -o "$BIN"

# --- verify checksum (best effort — fatal only if the file is present) --------
if curl -fsSL "$BASE/$SUMS" -o "$TMP/$SUMS" 2>/dev/null; then
  EXPECTED="$(grep " $ASSET\$" "$TMP/$SUMS" | awk '{print $1}')"
  if [ -n "$EXPECTED" ]; then
    ACTUAL="$(sha256sum "$BIN" | awk '{print $1}')"
    if [ "$EXPECTED" != "$ACTUAL" ]; then
      echo "ERROR: checksum mismatch for $ASSET"
      echo "  expected $EXPECTED"
      echo "  actual   $ACTUAL"
      exit 1
    fi
    echo "Checksum verified."
  fi
else
  echo "WARNING: no checksum file published for $VERSION — skipping verification."
fi

chmod +x "$BIN"

# --- install ------------------------------------------------------------------
echo ""
"$BIN" install "${INSTALL_ARGS[@]}"

echo ""
echo "Installed from prebuilt binary (no Rust, no rebuild):"
echo "  - SessionStart hook: auto-loads caveman rules every session"
echo "  - UserPromptSubmit hook: tracks mode + injects per-turn reinforcement"
echo "  - Statusline badge: shows [CAVEMAN] / [CAVEMAN:ULTRA] etc."
echo "  - Slash commands: /caveman, /caveman-commit, /caveman-review, ..."
