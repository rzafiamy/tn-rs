#!/usr/bin/env bash
# tests/e2e.sh — end-to-end check of the release binary: CLI normalization,
# lexicon, and the HTTP server (/health, /v1/normalize, lexicon reload).
#
#   tests/e2e.sh [binary]      default: target/release/tn-server
#
# covers: REQ-TXT-001, REQ-LEX-001, REQ-LEX-003, REQ-API-001
set -euo pipefail
cd "$(dirname "$0")/.."
BIN="${1:-target/release/tn-server}"
[ -x "$BIN" ] || { echo "binary not found: $BIN (run ./build.sh or cargo build --release)" >&2; exit 1; }
WORK=$(mktemp -d)
PORT=${PORT:-18990}
SERVER=""
cleanup() { [ -n "$SERVER" ] && kill "$SERVER" 2>/dev/null; rm -rf "$WORK"; }
trap cleanup EXIT
fail() { echo "FAIL: $*" >&2; exit 1; }
expect() { [ "$1" = "$2" ] || fail "expected [$2], got [$1]"; }

echo "== CLI"
expect "$("$BIN" normalize --lang fr "Rendez-vous à 9h30, 12,99 €.")" \
  "Rendez-vous à neuf heures trente, douze euros quatre-vingt-dix-neuf."
expect "$(echo "Gate 12B at 5 pm." | "$BIN" normalize --lang en --safe)" "Gate 12B at five p m."

echo "== lexicon"
printf 'fr\tNvidia\tène vidia\n' > "$WORK/lex.tsv"
expect "$("$BIN" normalize --lang fr -l "$WORK/lex.tsv" "Nvidia vend 3 GPU.")" "ène vidia vend trois GPU."

echo "== serve"
"$BIN" serve --port "$PORT" -l "$WORK/lex.tsv" >"$WORK/server.log" 2>&1 &
SERVER=$!
for _ in $(seq 50); do curl -sf "localhost:$PORT/health" >/dev/null && break; sleep 0.1; done
curl -sf "localhost:$PORT/health" | grep -q '"lexicon_entries":1' || { cat "$WORK/server.log"; fail "health"; }
out=$(curl -sf "localhost:$PORT/v1/normalize" -H 'content-type: application/json' \
  -d '{"text":"Nvidia, le 21/10/2026.","language":"fr"}')
echo "$out" | grep -q '"text":"ène vidia, le vingt et un octobre deux mille vingt-six."' || fail "normalize: $out"
sleep 1.1; printf 'fr\tNvidia\tenne vidia\n' > "$WORK/lex.tsv"
out=$(curl -sf "localhost:$PORT/v1/normalize" -H 'content-type: application/json' -d '{"text":"Nvidia.","language":"fr"}')
echo "$out" | grep -q '"text":"enne vidia."' || fail "lexicon reload: $out"
code=$(curl -s -o /dev/null -w '%{http_code}' "localhost:$PORT/v1/normalize" -H 'content-type: application/json' -d '{"text":"x","mode":"loose"}')
expect "$code" 400

echo "e2e OK"
