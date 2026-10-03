//! Roman numerals, read only where the context makes them unambiguous:
//!
//! - an ordinal suffix (French): `XVIIe siècle` → dix-septième, `Ier` → premier;
//! - after a word that announces a number: `chapitre IV`, `World War II`,
//!   `Super Bowl LVIII` → the cardinal;
//! - after a proper name (regnal numbers): `Louis XIV` → quatorze,
//!   `Henry VIII` → the eighth, `Jean-Paul II` → deux.
//!
//! Everything else stays as written: a lone capital (`vitamine C`, `plan D`)
//! or a word that is also a common acronym (`CV`, `CD`, `MD`, `DC`, `XL`…).

use regex::{Captures, Regex};
use std::sync::LazyLock;

const NUMERALS: [(u64, &str); 13] = [
    (1000, "M"),
    (900, "CM"),
    (500, "D"),
    (400, "CD"),
    (100, "C"),
    (90, "XC"),
    (50, "L"),
    (40, "XL"),
    (10, "X"),
    (9, "IX"),
    (5, "V"),
    (4, "IV"),
    (1, "I"),
];

fn encode(mut n: u64) -> String {
    let mut out = String::new();
    for (v, s) in NUMERALS {
        while n >= v {
            out.push_str(s);
            n -= v;
        }
    }
    out
}

/// Value of a canonical Roman numeral (`XIV`, not `XIIII` or `IC`), 1-3999.
pub(crate) fn value(s: &str) -> Option<u64> {
    let digit = |c: char| match c {
        'I' => Some(1),
        'V' => Some(5),
        'X' => Some(10),
        'L' => Some(50),
        'C' => Some(100),
        'D' => Some(500),
        'M' => Some(1000),
        _ => None,
    };
    let ds: Vec<u64> = s.chars().map(digit).collect::<Option<_>>()?;
    let mut n: i64 = 0;
    for (i, &d) in ds.iter().enumerate() {
        if ds.get(i + 1).is_some_and(|&next| next > d) {
            n -= d as i64;
        } else {
            n += d as i64;
        }
    }
    let n = u64::try_from(n).ok().filter(|n| (1..4000).contains(n))?;
    (encode(n) == s).then_some(n)
}

/// Valid numerals that are more often acronyms or words.
const ACRONYMS: &[&str] = &[
    "CV", "CD", "DC", "MD", "CM", "DM", "MC", "CC", "CI", "DI", "LI", "MI", "MM", "XL", "MIX",
    "DIV", "CIL", "MIL", "CLI", "CCC", "XXX", "VI", "MCM",
];

fn reads_as_number(s: &str) -> Option<u64> {
    if s.len() < 2 || ACRONYMS.contains(&s) {
        return None;
    }
    value(s)
}

/// Capitalized words that start a sentence or a noun group, not a name.
const NOT_NAMES: &[&str] = &[
    "Le",
    "La",
    "Les",
    "L",
    "Un",
    "Une",
    "Des",
    "Du",
    "De",
    "Mon",
    "Ma",
    "Mes",
    "Ton",
    "Ta",
    "Tes",
    "Son",
    "Sa",
    "Ses",
    "Notre",
    "Votre",
    "Nos",
    "Vos",
    "Leur",
    "Leurs",
    "Ce",
    "Cet",
    "Cette",
    "Ces",
    "En",
    "Au",
    "Aux",
    "Et",
    "Ou",
    "Par",
    "Pour",
    "Sur",
    "Avec",
    "Dans",
    "Chez",
    "Voici",
    "Voilà",
    "The",
    "A",
    "An",
    "My",
    "Your",
    "His",
    "Her",
    "Our",
    "Their",
    "This",
    "That",
    "These",
    "Those",
    "In",
    "On",
    "At",
    "For",
    "With",
    "By",
    "Of",
    "And",
    // sentence starts that are not names
    "Je",
    "Tu",
    "Il",
    "Elle",
    "Nous",
    "Vous",
    "Ils",
    "Elles",
    "Mais",
    "Si",
    "Quand",
    "Alors",
    "Puis",
    "Donc",
    "Comme",
    "Selon",
    "Après",
    "Avant",
    "Depuis",
    "Entre",
    "Sans",
    "Sous",
    "Vers",
    "Est",
    "Sont",
    "Utilisez",
    "Voir",
    "Installez",
    "Lancez",
    "Ouvrez",
    "Avec",
    "Via",
    "I",
    "We",
    "You",
    "He",
    "She",
    "It",
    "They",
    "If",
    "When",
    "But",
    "So",
    "As",
    "From",
    "To",
    "Is",
    "Are",
    "Use",
    "See",
    "Install",
    "Run",
    "Open",
    "Via",
    "Or",
    "Not",
    "All",
];

/// Words after which a Roman numeral is a cardinal.
const FR_KEYWORDS: &str = r"chapitre|tome|acte|scène|volume|partie|livre|article|titre|leçon|saison|épisode|annexe|phase|niveau|guerre mondiale|Bowl";
const EN_KEYWORDS: &str = r"chapter|volume|part|book|act|scene|article|title|lesson|season|episode|appendix|phase|level|World War|Bowl";

static FR_SUFFIXED: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b([IVXLCDM]+)(?:(er|re|ère|ᵉʳ)|(e|ème|ᵉ))\b").unwrap());
static FR_AFTER_KEYWORD: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(&format!(r"(?i:\b({FR_KEYWORDS}))\s+([IVXLCDM]+)\b")).unwrap());
static EN_AFTER_KEYWORD: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(&format!(r"(?i:\b({EN_KEYWORDS}))\s+([IVXLCDM]+)\b")).unwrap());
static AFTER_NAME: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b(\p{Lu}\p{Ll}+(?:-\p{Lu}\p{Ll}+)?)\s+([IVXLCDM]+)\b").unwrap());

/// A numeral not followed by more of a word (`XIV.` is fine, `XIVth` is not).
fn ends_word(hay: &str, end: usize) -> bool {
    !hay[end..].chars().next().is_some_and(char::is_alphanumeric)
}

/// French: `XVIIe` → dix-septième, `Ier` → premier, `chapitre IV` → chapitre
/// quatre, `Louis XIV` → Louis quatorze, `François Ier` → François premier.
pub(crate) fn french(
    s: &str,
    cardinal: fn(u64) -> String,
    ordinal: fn(u64, bool) -> String,
) -> String {
    let s = FR_SUFFIXED.replace_all(s, |c: &Captures| {
        let roman = &c[1];
        let first = c.get(2).is_some();
        // "Ier"/"Ire" only as 1; other suffixed forms need two letters
        // ("Ce", "De", "Me" are words).
        let n = match roman {
            "I" if first => Some(1),
            // "Ve République", "Xe siècle"; other lone letters are words
            // or initials ("Ce", "De", "Me", "Le").
            "V" | "X" if !first => value(roman),
            _ => reads_as_number(roman),
        };
        match n {
            Some(1) if first => if matches!(&c[2], "re" | "ère") {
                "première"
            } else {
                "premier"
            }
            .to_string(),
            Some(n) => ordinal(n, false),
            None => c[0].to_string(),
        }
    });
    let s = FR_AFTER_KEYWORD.replace_all(&s, |c: &Captures| match value(&c[2]) {
        Some(n) => format!("{} {}", &c[1], cardinal(n)),
        None => c[0].to_string(),
    });
    after_name(&s, |n| {
        if n == 1 {
            "premier".into()
        } else {
            cardinal(n)
        }
    })
}

/// English: `World War II` → World War two, `Henry VIII` → Henry the eighth.
pub(crate) fn english(s: &str, cardinal: fn(u64) -> String, ordinal: fn(u64) -> String) -> String {
    let s = EN_AFTER_KEYWORD.replace_all(s, |c: &Captures| match value(&c[2]) {
        Some(n) => format!("{} {}", &c[1], cardinal(n)),
        None => c[0].to_string(),
    });
    after_name(&s, |n| format!("the {}", ordinal(n)))
}

fn after_name(s: &str, read: impl Fn(u64) -> String) -> String {
    let hay = s.to_string();
    AFTER_NAME
        .replace_all(&hay, |c: &Captures| {
            let name = &c[1];
            let roman = c.get(2).unwrap();
            if NOT_NAMES.contains(&name) || !ends_word(&hay, roman.end()) {
                return c[0].to_string();
            }
            match reads_as_number(roman.as_str()) {
                Some(n) => format!("{name} {}", read(n)),
                None => c[0].to_string(),
            }
        })
        .into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// covers: REQ-TXT-006
    #[test]
    fn canonical_values() {
        for (s, n) in [
            ("I", 1),
            ("IV", 4),
            ("XIV", 14),
            ("XVII", 17),
            ("LVIII", 58),
            ("MCMXCIX", 1999),
            ("MMXXVI", 2026),
        ] {
            assert_eq!(value(s), Some(n), "{s}");
        }
        for s in ["IIII", "IC", "VV", "XM", "", "ABC"] {
            assert_eq!(value(s), None, "{s}");
        }
    }
}
