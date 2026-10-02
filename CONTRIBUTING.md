# Contributing

## Setup

```bash
./setup.sh      # prerequisites, dependencies, cargo check
./build.sh      # release binary in build/
```

## Repository

- `crates/tn`: library — `rules.rs` (per-language rules, Markdown),
  `lexicon.rs`, `lib.rs` (`Lang`, `Mode`, `normalize_text`).
- `crates/tn-server`: `tn-server` binary (HTTP API, CLI).
- `scripts/eval/`: quality benchmark (rules, LLMs, rules + LLM).
- `spec/`: requirements, traceability matrix, manual tests.
- `docs/`: API, lexicon, language-model pass.

## Before a pull request

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
tests/e2e.sh
```

- Every rule change comes with a test of its exact output, and
  `plain_text_unchanged` must still pass.
- When a reading depends on context, prefer leaving it to safe mode over
  guessing.
- New requirements get an ID in `spec/specification.md`, a row in
  `spec/matrix.md` and a `/// covers: REQ-…` comment on their test.
- Note user-visible changes in `CHANGELOG.md`.

## Coding agents

See [AGENTS.md](AGENTS.md).
