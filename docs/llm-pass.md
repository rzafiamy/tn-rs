# Language-model pass

The rules are exact on what they cover but cannot read context-dependent
forms (Roman numerals, street and phone numbers, abbreviated dates). A small
instruction-tuned LLM reads those well — and, alone, sometimes changes
values (`12,99 €` read "douze euros quatre-vingt-neuf"). The combination:

1. `tn` in **safe mode**: everything the rules can read for sure is spelled
   out; ambiguous numbers stay as digits.
2. The LLM, only on sentences that still contain digits, Roman numerals or
   abbreviations, with a few-shot prompt (`scripts/eval/llm_bench.py`).
3. A guard: if the LLM dropped words of the sentence, keep the strict-mode
   output for it.

zallama implements this chain for its TTS models (`params.normalizer`,
`params.normalizer_llm`).

## Benchmark (2026-10-02)

`scripts/eval/llm_bench.py`: 16 hand-referenced French and English sentences
(times, amounts, dates, phones, Roman numerals, units, Markdown). WER against
the references (several accepted readings), plain words of the input missing
from the output, latency per sentence, VRAM of the llama-server. RTX 4090,
llama.cpp through zallama, Q4 GGUF, context 4096, temperature 0, no reasoning.

| System | WER | Dropped words | ms / sentence | VRAM |
|---|---|---|---|---|
| tn strict | 6.0 % | 0 | ~1 (11 with process start) | 0 |
| **tn safe + Gemma-4-E2B QAT (UD-Q4_K_XL)** | **1.0 %** | **0** | ~190 | **1.79 GB** |
| tn safe + Gemma-4-E2B IQ4_XS | 1.9 % | 0 | ~200 | 1.87 GB |
| Gemma-4-E2B QAT alone | 5.3 % | 0 | 120 | 1.79 GB |
| Gemma-4-E2B IQ4_XS alone | 3.9 % | 0 | 124 | 1.87 GB |
| Ministral-3-3B-Instruct-2512 Q4_K_M | 9.3 % | 0 | 99 | 2.91 GB |
| Qwen3-4B-Instruct-2507 Q4_K_M | 12.7 % | 0 | 131 | 3.39 GB |
| granite-4.0-micro Q4_K_M | 14.7 % | 0 | 126 | 2.77 GB |
| Llama-3.2-3B-Instruct Q4_K_M | 16.6 % | 1 | 108 | 2.81 GB |
| Phi-4-mini-instruct Q4_K_M | 25.2 % | 5 | 111 | 3.31 GB |
| gemma-3-4b-it Q4_K_M | 36.7 % | 14 | 183 | 3.08 GB |
| SmolLM3-3B Q4_K_M | 37.2 % | 9 | 106 | 2.57 GB |
| LFM2.5-2.6B Q4_K_M | 88.3 % | 85 | 1173 | 2.12 GB |
| Qwen3-0.6B Q8_0 | — returns the input unchanged | | | |
| Qwen3.5-4B Q4_K_M MTP, no reasoning | — drops words, wrong values | | ~250 | 3.9 GB |
| Qwen3.5-4B Q4_K_M MTP, reasoning | — ~1 900 tokens per sentence, > 6 s | | | |

Gemma-4-E2B keeps its per-layer embeddings in system RAM, hence 1.8 GB of
VRAM for a 2.6 GB file. Gemma 4 multi-token prediction (separate
`mtp-gemma-4-E2B-it.gguf` head, `--spec-type draft-mtp`) works (37–72 %
draft acceptance) but saves ~5 %: answers are ~40 tokens and the few-shot
prompt prefill dominates.

Remaining errors of the best system: "Henry eight" (for "Henry the Eighth"),
"October twenty-one" (for "twenty-first").

NVIDIA NeMo text processing (WFST grammars) was also evaluated: English
grammars are strong (dates, addresses, phone numbers); the French grammar
leaves `9h30`, `€`, `°C` and `Mme` untouched.

## Heard through TTS engines (2026-10-02)

zallama `benchmarks/tts_normalization.py`: the 16 sentences spoken by each
engine (3 takes), transcribed by Parakeet TDT 0.6B v3, WER after both sides
go through tn strict (digit groups joined, so the ASR's own number
formatting is not counted). Pocket TTS on CPU, Kokoro with a voice per
language.

| Engine | No normalization | tn strict | tn safe + Gemma-4-E2B QAT |
|---|---|---|---|
| Pocket TTS French | 73.0 % | 14.1 % | **11.2 %** |
| Pocket TTS English | 35.1 % | 13.8 % | **7.4 %** |
| Kokoro French | 15.1 % | 8.5 % | **5.5 %** |
| Kokoro English | 8.9 % | 3.9 % | **1.5 %** |

Pocket TTS sometimes loops on long runs of repeated digits ("five five five
… four four four four"), whatever the normalizer; part of its remaining
error is that.
