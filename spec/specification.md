# Specification

## Goal

Give every TTS engine hosted by zallama text it can read: numbers, times,
dates, amounts, units, abbreviations and chat Markdown turned into words,
with the user's own pronunciations for new words, fast enough to sit in
front of live speech, in one dependency-free binary.

## Context and users

TTS models read digits and symbols poorly: Pocket TTS turns "9h30" into
noise, Kokoro's phonemizer reads simple numbers but not times, currencies or
Markdown, and chat answers are full of all three. Rule systems are exact but
miss context; small LLMs read context but sometimes change values.

Users:
- **zallama integrator**: runs `tn-server` as a backend and calls it before
  each TTS request; needs `/health`, `/v1/normalize`, low latency.
- **Assistant developer**: sends LLM answers to speech and wants them read
  correctly, Markdown included.
- **Lexicon maintainer**: adds new words (brands, names) and hears the
  result without restarting anything.

## Priorities

- **Must (core, MVP)**: French and English rules, Markdown handling, safe
  mode, lexicon, HTTP API, < 10 ms per paragraph.
- **Should**: lexicon reload without restart, batch requests.
- **Could**: other languages' rules, Roman numerals, grammar-based
  classification.

## Functional requirements

| ID | Requirement | Priority |
|---|---|---|
| REQ-TXT-001 | French and English numbers, decimals, negatives, times, dates, amounts, percentages, temperatures, units, ordinals, phone numbers, years (English), abbreviations and letter-glued codes are spelled out. | Must |
| REQ-TXT-002 | Markdown (headings, lists, emphasis, tables, code blocks, links), emoji and line breaks become plain sentences in every language. | Must |
| REQ-TXT-003 | Text with nothing to normalize comes out unchanged. | Must |
| REQ-MOD-001 | Safe mode leaves numbers the rules cannot read for sure (glued to letters, digit chains, parenthesized) as digits and normalizes the rest. | Must |
| REQ-LNG-001 | Languages are given as codes or names (`fr`, `fr-FR`, `french`, `english_2026-04`); unknown ones get the language-independent handling. | Must |
| REQ-LEX-001 | A lexicon (word → respelling, per language or all) is applied before the rules, case-insensitive, on whole words, longest key first, language-specific entries overriding general ones. | Must |
| REQ-LEX-002 | Lexicon errors name the faulty line. | Should |
| REQ-LEX-003 | The server reloads the lexicon when the file changes; a broken edit keeps the previous version. | Should |
| REQ-API-001 | HTTP API: `GET /health`, `POST /v1/normalize` with `text` or `texts`, `language`, `mode`; JSON errors with status 400. | Must |
| REQ-PRF-001 | A 3 600-character chat text is normalized in well under 10 ms (release build). | Must |

## Technical specification

### Architecture

Cargo workspace, two crates:

- `crates/tn` (library): `rules.rs` (regex rules per language, Markdown),
  `lexicon.rs` (parsing, matching), `lib.rs` (`Lang`, `Mode`,
  `normalize_text`: lexicon then rules).
- `crates/tn-server` (binary `tn-server`): `serve` (axum HTTP API,
  `lib.rs` router and lexicon reload) and `normalize` (CLI, text or stdin).

Pipeline per text: lexicon → Markdown to sentences → language rules
(abbreviations → times → ordinals → amounts → percentages / degrees / units →
dates → remaining numbers, each with its context check in safe mode) →
whitespace cleanup.

### Modules

| Module | Role |
|---|---|
| `crates/tn/src/lib.rs` | `Lang` (codes and names), `Mode`, `normalize_text` pipeline |
| `crates/tn/src/rules.rs` | Markdown to sentences; French and English passes; safe-mode `context()` |
| `crates/tn/src/lexicon.rs` | TSV parsing with line errors; per-language matching regex |
| `crates/tn-server/src/lib.rs` | axum router, request decoding, lexicon reload by modification time |
| `crates/tn-server/src/main.rs` | `serve` and `normalize` commands |

### Data

- Lexicon: TSV file chosen by the user (`--lexicon`), reloaded on
  modification time change. No other state, no database, no network access.

### APIs and commands

- CLI: `tn-server serve | normalize` (`--help`).
- HTTP: `GET /health`, `POST /v1/normalize` — `docs/api.md`.
- Rust: `tn::normalize_text`, `tn::normalize`, `tn::normalize_safe`,
  `tn::Lexicon`.

### Platforms

Pure Rust: Linux, macOS, Windows; x86_64 and aarch64. No GPU.

## Rules

- A rule ships only with a test of its exact output; plain text must stay
  unchanged (REQ-TXT-003).
- When a reading depends on context the rules cannot see, leave it to safe
  mode rather than guess (REQ-MOD-001).

## Known limitations

- Rules for French and English only.
- Roman numerals, English mm/dd without a year, homographs: not resolved by
  the rules (see the language-model pass, `docs/llm-pass.md`).
