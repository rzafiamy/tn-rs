# Pronunciation lexicon

Words the rules and the TTS model cannot know — recent terms, names, brands,
acronyms — mapped to how they should be read. Applied **before** the rules,
so a respelling may contain numbers or abbreviations the rules then expand.

## Format

UTF-8 text, one entry per line, fields separated by a **tab**:

```text
# comment
ChatGPT	tchatte gé pé té          ← all languages
fr	Nvidia	ène vidia             ← French only
en	Nvidia	en vidia              ← English only
*	RTX	R T X                     ← all languages (explicit)
```

- Language codes as in the API (`fr`, `en`, `fr-FR`, `french`); `*` or no
  code = all languages. A language-specific entry overrides an
  all-languages entry for the same word.
- Matching is case-insensitive and on whole words; a key may contain spaces,
  dots or symbols (`e.g.`, `C++`, `Md€`). Longer keys win (`C++` before `C`).
- Errors name the line (`lexicon.tsv: line 4: expected word<TAB>respelling…`).

Example file: [`../lexicon.example.tsv`](../lexicon.example.tsv).

## Editing a live lexicon

`tn-server serve -l lexicon.tsv` checks the file's modification time on each
request and reloads it when it changed. An edit that breaks the file is
logged and the previous version stays in use; `GET /health` reports the
entry count.

## Writing respellings

Write what the engine should *read*, in its language's spelling: `ène vidia`
rather than a phonetic alphabet. Spell letters with spaces (`R T X`). Test
with `tn-server normalize --lang fr -l lexicon.tsv "…"`, then listen.
