# HTTP API

`tn-server serve [--host 127.0.0.1] [--port 8090] [--lexicon file.tsv]`

## `GET /health`

```json
{"status": "healthy", "version": "0.1.0", "lexicon_entries": 12}
```

`503` with `{"error": {"message"}}` if the lexicon file can no longer be read.

## `POST /v1/normalize`

| Field | Type | Default | |
|---|---|---|---|
| `text` | string | — | text to normalize (or `texts`) |
| `texts` | list of strings | — | batch; the response has `texts` in the same order |
| `language` | string | `en` | `fr`, `en`, `fr-FR`, `french`, `english_2026-04`…; other languages get Markdown, emoji and lexicon handling only |
| `mode` | `strict` \| `safe` | `strict` | `safe` leaves ambiguous numbers (codes, phone chains, `(555)`) as digits |

```bash
curl -s localhost:8090/v1/normalize -H 'content-type: application/json' \
  -d '{"texts":["Gate 12B at 9:30 am.","Up 12%."],"language":"en","mode":"safe"}'
# {"language":"en","mode":"safe","texts":["Gate 12B at nine thirty a m.","Up twelve percent."]}
```

Errors: `400` `{"error": {"message"}}` when neither or both of `text` and
`texts` are given, or `mode` is unknown.

Line breaks matter: each non-empty line becomes a sentence (a period is
added when it ends without punctuation), so send chat text with its line
breaks.
