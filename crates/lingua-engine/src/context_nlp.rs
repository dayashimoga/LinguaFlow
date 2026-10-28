use lingua_core::{DomainGlossaryEntry, FormalityMode, LanguageCode};
use std::collections::HashMap;

/// Context-aware NLP processor for formality adaptation, glossary term enforcement,
/// and proper-noun preservation.
pub struct ContextNlpProcessor;

impl ContextNlpProcessor {
    /// Protect named entities and bracketed tokens before translation.
    /// Returns the masked text and a token map for post-translation restoration.
    pub fn protect_named_entities(text: &str) -> (String, HashMap<String, String>) {
        let mut protected_map = HashMap::new();
        let mut result = text.to_string();

        // 1. Bracketed entities e.g. {EntityName} or {{EntityName}}
        let mut token_counter = 0;
        let mut start_idx = 0;

        while let Some(open) = result[start_idx..].find('{') {
            let actual_open = start_idx + open;
            if let Some(close) = result[actual_open..].find('}') {
                let actual_close = actual_open + close;
                let original = result[actual_open..=actual_close].to_string();
                let placeholder = format!("__LF_PROTECT_{}__", token_counter);
                token_counter += 1;

                protected_map.insert(placeholder.clone(), original);
                result.replace_range(actual_open..=actual_close, &placeholder);
                start_idx = actual_open + placeholder.len();
            } else {
                break;
            }
        }

        // 2. Recognized standard technological and geographical entities
        let known_entities: &[&str] = &[
            "Antigravity",
            "LinguaFlow",
            "Linux",
            "Windows",
            "Rust",
            "Tokio",
            "SQLite",
            "Tauri",
            "Svelte",
            "GitHub",
            "Bengaluru",
            "Chennai",
            "Mysuru",
            "Karnataka",
            "Tamil Nadu",
        ];

        for &entity in known_entities {
            if result.contains(entity) {
                let placeholder = format!("__LF_PROTECT_{}__", token_counter);
                token_counter += 1;
                protected_map.insert(placeholder.clone(), entity.to_string());
                result = result.replace(entity, &placeholder);
            }
        }

        (result, protected_map)
    }

    /// Restore protected entities into the translated output.
    pub fn restore_named_entities(
        translated: &str,
        protected_map: &HashMap<String, String>,
    ) -> String {
        let mut output = translated.to_string();
        for (placeholder, original) in protected_map {
            output = output.replace(placeholder, original);
        }
        output
    }

    /// Apply domain glossary substitutions to the translated text.
    pub fn apply_domain_glossary(
        text: &str,
        source_lang: LanguageCode,
        target_lang: LanguageCode,
        glossary_entries: &[DomainGlossaryEntry],
    ) -> (String, Vec<String>) {
        let mut output = text.to_string();
        let mut applied_terms = Vec::new();

        for entry in glossary_entries {
            if entry.source_lang == source_lang && entry.target_lang == target_lang {
                let s_term = entry.source_term.to_lowercase();
                if s_term.is_empty() {
                    continue;
                }

                let lower_output = output.to_lowercase();
                if lower_output.contains(&s_term) {
                    let mut new_output = String::new();
                    let mut last_idx = 0;
                    let mut search_idx = 0;

                    while let Some(pos) = lower_output[search_idx..].find(&s_term) {
                        let actual_pos = search_idx + pos;
                        new_output.push_str(&output[last_idx..actual_pos]);
                        new_output.push_str(&entry.target_term);
                        last_idx = actual_pos + s_term.len();
                        search_idx = last_idx;
                    }
                    new_output.push_str(&output[last_idx..]);
                    output = new_output;
                    applied_terms.push(entry.source_term.clone());
                }
            }
        }

        (output, applied_terms)
    }

    /// Adapt text according to the requested formality level.
    pub fn adapt_formality(
        text: &str,
        target_lang: LanguageCode,
        formality: FormalityMode,
    ) -> String {
        match formality {
            FormalityMode::Neutral => text.to_string(),
            FormalityMode::Formal => Self::make_formal(text, target_lang),
            FormalityMode::Colloquial => Self::make_colloquial(text, target_lang),
        }
    }

    fn make_formal(text: &str, target_lang: LanguageCode) -> String {
        match target_lang {
            LanguageCode::KnIn => {
                let mut res = text.to_string();
                let formal_replacements = &[
                    ("ನೀನು", "ನೀವು"),
                    ("ಬಾ", "ಬನ್ನಿ"),
                    ("ಮಾಡು", "ಮಾಡಿ"),
                    ("ಹೋಗು", "ಹೋಗಿ"),
                    ("ಇದ್ದೀಯಾ", "ಇದ್ದೀರಾ"),
                    ("ತಿನ್ನು", "ಊಟ ಮಾಡಿ"),
                    ("ಕೇಳು", "ಕೇಳಿ"),
                    ("ಹೇಳು", "ಹೇಳಿ"),
                ];
                for (casual, formal) in formal_replacements {
                    res = res.replace(casual, formal);
                }
                res
            }
            LanguageCode::TaIn => {
                let mut res = text.to_string();
                let formal_replacements = &[
                    ("நீ", "நீங்கள்"),
                    ("வா", "வாருங்கள்"),
                    ("செய்", "செய்யுங்கள்"),
                    ("போ", "போங்கள்"),
                    ("இருக்கிறாய்", "இருக்கிறீர்கள்"),
                    ("சாப்பிடு", "சாப்பிடுங்கள்"),
                    ("கேள்", "கேளுங்கள்"),
                    ("சொல்", "சொல்லுங்கள்"),
                ];
                for (casual, formal) in formal_replacements {
                    res = res.replace(casual, formal);
                }
                res
            }
            LanguageCode::EnUs => {
                let mut res = text.to_string();
                let formal_replacements = &[
                    ("hey", "hello"),
                    ("thanks", "thank you very much"),
                    ("see ya", "goodbye"),
                ];
                for (casual, formal) in formal_replacements {
                    let words: Vec<&str> = res.split_whitespace().collect();
                    let mut new_words = Vec::new();
                    for w in words {
                        if w.eq_ignore_ascii_case(casual) {
                            new_words.push((*formal).to_string());
                        } else {
                            new_words.push(w.to_string());
                        }
                    }
                    res = new_words.join(" ");
                }
                res
            }
            _ => text.to_string(),
        }
    }

    fn make_colloquial(text: &str, target_lang: LanguageCode) -> String {
        match target_lang {
            LanguageCode::KnIn => {
                let mut res = text.to_string();
                let casual_replacements = &[
                    ("ನೀವು", "ನೀನು"),
                    ("ಬನ್ನಿ", "ಬಾ"),
                    ("ಮಾಡಿ", "ಮಾಡು"),
                    ("ಹೋಗಿ", "ಹೋಗು"),
                    ("ಇದ್ದೀರಾ", "ಇದ್ದೀಯಾ"),
                    ("ಹೇಗಿದ್ದೀರಿ", "ಹೇಗಿದ್ದೀಯಾ"),
                ];
                for (formal, casual) in casual_replacements {
                    res = res.replace(formal, casual);
                }
                res
            }
            LanguageCode::TaIn => {
                let mut res = text.to_string();
                let casual_replacements = &[
                    ("நீங்கள்", "நீ"),
                    ("வாருங்கள்", "வா"),
                    ("செய்யுங்கள்", "செய்"),
                    ("போங்கள்", "போ"),
                    ("இருக்கிறீர்கள்", "இருக்கிறாய்"),
                    ("எப்படி இருக்கிறீர்கள்", "எப்படி இருக்கீங்க"),
                ];
                for (formal, casual) in casual_replacements {
                    res = res.replace(formal, casual);
                }
                res
            }
            LanguageCode::EnUs => {
                let mut res = text.to_string();
                let casual_replacements = &[
                    ("good morning", "morning!"),
                    ("goodbye", "see ya!"),
                    ("how are you", "what's up"),
                ];
                for (formal, casual) in casual_replacements {
                    let words: Vec<&str> = res.split_whitespace().collect();
                    let mut new_words = Vec::new();
                    for w in words {
                        if w.eq_ignore_ascii_case(formal) {
                            new_words.push((*casual).to_string());
                        } else {
                            new_words.push(w.to_string());
                        }
                    }
                    res = new_words.join(" ");
                }
                res
            }
            _ => text.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_protect_named_entities() {
        let (masked, map) =
            ContextNlpProcessor::protect_named_entities("Welcome to Bengaluru with {Antigravity}");
        assert!(masked.contains("__LF_PROTECT_"));
        assert!(!masked.contains("Bengaluru"));
        assert!(!masked.contains("{Antigravity}"));

        let restored = ContextNlpProcessor::restore_named_entities(&masked, &map);
        assert_eq!(restored, "Welcome to Bengaluru with {Antigravity}");
    }

    #[test]
    fn test_domain_glossary_replacement() {
        let glossary = vec![DomainGlossaryEntry {
            id: "1".into(),
            domain: "tech".into(),
            source_term: "computer".into(),
            target_term: "ಗಣಕಯಂತ್ರ".into(),
            source_lang: LanguageCode::EnUs,
            target_lang: LanguageCode::KnIn,
            is_exact: true,
            created_at: Utc::now(),
        }];

        let (applied, terms) = ContextNlpProcessor::apply_domain_glossary(
            "This computer is fast",
            LanguageCode::EnUs,
            LanguageCode::KnIn,
            &glossary,
        );

        assert_eq!(applied, "This ಗಣಕಯಂತ್ರ is fast");
        assert_eq!(terms, vec!["computer"]);
    }

    #[test]
    fn test_formality_adaptation() {
        let formal_kn = ContextNlpProcessor::adapt_formality(
            "ನೀನು ಬಾ",
            LanguageCode::KnIn,
            FormalityMode::Formal,
        );
        assert_eq!(formal_kn, "ನೀವು ಬನ್ನಿ");

        let casual_ta = ContextNlpProcessor::adapt_formality(
            "நீங்கள் வாருங்கள்",
            LanguageCode::TaIn,
            FormalityMode::Colloquial,
        );
        assert_eq!(casual_ta, "நீ வா");
    }
}
