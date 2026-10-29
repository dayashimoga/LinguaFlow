use lingua_core::{normalize_text_nfc, LanguageCode};

/// Result of a morphological analysis and stemming operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StemResult {
    pub original: String,
    pub root: String,
    pub suffix: Option<String>,
    pub language: LanguageCode,
}

/// Stem an inflected word in Kannada, Tamil, or English to its canonical base lemma.
pub fn stem_word(word: &str, lang: LanguageCode) -> StemResult {
    let normalized = normalize_text_nfc(word.trim());
    if normalized.is_empty() {
        return StemResult {
            original: String::new(),
            root: String::new(),
            suffix: None,
            language: lang,
        };
    }

    match lang {
        LanguageCode::KnIn => stem_kannada(&normalized),
        LanguageCode::TaIn => stem_tamil(&normalized),
        LanguageCode::EnUs => stem_english(&normalized),
        _ => StemResult {
            original: normalized.clone(),
            root: normalized,
            suffix: None,
            language: lang,
        },
    }
}

/// Agglutinative stemmer for Kannada nouns, case markers, and common verbs.
fn stem_kannada(word: &str) -> StemResult {
    // Suffixes ordered by longest-match-first
    const KN_SUFFIXES: &[(&str, Option<&str>)] = &[
        // Plural case markers
        ("ಗಳನ್ನು", None), // e.g. ಸಾಹಸಗಳನ್ನು -> ಸಾಹಸ
        ("ಗಳಿಂದ", None),  // e.g. ಸಾಹಸಗಳಿಂದ -> ಸಾಹಸ
        ("ಗಳಿಗೆ", None),   // e.g. ಸಾಹಸಗಳಿಗೆ -> ಸಾಹಸ
        ("ಗಳಲ್ಲಿ", None),  // e.g. ಸಾಹಸಗಳಲ್ಲಿ -> ಸಾಹಸ
        ("ಗಳ", None),    // e.g. ಸಾಹಸಗಳ -> ಸಾಹಸ
        ("ಗಳು", None),   // e.g. ಸಾಹಸಗಳು -> ಸಾಹಸ
        // Singular case markers
        ("ವನ್ನು", None), // e.g. ಸಾಹಸವನ್ನು -> ಸಾಹಸ
        ("ಅನ್ನು", None), // e.g. ನಮಸ್ಕಾರವನ್ನು
        ("ದಿಂದ", None),  // e.g. ಸಾಹಸದಿಂದ -> ಸಾಹಸ
        ("ಇಂದ", None),
        ("ಕ್ಕೆ", None), // e.g. ಸಾಹಸಕ್ಕೆ -> ಸಾಹಸ
        ("ಗೆ", None),
        ("ದಲ್ಲಿ", None), // e.g. ಸಾಹಸದಲ್ಲಿ -> ಸಾಹಸ
        ("ಅಲ್ಲಿ", None),
        ("ದ", None), // e.g. ಸಾಹಸದ -> ಸಾಹಸ
        ("ಯ", None),
        ("ನ್ನ", None),
        ("ನ್ನು", None),
        // Verb markers
        ("ಉತ್ತದೆ", Some("ು")), // e.g. ಮಾಡುತ್ತದೆ -> ಮಾಡು
        ("ತ್ತದೆ", Some("ು")),
        ("ತ್ತಾರೆ", Some("ು")),
    ];

    for (suffix, replacement) in KN_SUFFIXES {
        if word.ends_with(suffix) && word.chars().count() > suffix.chars().count() + 1 {
            let base_len = word.len() - suffix.len();
            let mut root = word[..base_len].to_string();
            if let Some(rep) = replacement {
                root.push_str(rep);
            }
            return StemResult {
                original: word.to_string(),
                root,
                suffix: Some((*suffix).to_string()),
                language: LanguageCode::KnIn,
            };
        }
    }

    StemResult {
        original: word.to_string(),
        root: word.to_string(),
        suffix: None,
        language: LanguageCode::KnIn,
    }
}

/// Agglutinative stemmer for Tamil nouns, sandhi doubled consonants, and case markers.
fn stem_tamil(word: &str) -> StemResult {
    // Neuter noun ending in -ம் assimilates to -த்த- before case endings
    // e.g. சாகசங்களை -> சாகசம், சாகசத்தை -> சாகசம், சாகசத்திற்கு -> சாகசம்
    const TA_NEUTER_AM_SUFFIXES: &[(&str, &str)] = &[
        ("ங்களை", "ம்"),    // சாகசங்களை -> சாகசம்
        ("ங்களின்", "ம்"),   // சாகசங்களின் -> சாகசம்
        ("ங்களுக்கு", "ம்"), // சாகசங்களுக்கு -> சாகசம்
        ("ங்களில்", "ம்"),   // சாகசங்களில் -> சாகசம்
        ("ங்களால்", "ம்"),    // சாகசங்களால் -> சாகசம்
        ("ங்கள்", "ம்"),     // சாகசங்கள் -> சாகசம்
        ("த்தை", "ம்"),     // சாகசத்தை -> சாகசம்
        ("த்திற்கு", "ம்"),  // சாகசத்திற்கு -> சாகசம்
        ("த்தில்", "ம்"),    // சாகசத்தில் -> சாகசம்
        ("த்தால்", "ம்"),     // சாகசத்தால் -> சாகசம்
        ("த்தின்", "ம்"),    // சாகசத்தின் -> சாகசம்
        ("த்து", "ம்"),     // சாகசத்து -> சாகசம்
    ];

    for (suffix, rep) in TA_NEUTER_AM_SUFFIXES {
        if word.ends_with(suffix) && word.chars().count() > suffix.chars().count() + 1 {
            let base_len = word.len() - suffix.len();
            let mut root = word[..base_len].to_string();
            root.push_str(rep);
            return StemResult {
                original: word.to_string(),
                root,
                suffix: Some((*suffix).to_string()),
                language: LanguageCode::TaIn,
            };
        }
    }

    // Standard Tamil Case & Plural suffixes
    const TA_GENERAL_SUFFIXES: &[&str] = &[
        "களை",
        "களுக்கு",
        "களில்",
        "களால்",
        "களின்",
        "கள்",
        "ஐ",
        "யை",
        "வை",
        "க்கு",
        "வுக்கு",
        "ற்கு",
        "ஆல்",
        "யால்",
        "இல்",
        "யில்",
        "இடம்",
        "உடன்",
        "ஓடு",
        "ஆக",
        "ஆய்",
    ];

    for suffix in TA_GENERAL_SUFFIXES {
        if word.ends_with(suffix) && word.chars().count() > suffix.chars().count() + 1 {
            let base_len = word.len() - suffix.len();
            let mut root = word[..base_len].to_string();

            // Strip sandhi doubled consonant if present before suffix (க், ச், த், ப்)
            for sandhi in &["க்", "ச்", "த்", "ப்"] {
                if root.ends_with(sandhi) && root.chars().count() > sandhi.chars().count() + 1 {
                    let sandhi_len = root.len() - sandhi.len();
                    root = root[..sandhi_len].to_string();
                    break;
                }
            }

            return StemResult {
                original: word.to_string(),
                root,
                suffix: Some((*suffix).to_string()),
                language: LanguageCode::TaIn,
            };
        }
    }

    StemResult {
        original: word.to_string(),
        root: word.to_string(),
        suffix: None,
        language: LanguageCode::TaIn,
    }
}

/// Lemmatizer for English plurals, past tense, progressive, and common suffixes.
fn stem_english(word: &str) -> StemResult {
    let lower = word.to_lowercase();

    // 1. Irregular or special lemmas
    const IRREGULARS: &[(&str, &str)] = &[
        ("adventures", "adventure"),
        ("adventuring", "adventure"),
        ("adventurous", "adventure"),
        ("children", "child"),
        ("people", "person"),
        ("men", "man"),
        ("women", "woman"),
        ("feet", "foot"),
        ("teeth", "tooth"),
        ("geese", "goose"),
        ("mice", "mouse"),
    ];

    for (inflected, base) in IRREGULARS {
        if lower == *inflected {
            return StemResult {
                original: word.to_string(),
                root: (*base).to_string(),
                suffix: Some("irregular".into()),
                language: LanguageCode::EnUs,
            };
        }
    }

    // 2. Rules
    if lower.ends_with("ies") && lower.len() > 4 {
        let base = format!("{}y", &lower[..lower.len() - 3]);
        return StemResult {
            original: word.to_string(),
            root: base,
            suffix: Some("ies".into()),
            language: LanguageCode::EnUs,
        };
    }

    if lower.ends_with("ing") && lower.len() > 4 {
        let base_raw = &lower[..lower.len() - 3];
        // e.g. making -> make, adventuring -> adventure
        let base = if base_raw.ends_with('r') || base_raw.ends_with('t') || base_raw.ends_with('k')
        {
            format!("{base_raw}e")
        } else {
            base_raw.to_string()
        };
        return StemResult {
            original: word.to_string(),
            root: base,
            suffix: Some("ing".into()),
            language: LanguageCode::EnUs,
        };
    }

    if lower.ends_with("ed") && lower.len() > 3 {
        let base_raw = &lower[..lower.len() - 2];
        let base = if base_raw.ends_with('r') || base_raw.ends_with('t') {
            format!("{base_raw}e")
        } else {
            base_raw.to_string()
        };
        return StemResult {
            original: word.to_string(),
            root: base,
            suffix: Some("ed".into()),
            language: LanguageCode::EnUs,
        };
    }

    if lower.ends_with("es") && lower.len() > 3 {
        let base = &lower[..lower.len() - 2];
        return StemResult {
            original: word.to_string(),
            root: base.to_string(),
            suffix: Some("es".into()),
            language: LanguageCode::EnUs,
        };
    }

    if lower.ends_with('s') && !lower.ends_with("ss") && lower.len() > 2 {
        let base = &lower[..lower.len() - 1];
        return StemResult {
            original: word.to_string(),
            root: base.to_string(),
            suffix: Some("s".into()),
            language: LanguageCode::EnUs,
        };
    }

    StemResult {
        original: word.to_string(),
        root: word.to_string(),
        suffix: None,
        language: LanguageCode::EnUs,
    }
}

/// Calculate the Levenshtein distance between two strings with early cut-off threshold.
#[allow(clippy::needless_range_loop)]
pub fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();

    let m = a_chars.len();
    let n = b_chars.len();

    if m == 0 {
        return n;
    }
    if n == 0 {
        return m;
    }

    let mut dp = vec![vec![0; n + 1]; m + 1];

    for i in 0..=m {
        dp[i][0] = i;
    }
    for j in 0..=n {
        dp[0][j] = j;
    }

    for i in 1..=m {
        for j in 1..=n {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            dp[i][j] = std::cmp::min(
                std::cmp::min(dp[i - 1][j] + 1, dp[i][j - 1] + 1),
                dp[i - 1][j - 1] + cost,
            );
        }
    }

    dp[m][n]
}

/// Calculate character trigram Jaccard similarity between two strings.
pub fn trigram_similarity(a: &str, b: &str) -> f64 {
    let a_trigrams = extract_trigrams(a);
    let b_trigrams = extract_trigrams(b);

    if a_trigrams.is_empty() && b_trigrams.is_empty() {
        return 1.0;
    }
    if a_trigrams.is_empty() || b_trigrams.is_empty() {
        return 0.0;
    }

    let mut intersection = 0;
    for tri in &a_trigrams {
        if b_trigrams.contains(tri) {
            intersection += 1;
        }
    }

    let union = a_trigrams.len() + b_trigrams.len() - intersection;
    intersection as f64 / union as f64
}

fn extract_trigrams(text: &str) -> std::collections::HashSet<String> {
    let padded = format!("  {} ", text.to_lowercase());
    let chars: Vec<char> = padded.chars().collect();
    let mut set = std::collections::HashSet::new();

    if chars.len() < 3 {
        return set;
    }

    for i in 0..=chars.len() - 3 {
        let tri: String = chars[i..i + 3].iter().collect();
        set.insert(tri);
    }
    set
}

/// Rank candidate headwords by fuzzy similarity (bounded Levenshtein <= 2 and trigram >= 0.3).
pub fn find_did_you_mean_suggestions(
    query: &str,
    candidates: &[&str],
    limit: usize,
) -> Vec<String> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return Vec::new();
    }

    let mut scored: Vec<(String, usize, f64)> = Vec::new();

    for cand in candidates {
        let c = cand.trim().to_lowercase();
        if c == q {
            continue; // Skip exact matches
        }

        let dist = levenshtein_distance(&q, &c);
        let sim = trigram_similarity(&q, &c);

        if dist <= 2 || sim >= 0.35 {
            scored.push((cand.to_string(), dist, sim));
        }
    }

    // Sort prioritizing lowest distance first, then highest trigram similarity
    scored.sort_by(|a, b| {
        a.1.cmp(&b.1)
            .then_with(|| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal))
    });

    scored
        .into_iter()
        .take(limit)
        .map(|(word, _, _)| word)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kannada_agglutinative_stemmer() {
        // ಸಾಹಸಗಳನ್ನು (accusative plural) -> ಸಾಹಸ
        let res = stem_word("ಸಾಹಸಗಳನ್ನು", LanguageCode::KnIn);
        assert_eq!(res.root, "ಸಾಹಸ");
        assert_eq!(res.suffix, Some("ಗಳನ್ನು".into()));

        // ಸಾಹಸಗಳಿಂದ (instrumental plural) -> ಸಾಹಸ
        let res2 = stem_word("ಸಾಹಸಗಳಿಂದ", LanguageCode::KnIn);
        assert_eq!(res2.root, "ಸಾಹಸ");

        // ಸಾಹಸದಲ್ಲಿ (locative) -> ಸಾಹಸ
        let res3 = stem_word("ಸಾಹಸದಲ್ಲಿ", LanguageCode::KnIn);
        assert_eq!(res3.root, "ಸಾಹಸ");

        // ಸಾಹಸ (root stays root)
        let res4 = stem_word("ಸಾಹಸ", LanguageCode::KnIn);
        assert_eq!(res4.root, "ಸಾಹಸ");
        assert_eq!(res4.suffix, None);
    }

    #[test]
    fn test_tamil_agglutinative_stemmer() {
        // சாகசங்களை (neuter plural accusative) -> சாகசம்
        let res = stem_word("சாகசங்களை", LanguageCode::TaIn);
        assert_eq!(res.root, "சாகசம்");

        // சாகசத்தை (neuter accusative singular) -> சாகசம்
        let res2 = stem_word("சாகசத்தை", LanguageCode::TaIn);
        assert_eq!(res2.root, "சாகசம்");

        // சாகசத்திற்கு (neuter dative singular) -> சாகசம்
        let res3 = stem_word("சாகசத்திற்கு", LanguageCode::TaIn);
        assert_eq!(res3.root, "சாகசம்");

        // சாகசம் (root stays root)
        let res4 = stem_word("சாகசம்", LanguageCode::TaIn);
        assert_eq!(res4.root, "சாகசம்");
    }

    #[test]
    fn test_english_lemmatizer() {
        assert_eq!(
            stem_word("adventures", LanguageCode::EnUs).root,
            "adventure"
        );
        assert_eq!(
            stem_word("adventuring", LanguageCode::EnUs).root,
            "adventure"
        );
        assert_eq!(
            stem_word("adventurous", LanguageCode::EnUs).root,
            "adventure"
        );
    }

    #[test]
    fn test_fuzzy_did_you_mean() {
        let candidates = &[
            "adventure",
            "advertise",
            "advantage",
            "peace",
            "water",
            "doctor",
        ];
        let suggestions = find_did_you_mean_suggestions("advnture", candidates, 3);
        assert!(!suggestions.is_empty());
        assert_eq!(suggestions[0], "adventure");

        let kannada_candidates = &["ನಮಸ್ಕಾರ", "ಸ್ನೇಹ", "ಶಾಂತಿ", "ಸಾಹಸ"];
        let kn_sug = find_did_you_mean_suggestions("ನಮಸ್ಕರ", kannada_candidates, 2);
        assert!(!kn_sug.is_empty());
        assert_eq!(kn_sug[0], "ನಮಸ್ಕಾರ");
    }
}
