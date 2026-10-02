//! User pronunciation lexicon: words the rules and the TTS model cannot know
//! (recent words, names, brands, acronyms) mapped to a respelling.
//!
//! Format: UTF-8 text, one entry per line, tab-separated:
//!
//! ```text
//! # comment
//! ChatGPT<TAB>tchatte gé pé té        all languages
//! fr<TAB>Nvidia<TAB>ène vidia         French only
//! en<TAB>Nvidia<TAB>en vidia          English only
//! *<TAB>RTX<TAB>R T X                 all languages (explicit)
//! ```
//!
//! Matching is case-insensitive on whole words (a key may contain spaces,
//! dots or symbols: `e.g.`, `C++`, `Md€`); longer keys win over shorter ones.
//! A language-specific entry wins over an all-languages entry for the same key.

use crate::Lang;
use regex::{Regex, RegexBuilder};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Lexicon {
    by_lang: HashMap<Lang, Table>,
    len: usize,
}

#[derive(Debug, Clone)]
struct Table {
    re: Regex,
    map: HashMap<String, String>,
}

const LANGS: [Lang; 3] = [Lang::Fr, Lang::En, Lang::Other];

impl Lexicon {
    /// Parses the lexicon format; errors name the faulty line.
    pub fn parse(text: &str) -> Result<Self, String> {
        // (lang, key, respelling); lang None = every language
        let mut entries: Vec<(Option<Lang>, String, String)> = Vec::new();
        for (i, line) in text.lines().enumerate() {
            let line = line.trim_end_matches('\r');
            if line.trim().is_empty() || line.trim_start().starts_with('#') {
                continue;
            }
            let fields: Vec<&str> = line.split('\t').map(str::trim).collect();
            let (lang, key, value) = match fields.as_slice() {
                [k, v] => (None, *k, *v),
                ["*", k, v] => (None, *k, *v),
                [l, k, v] => (Some(Lang::from_code(l)), *k, *v),
                _ => {
                    return Err(format!(
                        "line {}: expected `word<TAB>respelling` or `lang<TAB>word<TAB>respelling`",
                        i + 1
                    ));
                }
            };
            if key.is_empty() || value.is_empty() {
                return Err(format!("line {}: empty word or respelling", i + 1));
            }
            entries.push((lang, key.to_string(), value.to_string()));
        }
        let len = entries.len();
        let mut by_lang = HashMap::new();
        for lang in LANGS {
            let mut map: HashMap<String, String> = HashMap::new();
            // all-languages entries first, so language-specific ones override
            for (l, k, v) in entries.iter().filter(|e| e.0.is_none()) {
                debug_assert!(l.is_none());
                map.insert(k.to_lowercase(), v.clone());
            }
            for (_, k, v) in entries.iter().filter(|e| e.0 == Some(lang)) {
                map.insert(k.to_lowercase(), v.clone());
            }
            if map.is_empty() {
                continue;
            }
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort_by_key(|k| std::cmp::Reverse(k.chars().count()));
            let alternation = keys
                .iter()
                .map(|k| regex::escape(k))
                .collect::<Vec<_>>()
                .join("|");
            let re = RegexBuilder::new(&alternation)
                .case_insensitive(true)
                .build()
                .map_err(|e| format!("lexicon too large or invalid: {e}"))?;
            by_lang.insert(lang, Table { re, map });
        }
        Ok(Lexicon { by_lang, len })
    }

    /// Reads and parses a lexicon file.
    pub fn load(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("cannot read lexicon {}: {e}", path.display()))?;
        Self::parse(&text).map_err(|e| format!("{}: {e}", path.display()))
    }

    /// Number of entries in the file (all languages).
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Replaces every whole-word occurrence of a key for `lang`.
    pub fn apply(&self, text: &str, lang: Lang) -> String {
        let Some(table) = self.by_lang.get(&lang) else {
            return text.to_string();
        };
        let mut out = String::with_capacity(text.len());
        let mut last = 0;
        for m in table.re.find_iter(text) {
            let before = text[..m.start()].chars().next_back();
            let after = text[m.end()..].chars().next();
            let key = m.as_str();
            // a key's own edge may be a symbol ("Md€", "e.g."); only a word
            // character glued to a word-character edge breaks the match
            let glued_before =
                key.chars().next().is_some_and(is_word) && before.is_some_and(is_word);
            let glued_after =
                key.chars().next_back().is_some_and(is_word) && after.is_some_and(is_word);
            if glued_before || glued_after {
                continue;
            }
            if let Some(value) = table.map.get(&key.to_lowercase()) {
                out.push_str(&text[last..m.start()]);
                out.push_str(value);
                last = m.end();
            }
        }
        out.push_str(&text[last..]);
        out
    }
}

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

#[cfg(test)]
mod tests {
    use super::*;

    /// covers: REQ-LEX-001
    #[test]
    fn whole_words_case_insensitive() {
        let lex = Lexicon::parse("ChatGPT\ttchatte gé pé té\nGPT\tgé pé té\n").unwrap();
        assert_eq!(
            lex.apply("J'utilise chatgpt et GPT, pas GPTs ni xGPT.", Lang::Fr),
            "J'utilise tchatte gé pé té et gé pé té, pas GPTs ni xGPT."
        );
    }

    /// covers: REQ-LEX-001
    #[test]
    fn language_specific_overrides_all() {
        let lex = Lexicon::parse("# brands\nNvidia\ten vidia\nfr\tNvidia\tène vidia\n").unwrap();
        assert_eq!(lex.apply("Nvidia.", Lang::Fr), "ène vidia.");
        assert_eq!(lex.apply("Nvidia.", Lang::En), "en vidia.");
        assert_eq!(lex.len(), 2);
    }

    /// covers: REQ-LEX-001
    #[test]
    fn symbol_keys_and_longest_first() {
        let lex = Lexicon::parse("C\tcé\nC++\tcé plus plus\ne.g.\tfor example\n").unwrap();
        assert_eq!(
            lex.apply("C++ and C, e.g. here", Lang::En),
            "cé plus plus and cé, for example here"
        );
    }

    /// covers: REQ-LEX-002
    #[test]
    fn errors_name_the_line() {
        let err = Lexicon::parse("ok\tok\nbroken line\n").unwrap_err();
        assert!(err.contains("line 2"), "{err}");
        let err = Lexicon::parse("fr\t\tvalue\n").unwrap_err();
        assert!(err.contains("line 1"), "{err}");
    }
}
