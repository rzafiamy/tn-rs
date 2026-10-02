# Manual tests

Results of 2026-10-02 (v0.1.0), i9-14900K, RTX 4090, Ubuntu 24.04.

## MT-01 — Normalization quality against references (REQ-TXT-001, REQ-MOD-001)

```bash
cargo build --release
TN_BIN=target/release/tn-server python3 scripts/eval/llm_bench.py http://localhost:6767 @rules @safe @hybrid:<llm>
```

Expected: no dropped words; strict WER ≤ 6 %. Result: ✅ strict 6.0 %, safe +
Gemma-4-E2B QAT 1.0 %, 0 dropped words (`docs/llm-pass.md`).

## MT-02 — Speed (REQ-PRF-001)

```bash
python3 - <<'PY'
import subprocess, time
text = open("README.md").read() * 4
t = time.time()
subprocess.run(["target/release/tn-server", "normalize", "--lang", "fr", text], capture_output=True)
print(f"{(time.time()-t)*1000:.1f} ms for {len(text)} chars (process start included)")
PY
```

Expected: < 50 ms with process start. Result: ✅ 9.5 ms for 25 568 characters, process start included.

## MT-03 — Listening test through a TTS engine (REQ-TXT-001, REQ-TXT-002)

Send a chat answer with Markdown, times, amounts and a date through zallama's
`/v1/audio/speech` with and without the normalizer and transcribe both with
Parakeet. Expected: numbers read correctly only with the normalizer.
Result: ✅ Pocket TTS: garbled numbers without, correct with (zallama
CHANGELOG 1.26.1).
