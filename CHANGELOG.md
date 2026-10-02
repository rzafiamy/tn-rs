# Changelog

Notable changes of tn-rs. Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
versions: [SemVer](https://semver.org/).

## [Unreleased]

## [0.2.0] - 2026-10-02

### Added
- E-mails, URLs and domains read as spoken ("noreply arobase imgam point
  ovh", "example dot com slash docs"); IPv4 addresses octet by octet;
  `102.x` wildcards; ranges between words or times (`06h51–06h52` → "à",
  `Mon–Fri` → "to"); `word/word` → "ou"/"or", spaced ` / ` → pause,
  `/path` → "slash path"; arrows (`→`, `->`, `=>`) → pause.
- French version numbers and dot thousands: `Qwen-Image 2.0` → "deux point
  zéro", `Python 3.11` → "trois point onze", `1.250.000` → a million…
- Words of 5+ capitals get a normal case (`RANDRIANARIZAKA` →
  Randrianarizaka) instead of being spelled letter by letter.

### Fixed
- A line ending in `)` or `»` without punctuation inside now gets its
  pause; such lines ran into the next one.

A chat answer about login logs (URL, e-mails, IP addresses, slashes) made
Pocket TTS read half of its two minutes as noise; these rules fix it.

## [0.1.0] - 2026-10-02

First release. The rules started in pocket-tts-rs (`normalize.rs`) and move
here so every TTS engine can use them.

### Added
- `tn` library: French and English rules (numbers, decimals, negatives,
  times, dates, amounts, percentages, temperatures, units, ordinals, phone
  numbers, English years, abbreviations, letter-glued codes, `de un` →
  `d'un`), Markdown/emoji/line handling for every language, strict and safe
  modes, language codes and names.
- Pronunciation lexicon (TSV, per language or all, case-insensitive whole
  words, longest key first, errors with line numbers).
- `tn-server`: `serve` (HTTP API `GET /health`, `POST /v1/normalize` with
  `text` or `texts`, lexicon reloaded when the file changes) and `normalize`
  (CLI, argument or standard input).
- `scripts/eval/llm_bench.py` and `docs/llm-pass.md`: benchmark of rules,
  small LLMs and rules + LLM (best: safe rules + Gemma-4-E2B QAT, 1.0 % WER,
  1.8 GB VRAM).
- `prereq.sh`, `setup.sh`, `build.sh`, `tests/e2e.sh`, spec, traceability
  matrix, manual tests, portfolio.

## [0.0.1] - 2026-10-02

Prototype inside pocket-tts-rs (`crates/pocket-tts/src/normalize.rs`,
commits 2419768 and e3939e7 there), not released on its own.

### Added
- French and English number, time, amount, unit and abbreviation rules;
  Markdown to sentences; safe mode; dates.

[Unreleased]: https://github.com/rzafiamy/tn-rs/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/rzafiamy/tn-rs/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/rzafiamy/tn-rs/releases/tag/v0.1.0
