# AGENTS.md

Guidance for AI agents working in this repository.

## Project

Text normalization for TTS: written text → words a speaker says. Library
`tn` + binary `tn-server` (HTTP API and CLI). Used by zallama in front of
its TTS engines and by pocket-tts-rs.

## Where things are

- `crates/tn/src/rules.rs`: Markdown handling, then per-language passes
  (abbreviations → times → ordinals → money → %/°/units → dates → numbers).
  `context()` decides what safe mode leaves as digits.
- `crates/tn/src/lexicon.rs`: TSV lexicon, one regex per language.
- `crates/tn-server/src/lib.rs`: router, lexicon reload by mtime.

## Commands

```bash
cargo test --workspace
tests/e2e.sh
target/release/tn-server normalize --lang fr "…"
python3 scripts/eval/llm_bench.py http://localhost:6767 @rules @hybrid:<model>
```

## Rules

- Rules must be exact: each change gets a test of its output; plain text
  must stay byte-identical (`plain_text_unchanged`).
- Never guess a context-dependent reading in safe mode; leave the digits.
- Keep the binary dependency-free at run time (no model, no network).
