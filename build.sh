#!/usr/bin/env bash
# build.sh — builds the release `tn-server` binary into
# build/tn-server-<os>-<arch>-<version>[.exe]
# (Linux, macOS, Windows through Git Bash/MSYS2; x86_64 or aarch64).
#
#   ./build.sh
#
# Non-interactive (CI friendly); writes only to target/ and build/.
set -euo pipefail
cd "$(dirname "$0")"
export PATH="$HOME/.cargo/bin:$PATH"
command -v cargo >/dev/null || { echo "cargo not found: run ./prereq.sh (installs Rust)" >&2; exit 1; }
trap 'echo "build failed — run ./setup.sh to check prerequisites" >&2' ERR
VERSION=$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)
cargo build --release --locked -p tn-server
case "$(uname -s)" in
  Linux) OS=linux; EXE= ;;
  Darwin) OS=macos; EXE= ;;
  MINGW*|MSYS*|CYGWIN*) OS=windows; EXE=.exe ;;
  *) OS=$(uname -s | tr '[:upper:]' '[:lower:]'); EXE= ;;
esac
ARCH=$(uname -m); [ "$ARCH" = arm64 ] && ARCH=aarch64
mkdir -p build
OUT="build/tn-server-$OS-$ARCH-$VERSION$EXE"
cp "target/release/tn-server$EXE" "$OUT"
"$OUT" --version
echo "artifact: $OUT"
