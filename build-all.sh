#!/usr/bin/env bash
# build-all.sh — build pkgtui for all platforms locally using `cross`
#
# Requirements:
#   - Docker running
#   - cargo install cross
#
# Usage:
#   ./build-all.sh              # build all targets
#   ./build-all.sh linux        # build Linux targets only
#   ./build-all.sh windows      # Windows only
#   ./build-all.sh macos        # macOS only (requires macOS host)

set -euo pipefail

BINARY="rust-tui"
OUT="dist"
mkdir -p "$OUT"

# ── Target definitions ────────────────────────────────────────────────────────
# Format: "target|output_name"
ALL_TARGETS=(
  "x86_64-unknown-linux-musl|${BINARY}-linux-x86_64"
  "aarch64-unknown-linux-musl|${BINARY}-linux-arm64"
  "x86_64-pc-windows-gnu|${BINARY}-windows-x86_64.exe"
  "aarch64-apple-darwin|${BINARY}-macos-arm64"
  "x86_64-apple-darwin|${BINARY}-macos-x86_64"
)

LINUX_TARGETS=(
  "x86_64-unknown-linux-musl|${BINARY}-linux-x86_64"
  "aarch64-unknown-linux-musl|${BINARY}-linux-arm64"
)

WINDOWS_TARGETS=(
  "x86_64-pc-windows-gnu|${BINARY}-windows-x86_64.exe"
)

MACOS_TARGETS=(
  "aarch64-apple-darwin|${BINARY}-macos-arm64"
  "x86_64-apple-darwin|${BINARY}-macos-x86_64"
)

# ── Select targets ────────────────────────────────────────────────────────────
case "${1:-all}" in
  linux)   TARGETS=("${LINUX_TARGETS[@]}") ;;
  windows) TARGETS=("${WINDOWS_TARGETS[@]}") ;;
  macos)   TARGETS=("${MACOS_TARGETS[@]}") ;;
  all)     TARGETS=("${ALL_TARGETS[@]}") ;;
  *)
    echo "Usage: $0 [all|linux|windows|macos]"
    exit 1
    ;;
esac

# ── Check dependencies ────────────────────────────────────────────────────────
if ! command -v cross &>/dev/null; then
  echo "error: 'cross' not found. Install it with:"
  echo "  cargo install cross"
  exit 1
fi

if ! docker info &>/dev/null; then
  echo "error: Docker is not running. cross requires Docker."
  exit 1
fi

# ── Build loop ────────────────────────────────────────────────────────────────
for entry in "${TARGETS[@]}"; do
  TARGET="${entry%%|*}"
  ASSET="${entry##*|}"

  echo ""
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo "  Building → $TARGET"
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

  cross build --release --target "$TARGET"

  # Locate the built binary (Windows has .exe)
  if [[ "$TARGET" == *windows* ]]; then
    SRC="target/$TARGET/release/${BINARY}.exe"
  else
    SRC="target/$TARGET/release/$BINARY"
  fi

  cp "$SRC" "$OUT/$ASSET"
  echo "  ✔  $OUT/$ASSET  ($(du -sh "$OUT/$ASSET" | cut -f1))"
done

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Done. Binaries in ./$OUT/"
ls -lh "$OUT/"