use lingua_core::{
    normalize_text_nfc, DictionaryExample, LanguageCode, LexiconSense, LinguaError,
    RichDictionaryEntry,
};
use lingua_storage::StorageService;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::time::Instant;
use tracing::info;

#[derive(Debug, Clone, Default)]
pub struct ImportStats {
    pub total_read: usize,
    pub valid_entries: usize,
    pub rejected_invalid: usize,
    pub duplicates_skipped: usize,
    pub duration_ms: u64,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct WiktextractEntry {
    word: Option<String>,
    pos: Option<String>,
    lang_code: Option<String>,
    senses: Option<Vec<WiktextractSense>>,
    sounds: Option<Vec<WiktextractSound>>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct WiktextractSense {
    glosses: Option<Vec<String>>,
    tags: Option<Vec<String>>,
    examples: Option<Vec<WiktextractExample>>,
    synonyms: Option<Vec<WiktextractLink>>,
    antonyms: Option<Vec<WiktextractLink>>,
}

#[derive(Debug, Deserialize)]
struct WiktextractSound {
    ipa: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WiktextractExample {
    text: Option<String>,
    english: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WiktextractLink {
    word: Option<String>,
}

/// Strip malformed HTML and wikitext markup (e.g. {{...}}, [[...]], <tag>).
pub fn strip_wikitext_and_html(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_html = false;
    let mut in_wiki_double = 0;

    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let c = chars[i];
        if c == '<' {
            in_html = true;
            i += 1;
            continue;
        }
        if c == '>' {
            in_html = false;
            i += 1;
            continue;
        }
        if in_html {
            i += 1;
            continue;
        }

        if c == '{' && i + 1 < len && chars[i + 1] == '{' {
            in_wiki_double += 1;
            i += 2;
            continue;
        }
        if c == '}' && i + 1 < len && chars[i + 1] == '}' && in_wiki_double > 0 {
            in_wiki_double -= 1;
            i += 2;
            continue;
        }

        if in_wiki_double > 0 {
            i += 1;
            continue;
        }

        if c == '[' && i + 1 < len && chars[i + 1] == '[' {
            i += 2;
            continue;
        }
        if c == ']' && i + 1 < len && chars[i + 1] == ']' {
            i += 2;
            continue;
        }

        out.push(c);
        i += 1;
    }

    out.trim().to_string()
}

/// Validate whether the given word conforms to BCP-47 script Unicode boundaries.
pub fn validate_script_boundary(word: &str, lang: LanguageCode) -> bool {
    let normalized = normalize_text_nfc(word.trim());
    if normalized.is_empty() {
        return false;
    }

    match lang {
        LanguageCode::KnIn => {
            // Must contain Kannada characters (U+0C80–U+0CFF)
            normalized
                .chars()
                .any(|c| ('\u{0C80}'..='\u{0CFF}').contains(&c))
        }
        LanguageCode::TaIn => {
            // Must contain Tamil characters (U+0B80–U+0BFF)
            normalized
                .chars()
                .any(|c| ('\u{0B80}'..='\u{0BFF}').contains(&c))
        }
        LanguageCode::EnUs => {
            // Must contain ASCII or Latin alphabetic characters
            normalized
                .chars()
                .any(|c| c.is_alphabetic() && (c.is_ascii_alphabetic() || c <= '\u{024F}'))
        }
        _ => !normalized.trim().is_empty(),
    }
}

/// Ingest and compile raw dictionary sources (JSONL, CSV, TSV) into the rich lexicon database.
pub fn compile_lexicon_file<P: AsRef<Path>>(
    path: P,
    lang: LanguageCode,
    storage: &StorageService,
    provenance_tag: &str,
) -> Result<ImportStats, LinguaError> {
    let start = Instant::now();
    let file = File::open(path.as_ref())
        .map_err(|e| LinguaError::Validation(format!("Failed to open lexicon file: {e}")))?;
    let reader = BufReader::new(file);

    let mut stats = ImportStats::default();
    let mut entries_batch: Vec<RichDictionaryEntry> = Vec::with_capacity(500);
    let mut seen_words: HashSet<String> = HashSet::new();

    for line_res in reader.lines() {
        let line = match line_res {
            Ok(l) => l,
            Err(_) => {
                stats.rejected_invalid += 1;
                continue;
            }
        };

        stats.total_read += 1;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Try JSONL parse (Kaikki/Wiktextract)
        let parsed_entry = if trimmed.starts_with('{') {
            parse_jsonl_line(trimmed, lang, provenance_tag)
        } else if trimmed.contains('\t') || trimmed.contains(',') {
            parse_delimited_line(trimmed, lang, provenance_tag)
        } else {
            None
        };

        match parsed_entry {
            Some(entry) => {
                if !validate_script_boundary(&entry.headword, lang) {
                    stats.rejected_invalid += 1;
                    continue;
                }

                if seen_words.contains(&entry.normalized_stem) {
                    stats.duplicates_skipped += 1;
                    continue;
                }

                seen_words.insert(entry.normalized_stem.clone());
                entries_batch.push(entry);
                stats.valid_entries += 1;

                if entries_batch.len() >= 500 {
                    storage.import_rich_lexicon_entries(&entries_batch)?;
                    entries_batch.clear();
                }
            }
            None => {
                stats.rejected_invalid += 1;
            }
        }
    }

    if !entries_batch.is_empty() {
        storage.import_rich_lexicon_entries(&entries_batch)?;
    }

    stats.duration_ms = start.elapsed().as_millis() as u64;
    info!(
        "Compiled lexicon file in {} ms: {} read, {} valid, {} duplicates, {} rejected",
        stats.duration_ms,
        stats.total_read,
        stats.valid_entries,
        stats.duplicates_skipped,
        stats.rejected_invalid
    );

    Ok(stats)
}

fn parse_jsonl_line(
    line: &str,
    lang: LanguageCode,
    provenance: &str,
) -> Option<RichDictionaryEntry> {
    let parsed: WiktextractEntry = serde_json::from_str(line).ok()?;
    let raw_word = parsed.word?;
    let cleaned_word = strip_wikitext_and_html(&raw_word);
    let normalized = normalize_text_nfc(&cleaned_word).to_lowercase();
    if normalized.is_empty() {
        return None;
    }

    let ipa = parsed
        .sounds
        .as_ref()
        .and_then(|s| s.iter().find_map(|snd| snd.ipa.clone()));

    let pos = parsed.pos.unwrap_or_else(|| "noun".into());

    let mut senses = Vec::new();
    let mut synonyms = Vec::new();
    let mut antonyms = Vec::new();
    let mut examples = Vec::new();

    if let Some(raw_senses) = parsed.senses {
        for (idx, s) in raw_senses.into_iter().enumerate() {
            let def = s
                .glosses
                .and_then(|g| g.into_iter().next())
                .map(|g| strip_wikitext_and_html(&g))
                .unwrap_or_default();

            if def.is_empty() {
                continue;
            }

            if let Some(syns) = s.synonyms {
                for lk in syns {
                    if let Some(w) = lk.word {
                        synonyms.push(w);
                    }
                }
            }
            if let Some(ants) = s.antonyms {
                for lk in ants {
                    if let Some(w) = lk.word {
                        antonyms.push(w);
                    }
                }
            }
            if let Some(exs) = s.examples {
                for ex in exs {
                    if let Some(txt) = ex.text {
                        examples.push(DictionaryExample {
                            source: strip_wikitext_and_html(&txt),
                            target: ex.english.unwrap_or_default(),
                        });
                    }
                }
            }

            senses.push(LexiconSense {
                sense_index: (idx + 1) as u32,
                part_of_speech: pos.clone(),
                definition: def,
                register: "general".into(),
                synonyms: Vec::new(),
                antonyms: Vec::new(),
                examples: Vec::new(),
            });
        }
    }

    if senses.is_empty() {
        return None;
    }

    let id = format!("dict-import-{}-{}", lang.as_bcp47(), hex_hash(&normalized));

    Some(RichDictionaryEntry {
        id,
        headword: cleaned_word,
        normalized_stem: normalized,
        language: lang,
        ipa_pronunciation: ipa,
        native_phonetic_respelling: None,
        part_of_speech: pos,
        gender_inflection_notes: None,
        register: "general".into(),
        senses,
        bilingual_examples: examples,
        synonyms,
        antonyms,
        derived_forms: Vec::new(),
        target_equivalents: HashMap::new(),
        frequency_rank: 500,
        provenance: provenance.to_string(),
    })
}

fn parse_delimited_line(
    line: &str,
    lang: LanguageCode,
    provenance: &str,
) -> Option<RichDictionaryEntry> {
    let parts: Vec<&str> = if line.contains('\t') {
        line.split('\t').collect()
    } else {
        line.split(',').collect()
    };

    if parts.len() < 2 {
        return None;
    }

    let headword = strip_wikitext_and_html(parts[0]);
    let normalized = normalize_text_nfc(&headword).to_lowercase();
    let definition = strip_wikitext_and_html(parts[1]);
    let pos = if parts.len() > 2 {
        parts[2].trim().to_string()
    } else {
        "noun".into()
    };
    let ipa = if parts.len() > 3 && !parts[3].trim().is_empty() {
        Some(parts[3].trim().to_string())
    } else {
        None
    };

    if headword.is_empty() || definition.is_empty() {
        return None;
    }

    let id = format!("dict-delim-{}-{}", lang.as_bcp47(), hex_hash(&normalized));

    Some(RichDictionaryEntry {
        id,
        headword,
        normalized_stem: normalized,
        language: lang,
        ipa_pronunciation: ipa,
        native_phonetic_respelling: None,
        part_of_speech: pos.clone(),
        gender_inflection_notes: None,
        register: "general".into(),
        senses: vec![LexiconSense {
            sense_index: 1,
            part_of_speech: pos,
            definition,
            register: "general".into(),
            synonyms: Vec::new(),
            antonyms: Vec::new(),
            examples: Vec::new(),
        }],
        bilingual_examples: Vec::new(),
        synonyms: Vec::new(),
        antonyms: Vec::new(),
        derived_forms: Vec::new(),
        target_equivalents: HashMap::new(),
        frequency_rank: 500,
        provenance: provenance.to_string(),
    })
}

fn hex_hash(s: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    let res = hasher.finalize();
    hex::encode(&res[..6])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_wikitext_and_html() {
        assert_eq!(strip_wikitext_and_html("<b>Hello</b> world"), "Hello world");
        assert_eq!(
            strip_wikitext_and_html("{{IPA|/test/}} adventure"),
            "adventure"
        );
        assert_eq!(
            strip_wikitext_and_html("[[Kannada]] language"),
            "Kannada language"
        );
    }

    #[test]
    fn test_validate_script_boundaries() {
        assert!(validate_script_boundary("ಸಾಹಸ", LanguageCode::KnIn));
        assert!(!validate_script_boundary("adventure", LanguageCode::KnIn));

        assert!(validate_script_boundary("சாகசம்", LanguageCode::TaIn));
        assert!(!validate_script_boundary("adventure", LanguageCode::TaIn));

        assert!(validate_script_boundary("adventure", LanguageCode::EnUs));
        assert!(!validate_script_boundary("ಸಾಹಸ", LanguageCode::EnUs));
    }
}
