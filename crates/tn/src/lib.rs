//! Text normalization for speech synthesis: turns written text into the
//! words a speaker would say, so a TTS model never has to guess how to read
//! "9h30", "1 250 000 €", "21/10/2026", "Mme" or Markdown.
//!
//! Two layers, applied in order by [`normalize_text`]:
//! 1. a [`Lexicon`] of user pronunciations (recent words, names, brands), and
//! 2. the [`rules`]: French and English numbers, times, dates, amounts, units,
//!    ordinals and abbreviations; Markdown, emoji and lines for every language.
//!
//! [`Mode::Safe`] leaves the numbers the rules cannot read with certainty
//! (codes like `221B`, phone chains, `(555)`) as digits, for a language
//! model pass that handles them without touching what the rules got right.

pub mod lexicon;
pub mod rules;

pub use lexicon::Lexicon;
pub use rules::{normalize, normalize_safe};

/// Language whose rules apply.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Lang {
    Fr,
    En,
    /// Markdown, emoji and line handling only.
    Other,
}

impl Lang {
    /// Parses a language code or name: `fr`, `fr-FR`, `fra`, `french`,
    /// `français`, `en`, `en_US`, `english`, `english_2026-04`, ...
    /// Anything else is [`Lang::Other`].
    pub fn from_code(code: &str) -> Self {
        let code = code.trim().to_lowercase();
        let head = code.split(['-', '_', '.', ' ']).next().unwrap_or("");
        match head {
            "fr" | "fra" | "fre" | "french" | "français" | "francais" => Lang::Fr,
            "en" | "eng" | "english" | "anglais" => Lang::En,
            _ => Lang::Other,
        }
    }

    /// ISO 639-1 code (`other` for [`Lang::Other`]).
    pub fn code(self) -> &'static str {
        match self {
            Lang::Fr => "fr",
            Lang::En => "en",
            Lang::Other => "other",
        }
    }
}

/// How numbers the rules cannot be sure of are treated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    /// Spell everything out, ambiguous numbers on a best-effort basis.
    #[default]
    Strict,
    /// Leave ambiguous numbers as digits for a later language-model pass.
    Safe,
}

impl Mode {
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "strict" => Some(Mode::Strict),
            "safe" => Some(Mode::Safe),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Mode::Strict => "strict",
            Mode::Safe => "safe",
        }
    }
}

/// Lexicon (if any), then rules.
pub fn normalize_text(text: &str, lang: Lang, mode: Mode, lexicon: Option<&Lexicon>) -> String {
    let text = match lexicon {
        Some(lex) => lex.apply(text, lang),
        None => text.to_string(),
    };
    match mode {
        Mode::Strict => normalize(&text, lang),
        Mode::Safe => normalize_safe(&text, lang),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// covers: REQ-LNG-001
    #[test]
    fn language_codes() {
        for c in ["fr", "fr-FR", "FRA", "french", "Français", "french_24l"] {
            assert_eq!(Lang::from_code(c), Lang::Fr, "{c}");
        }
        for c in ["en", "en_US", "english", "english_2026-04"] {
            assert_eq!(Lang::from_code(c), Lang::En, "{c}");
        }
        for c in ["de", "mg", "", "spanish"] {
            assert_eq!(Lang::from_code(c), Lang::Other, "{c}");
        }
    }

    /// covers: REQ-LEX-001
    #[test]
    fn lexicon_runs_before_rules() {
        let lex = Lexicon::parse("RTX\tR T X\nfr\tMd€\tmilliards d'euros\n").unwrap();
        assert_eq!(
            normalize_text(
                "La RTX 4090 coûte 2 Md€.",
                Lang::Fr,
                Mode::Strict,
                Some(&lex)
            ),
            "La R T X quatre mille quatre-vingt-dix coûte deux milliards d'euros."
        );
    }

    /// covers: REQ-MOD-001
    #[test]
    fn modes() {
        assert_eq!(Mode::parse("Safe"), Some(Mode::Safe));
        assert_eq!(Mode::parse("x"), None);
        let s = normalize_text("Gate 12B at 9:30 am.", Lang::En, Mode::Safe, None);
        assert_eq!(s, "Gate 12B at nine thirty a m.");
        let s = normalize_text("Gate 12B at 9:30 am.", Lang::En, Mode::Strict, None);
        assert_eq!(s, "Gate twelve B at nine thirty a m.");
    }
}
