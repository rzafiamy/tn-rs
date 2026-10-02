#!/usr/bin/env bash
# setup.sh — prepares a development checkout: prerequisites, Rust
# dependencies, compile check. Idempotent and non-interactive.
set -euo pipefail
cd "$(dirname "$0")"
./prereq.sh
export PATH="$HOME/.cargo/bin:$PATH"
for tool in cargo rustc git; do
  command -v "$tool" >/dev/null || { echo "missing tool: $tool (rerun ./prereq.sh)" >&2; exit 1; }
done
cargo fetch --locked || { echo "cargo fetch failed (network?)" >&2; exit 1; }
cargo check --workspace --locked
echo "setup OK — next: ./build.sh then build/tn-server-* normalize --lang fr \"Rendez-vous à 9h30.\""
