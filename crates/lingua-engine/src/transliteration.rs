use crate::lru_cache::BoundedLruCache;
use lingua_core::{LanguageCode, LinguaError, TransliterationResponse};

/// High-performance, LRU-cached phonetic transliterator for Romanized Indic text.
pub struct TransliterationEngine {
    cache: BoundedLruCache<String, String>,
}

impl Default for TransliterationEngine {
    fn default() -> Self {
        Self::new(1000)
    }
}

impl TransliterationEngine {
    pub fn new(cache_size: usize) -> Self {
        Self {
            cache: BoundedLruCache::new(cache_size),
        }
    }

    /// Transliterate Romanized (Latin) text into the native script of `target_lang`.
    pub fn transliterate(
        &self,
        input: &str,
        target_lang: LanguageCode,
    ) -> Result<TransliterationResponse, LinguaError> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Ok(TransliterationResponse {
                input: input.to_string(),
                transliterated: String::new(),
                target_lang,
            });
        }

        let cache_key = format!("{}:{}", target_lang.as_bcp47(), trimmed.to_lowercase());
        if let Some(cached) = self.cache.get(&cache_key) {
            return Ok(TransliterationResponse {
                input: input.to_string(),
                transliterated: cached,
                target_lang,
            });
        }

        let result = match target_lang {
            LanguageCode::KnIn => self.transliterate_to_kannada(trimmed),
            LanguageCode::TaIn => self.transliterate_to_tamil(trimmed),
            LanguageCode::EnUs => trimmed.to_string(),
            _ => trimmed.to_string(),
        };

        self.cache.insert(cache_key, result.clone());

        Ok(TransliterationResponse {
            input: input.to_string(),
            transliterated: result,
            target_lang,
        })
    }

    fn transliterate_to_kannada(&self, text: &str) -> String {
        // First check common colloquial phrase dictionary for idiomatic mapping
        let lower = text.to_lowercase();
        let direct_mappings: &[(&str, &str)] = &[
            ("namaskara", "ನಮಸ್ಕಾರ"),
            ("namaste", "ನಮಸ್ತೆ"),
            ("kannada", "ಕನ್ನಡ"),
            ("hegiddira", "ಹೇಗಿದ್ದೀರಾ"),
            ("hegidhdhira", "ಹೇಗಿದ್ದೀರಾ"),
            ("hegiddiya", "ಹೇಗಿದ್ದೀಯಾ"),
            ("dhanyavada", "ಧನ್ಯವಾದ"),
            ("dhanyavadagalu", "ಧನ್ಯವಾದಗಳು"),
            ("banni", "ಬನ್ನಿ"),
            ("ba", "ಬಾ"),
            ("neevu", "ನೀವು"),
            ("neenu", "ನೀನು"),
            ("oota aayitha", "ಊಟ ಆಯ್ತಾ"),
            ("oota", "ಊಟ"),
            ("aayitha", "ಆಯ್ತಾ"),
            ("chennagiddira", "ಚೆನ್ನಾಗಿದ್ದೀರಾ"),
            ("chennagiddini", "ಚೆನ್ನಾಗಿದ್ದೀನಿ"),
            ("houdu", "ಹೌದು"),
            ("illa", "ಇಲ್ಲ"),
            ("shubhodaya", "ಶುಭೋದಯ"),
            ("shubhadina", "ಶುಭದಿನ"),
            ("shubharathri", "ಶುಭರಾತ್ರಿ"),
            ("bengaluru", "ಬೆಂಗಳೂರು"),
            ("mysuru", "ಮೈಸೂರು"),
            ("karnataka", "ಕರ್ನಾಟಕ"),
            ("sahaya", "ಸಹಾಯ"),
            ("dayavittu", "ದಯವಿಟ್ಟು"),
            ("tumba", "ತುಂಬಾ"),
            ("santosa", "ಸಂತೋಷ"),
            ("tante", "ತಂತೆ"),
            ("maneyalli", "ಮನೆಯಲ್ಲಿ"),
            ("yaru", "ಯಾರು"),
            ("yavaga", "ಯಾವಾಗ"),
            ("elli", "ಎಲ್ಲಿ"),
            ("hege", "ಹೇಗೆ"),
            ("enu", "ಏನು"),
        ];

        for (roman, kannada) in direct_mappings {
            if lower == *roman {
                return (*kannada).to_string();
            }
        }

        // Word-by-word transliteration
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut converted_words = Vec::new();

        for w in words {
            let clean = w.to_lowercase();
            let mut matched = false;
            for (roman, kannada) in direct_mappings {
                if clean == *roman {
                    converted_words.push((*kannada).to_string());
                    matched = true;
                    break;
                }
            }
            if !matched {
                converted_words.push(Self::syllabic_kannada_translit(&clean));
            }
        }

        converted_words.join(" ")
    }

    fn transliterate_to_tamil(&self, text: &str) -> String {
        let lower = text.to_lowercase();
        let direct_mappings: &[(&str, &str)] = &[
            ("vanakkam", "வணக்கம்"),
            ("tamil", "தமிழ்"),
            ("thamizh", "தமிழ்"),
            ("nandri", "நன்றி"),
            ("eppadi irukkinga", "எப்படி இருக்கீங்க"),
            ("eppadi irukkirai", "எப்படி இருக்கிறாய்"),
            ("eppadi", "எப்படி"),
            ("irukkinga", "இருக்கீங்க"),
            ("vaanga", "வாங்க"),
            ("vaa", "வா"),
            ("neengal", "நீங்கள்"),
            ("nee", "நீ"),
            ("saapitaacha", "சாப்பிட்டாச்சா"),
            ("saapadu", "சாப்பாடு"),
            ("aamaam", "ஆமாம்"),
            ("aam", "ஆம்"),
            ("illai", "இல்லை"),
            ("kaalai vanakkam", "காலை வணக்கம்"),
            ("iravu vanakkam", "இரவு வணக்கம்"),
            ("chennai", "சென்னை"),
            ("madurai", "மதுரை"),
            ("tamil nadu", "தமிழ் நாடு"),
            ("udhavi", "உதவி"),
            ("dhayavuseidhu", "தயவுசெய்து"),
            ("romba", "ரொம்ப"),
            ("magizhchi", "மகிழ்ச்சி"),
            ("yaar", "யார்"),
            ("eppodhu", "எப்போது"),
            ("enge", "எங்கே"),
            ("enna", "என்ன"),
        ];

        for (roman, tamil) in direct_mappings {
            if lower == *roman {
                return (*tamil).to_string();
            }
        }

        let words: Vec<&str> = text.split_whitespace().collect();
        let mut converted_words = Vec::new();

        for w in words {
            let clean = w.to_lowercase();
            let mut matched = false;
            for (roman, tamil) in direct_mappings {
                if clean == *roman {
                    converted_words.push((*tamil).to_string());
                    matched = true;
                    break;
                }
            }
            if !matched {
                converted_words.push(Self::syllabic_tamil_translit(&clean));
            }
        }

        converted_words.join(" ")
    }

    /// Syllabic rule-based mapping for arbitrary romanized tokens to Kannada script
    fn syllabic_kannada_translit(word: &str) -> String {
        // Table of consonant-vowel syllable clusters
        let mut res = String::new();
        let chars: Vec<char> = word.chars().collect();
        let n = chars.len();
        let mut i = 0;

        while i < n {
            // Check 3-char clusters
            if i + 2 < n {
                let sub3: String = chars[i..i + 3].iter().collect();
                if let Some(mapped) = match sub3.as_str() {
                    "ksha" | "kxa" => Some("ಕ್ಷ"),
                    "shha" => Some("ಷ"),
                    "chha" => Some("ಛ"),
                    "jhha" => Some("ಝ"),
                    "thha" => Some("ಠ"),
                    "dhha" => Some("ಢ"),
                    _ => None,
                } {
                    res.push_str(mapped);
                    i += 3;
                    continue;
                }
            }

            // Check 2-char clusters
            if i + 1 < n {
                let sub2: String = chars[i..i + 2].iter().collect();
                if let Some(mapped) = match sub2.as_str() {
                    "aa" => Some("ಾ"),
                    "ee" | "ii" => Some("ೀ"),
                    "oo" | "uu" => Some("ೂ"),
                    "ai" => Some("ೈ"),
                    "au" | "ou" => Some("ೌ"),
                    "kh" => Some("ಖ"),
                    "gh" => Some("ಘ"),
                    "ch" => Some("ಚ"),
                    "jh" => Some("ಝ"),
                    "th" => Some("ತ"),
                    "dh" => Some("ಧ"),
                    "ph" | "bh" => Some("ಭ"),
                    "sh" => Some("ಶ"),
                    "ng" => Some("ಂಗ"),
                    "ny" => Some("ಞ"),
                    _ => None,
                } {
                    res.push_str(mapped);
                    i += 2;
                    continue;
                }
            }

            // Single characters
            let c = chars[i];
            let mapped = match c {
                'a' => "", // implicit vowel when after consonant
                'i' => "ಿ",
                'u' => "ು",
                'e' => "ೆ",
                'o' => "ೊ",
                'k' => "ಕ",
                'g' => "ಗ",
                'c' => "ಚ",
                'j' => "ಜ",
                't' => "ಟ",
                'd' => "ಡ",
                'n' => "ನ",
                'p' => "ಪ",
                'b' => "ಬ",
                'm' => "ಮ",
                'y' => "ಯ",
                'r' => "ರ",
                'l' => "ಲ",
                'v' | 'w' => "ವ",
                's' => "ಸ",
                'h' => "ಹ",
                _ => {
                    res.push(c);
                    i += 1;
                    continue;
                }
            };
            res.push_str(mapped);
            i += 1;
        }

        if res.is_empty() {
            word.to_string()
        } else {
            res
        }
    }

    /// Syllabic rule-based mapping for arbitrary romanized tokens to Tamil script
    fn syllabic_tamil_translit(word: &str) -> String {
        let mut res = String::new();
        let chars: Vec<char> = word.chars().collect();
        let n = chars.len();
        let mut i = 0;

        while i < n {
            if i + 1 < n {
                let sub2: String = chars[i..i + 2].iter().collect();
                if let Some(mapped) = match sub2.as_str() {
                    "aa" => Some("ா"),
                    "ee" | "ii" => Some("ீ"),
                    "oo" | "uu" => Some("ூ"),
                    "ai" => Some("ை"),
                    "au" | "ou" => Some("ௌ"),
                    "th" => Some("த"),
                    "sh" => Some("ஷ"),
                    "ng" => Some("ங"),
                    "ny" | "nj" => Some("ஞ"),
                    "zh" => Some("ழ"),
                    _ => None,
                } {
                    res.push_str(mapped);
                    i += 2;
                    continue;
                }
            }

            let c = chars[i];
            let mapped = match c {
                'a' => "",
                'i' => "ி",
                'u' => "ு",
                'e' => "ெ",
                'o' => "ொ",
                'k' | 'g' => "க",
                'c' | 's' => "ச",
                'j' => "ஜ",
                't' | 'd' => "ட",
                'n' => "ந",
                'p' | 'b' => "ப",
                'm' => "ம",
                'y' => "ய",
                'r' => "ர",
                'l' => "ல",
                'v' | 'w' => "வ",
                'h' => "ஹ",
                _ => {
                    res.push(c);
                    i += 1;
                    continue;
                }
            };
            res.push_str(mapped);
            i += 1;
        }

        if res.is_empty() {
            word.to_string()
        } else {
            res
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kannada_romanized_transliteration() {
        let engine = TransliterationEngine::default();
        let res = engine
            .transliterate("namaskara", LanguageCode::KnIn)
            .expect("transliterate failed");
        assert_eq!(res.transliterated, "ನಮಸ್ಕಾರ");

        let res2 = engine
            .transliterate("kannada", LanguageCode::KnIn)
            .expect("transliterate failed");
        assert_eq!(res2.transliterated, "ಕನ್ನಡ");

        let res3 = engine
            .transliterate("hegiddira", LanguageCode::KnIn)
            .expect("transliterate failed");
        assert_eq!(res3.transliterated, "ಹೇಗಿದ್ದೀರಾ");
    }

    #[test]
    fn test_tamil_romanized_transliteration() {
        let engine = TransliterationEngine::default();
        let res = engine
            .transliterate("vanakkam", LanguageCode::TaIn)
            .expect("transliterate failed");
        assert_eq!(res.transliterated, "வணக்கம்");

        let res2 = engine
            .transliterate("tamil", LanguageCode::TaIn)
            .expect("transliterate failed");
        assert_eq!(res2.transliterated, "தமிழ்");

        let res3 = engine
            .transliterate("nandri", LanguageCode::TaIn)
            .expect("transliterate failed");
        assert_eq!(res3.transliterated, "நன்றி");
    }

    #[test]
    fn test_cache_hit_performance() {
        let engine = TransliterationEngine::new(50);
        let res1 = engine
            .transliterate("bengaluru", LanguageCode::KnIn)
            .unwrap();
        let res2 = engine
            .transliterate("bengaluru", LanguageCode::KnIn)
            .unwrap();
        assert_eq!(res1.transliterated, "ಬೆಂಗಳೂರು");
        assert_eq!(res2.transliterated, "ಬೆಂಗಳೂರು");
        assert_eq!(engine.cache.stats().hits, 1);
    }
}
