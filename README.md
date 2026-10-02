# tn-rs

Text normalization for speech synthesis: turns written text into the words a
speaker would say, so a TTS engine never has to guess how to read `9h30`,
`1 250 000 €`, `21/10/2026`, `Mme`, `$5.50` or a Markdown list. One small
Rust binary (`tn-server`, CLI and HTTP API), no model, no GPU, about 1 ms per
paragraph.

```text
## Votre semaine                       Votre semaine.
1. **Réunion** lundi à 9h30 :     →    Réunion lundi à neuf heures trente : budget d'un
   budget de 1 250 000 €.              million deux cent cinquante mille euros.
- Il fera 18 °C le 21/10/2026.         Il fera dix-huit degrés le vingt et un octobre
                                       deux mille vingt-six.
```

Built for the TTS engines hosted by
[zallama](https://github.com/rzafiamy/zallama) (Pocket TTS, Kokoro,
Voxtral), which read digits and symbols poorly or not at all. Overview:
[portfolio/](portfolio/README.md).

## Features

Core (must work, covered by tests — see [spec/matrix.md](spec/matrix.md)):

- **French and English rules** (`crates/tn/src/rules.rs`): cardinals,
  decimals, negative numbers, times (`9h30`, `21:05`, `9:30 am`), dates
  (`21/10/2026`; `10/21/2026` in English), amounts (`€`, `$`, `£`, cents,
  "deux millions d'euros"), `%`, `°C`/`°F`, units (km, kg, km/h, Go, kWh…),
  ordinals (`1er`, `3e`, `21st`), phone numbers read in pairs (French),
  English years ("nineteen eighty-four"), abbreviations (`Mme`, `M.`, `Dr`,
  `Mr.`, `e.g.`, `n°`), codes glued to letters (`221B`, `Q2`).
- **Markdown and chat text, every language**: headings, lists, emphasis,
  tables, code blocks, links and emoji become plain sentences, one per line,
  so the engine pauses where the reader would.
- **Pronunciation lexicon** (`crates/tn/src/lexicon.rs`): your words first —
  recent terms, names, brands, acronyms (`ChatGPT`, `Nvidia`, `RTX`) — per
  language or for all, case-insensitive, whole words, reloaded when the file
  changes.
- **Safe mode**: numbers the rules cannot be sure of (codes, phone chains,
  `(555)`) stay as digits for a language-model pass, which then never sees —
  and never rewrites — the amounts the rules got right
  ([docs/llm-pass.md](docs/llm-pass.md)).
- **HTTP API**: `POST /v1/normalize` (single text or batch), `GET /health`
  (`crates/tn-server/src/lib.rs`); CLI `normalize` and `serve`
  (`crates/tn-server/src/main.rs`); library entry point `tn::normalize_text`
  (`crates/tn/src/lib.rs`).

Secondary: other languages get the Markdown, emoji and lexicon handling only.

## Installation

### Prerequisites

- Linux, macOS or Windows (Git Bash); a C linker (`build-essential`, Xcode
  Command Line Tools or Visual Studio Build Tools), git, curl.
- Rust 1.97.1 (pinned in `rust-toolchain.toml`, installed by rustup).
- Optional: Python 3 for `scripts/eval/llm_bench.py`.

`./prereq.sh` checks all of this and installs Rust and the build packages
when missing (`CHECK_ONLY=1 ./prereq.sh` only checks).

### Commands

```bash
./setup.sh      # prerequisites, cargo fetch, cargo check
./build.sh      # build/tn-server-<os>-<arch>-<version>
```

### Check the installation

```bash
build/tn-server-linux-x86_64-0.1.0 --version
build/tn-server-linux-x86_64-0.1.0 normalize --lang fr "Rendez-vous à 9h30."
tests/e2e.sh build/tn-server-linux-x86_64-0.1.0     # CLI, lexicon, HTTP API
```

## Usage

```bash
tn-server normalize --lang fr "Le 1er mai, 3,5 % de hausse."   # → Le premier mai, trois virgule cinq pour cent de hausse.
echo "Gate 12B at 5 pm." | tn-server normalize --lang en --safe  # → Gate 12B at five p m.
tn-server normalize --lang fr -l lexicon.tsv "Nvidia sort la RTX 5090."

tn-server serve --port 8090 -l lexicon.tsv
curl -s localhost:8090/v1/normalize -H 'content-type: application/json' \
  -d '{"text":"Réunion à 14h15, salle n° 204.","language":"fr"}'
# {"language":"fr","mode":"strict","text":"Réunion à quatorze heures quinze, salle numéro deux cent quatre."}
```

API details: [docs/api.md](docs/api.md). Lexicon format:
[docs/lexicon.md](docs/lexicon.md). Rust library: the `tn` crate
(`tn::normalize_text(text, Lang::Fr, Mode::Strict, Some(&lexicon))`).

## Configuration

**Location**: there is no configuration file. `tn-server` reads
command-line options or environment variables (template:
[`tn.example.env`](tn.example.env), copy to `.env` or export); options take
precedence. The only file it reads is the lexicon you point it to
(`--lexicon`, `TN_LEXICON`), re-read automatically when it changes. To
change any other setting, change the option or variable and restart the
process.

In production, keep the lexicon at a fixed path and point `TN_LEXICON` (or
`--lexicon`) to it; suggested locations: `~/.config/tn/lexicon.tsv` on Linux,
`~/Library/Application Support/tn/lexicon.tsv` on macOS,
`%APPDATA%\tn\lexicon.tsv` on Windows. Under zallama the lexicon is the
registry entry's `file` (in `models_dir`, e.g. `/bank2/zallama/models/tn-lexicon.tsv`);
edit it in place, it applies to the next request.

| Option / variable | Role | Default |
|---|---|---|
| `--host`, `TN_HOST` | Bind address (`serve`) | `127.0.0.1` |
| `--port`, `TN_PORT` | Port (`serve`) | `8090` |
| `--lexicon`, `-l`, `TN_LEXICON` | Lexicon TSV file | none |
| `--lang` | Language (`normalize`): `fr`, `en`, `french`, `en-US`… | `en` |
| `--safe` | Safe mode (`normalize`) | strict |
| `RUST_LOG` | Log level | `info` |

**Verify** a configuration: `GET /health` reports the version and the
number of lexicon entries; a lexicon with an error is reported with its line
number (at startup it stops the server, later the previous version is kept).

## Tests

```bash
cargo test --workspace                               # unit + API tests
tests/e2e.sh                                         # release binary end to end
python3 scripts/eval/llm_bench.py <url> @rules ...   # quality benchmark (docs/llm-pass.md)
```

Requirements, traceability and manual tests: [spec/](spec/specification.md).

## Known limitations

- Rules exist for French and English only; other languages get Markdown,
  emoji and lexicon handling.
- Context-dependent readings are not resolved: Roman numerals (`Louis XIV`,
  `XVIIe siècle`), `3/4` without a year in English, homographs. The
  language-model pass handles those ([docs/llm-pass.md](docs/llm-pass.md)).
- English phone numbers and street numbers are read as cardinals in strict
  mode; use safe mode plus the model pass.
- A very large lexicon (tens of thousands of entries) makes the matching
  regex slow to build; it is built once per change.

## Roadmap

Details: [TODO.md](TODO.md).

- Roman numerals and more abbreviations in the rules.
- German, Spanish, Italian, Portuguese rules.
- Grammar-based classification (NeMo-style WFST) for context-dependent cases.

## Project

- [CHANGELOG.md](CHANGELOG.md) · [CREDITS.md](CREDITS.md) · [CONTRIBUTING.md](CONTRIBUTING.md)
- [spec/specification.md](spec/specification.md) · [spec/matrix.md](spec/matrix.md) · [spec/manual-tests.md](spec/manual-tests.md)
- [portfolio/](portfolio/README.md)
- Issues and suggestions: https://github.com/rzafiamy/tn-rs/issues

## License

MIT (see [LICENSE](LICENSE)).
