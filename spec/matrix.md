# Traceability matrix

| Requirement | Feature | Area | Code | Tests | Status | Notes |
|---|---|---|---|---|---|---|
| REQ-TXT-001 | French/English rules | Rules | `crates/tn/src/rules.rs` | `french_cardinals`, `french_text`, `english_text`, `dates_and_glued_numbers`, `normalizes_text_and_batches`, `tests/e2e.sh` | ✅ | |
| REQ-TXT-004 | Addresses and symbols | Rules | `rules.rs` (`web_and_symbols`, `FR_DOT_VERSION`) | `addresses_ips_ranges_and_slashes`, `french_versions_and_dot_thousands` | ✅ | found on a chat answer that Pocket TTS turned to noise |
| REQ-TXT-002 | Markdown and lines | Rules | `rules.rs` (`markdown_to_sentences`) | `markdown`, `shouting_arrows_and_line_ends` | ✅ | |
| REQ-TXT-003 | Plain text unchanged | Rules | `rules.rs` | `plain_text_unchanged` | ✅ | |
| REQ-TXT-005 | Signs, HT/TTC | Rules | `rules.rs` (`sign_word`, `FR_ABBR`) | `signs_and_tax_abbreviations` | ✅ | `(+20 %)` was read "vingt pour cent" |
| REQ-TXT-006 | Roman numerals in context | Rules | `crates/tn/src/roman.rs` | `roman_numerals_in_context`, `canonical_values` | ✅ | no false positive on the zallama / tool docs |
| REQ-MOD-001 | Safe mode | Rules | `rules.rs` (`context`, `normalize_safe`), `lib.rs` (`Mode`) | `safe_mode_leaves_ambiguous_numbers`, `modes` | ✅ | |
| REQ-LNG-001 | Language codes | Library | `lib.rs` (`Lang::from_code`) | `language_codes` | ✅ | |
| REQ-LEX-001 | Lexicon | Lexicon | `crates/tn/src/lexicon.rs` | `whole_words_case_insensitive`, `language_specific_overrides_all`, `symbol_keys_and_longest_first`, `lexicon_runs_before_rules`, `lexicon_is_reloaded_when_edited` | ✅ | |
| REQ-LEX-002 | Lexicon errors | Lexicon | `lexicon.rs` (`parse`) | `errors_name_the_line` | ✅ | |
| REQ-LEX-003 | Lexicon reload | Server | `crates/tn-server/src/lib.rs` (`LexiconSource`) | `lexicon_is_reloaded_when_edited`, `tests/e2e.sh` | ✅ | |
| REQ-API-001 | HTTP API | Server | `crates/tn-server/src/lib.rs` | `health_reports_version`, `normalizes_text_and_batches`, `rejects_bad_requests`, `tests/e2e.sh` | ✅ | |
| REQ-PRF-001 | Speed | Rules | `rules.rs` | `fast_enough_for_live_speech`, manual MT-02 | ✅ | ~1 ms release |
