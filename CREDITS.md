# Credits

tn-rs is distributed under MIT. It builds on the work below, each used under
its own license. Versions are those of `Cargo.lock`.

## Authors

- Rija Z. ([@rzafiamy](https://github.com/rzafiamy)) — design, rules,
  integration with zallama.
- Developed with the assistance of Claude (Anthropic) through Claude Code.

## Third-party sources

- No third-party data or grammar is bundled. The rules are original; the
  evaluation compared them with NVIDIA NeMo text processing (Apache-2.0),
  which is not a dependency.
- Language models named in `docs/llm-pass.md` are only used through an
  external OpenAI-compatible server; none is distributed here.

## Rust dependencies

| Dependency | Version | Role | License | Source |
|---|---|---|---|---|
| `regex` | 1.13.1 | Rules and lexicon matching | MIT OR Apache-2.0 | https://github.com/rust-lang/regex |
| `axum` | 0.7.9 | HTTP server | MIT | https://github.com/tokio-rs/axum |
| `tokio` | 1.53.1 | Async runtime | MIT | https://github.com/tokio-rs/tokio |
| `serde` | 1.0.229 | Request decoding | MIT OR Apache-2.0 | https://github.com/serde-rs/serde |
| `serde_json` | 1.0.151 | JSON | MIT OR Apache-2.0 | https://github.com/serde-rs/json |
| `clap` | 4.6.7 | Command line | MIT OR Apache-2.0 | https://github.com/clap-rs/clap |
| `tracing` | 0.1.44 | Logging | MIT | https://github.com/tokio-rs/tracing |
| `tracing-subscriber` | 0.3.23 | Log output | MIT | https://github.com/tokio-rs/tracing |
| `anyhow` | 1.0.104 | Errors | MIT OR Apache-2.0 | https://github.com/dtolnay/anyhow |
| `tower` (tests) | 0.4.13 | API tests | MIT | https://github.com/tower-rs/tower |

No JavaScript/TypeScript dependencies, no Tauri libraries, no icons or fonts.
All licenses above are compatible with MIT distribution.
