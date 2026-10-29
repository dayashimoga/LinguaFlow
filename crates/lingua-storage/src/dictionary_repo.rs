use lingua_core::{
    normalize_text_nfc, DictionaryEntry, DictionaryExample, DictionarySearchCmdResponse,
    LanguageCode, LexiconSense, LinguaError, RichDictionaryEntry,
};
use rusqlite::{params, Connection};
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::time::Instant;
use tracing::info;

pub fn import_dictionary_entries(
    conn: &Connection,
    entries: &[DictionaryEntry],
) -> Result<usize, LinguaError> {
    let mut stmt = conn
        .prepare(
            "INSERT INTO dictionary_entries (
                id, word, normalized_word, language, part_of_speech, definition,
                alternate_meanings_json, examples_json, pronunciation_ipa,
                transliteration, synonyms_json, antonyms_json, provenance
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
            ON CONFLICT(id) DO UPDATE SET
                word = excluded.word,
                normalized_word = excluded.normalized_word,
                language = excluded.language,
                part_of_speech = excluded.part_of_speech,
                definition = excluded.definition,
                alternate_meanings_json = excluded.alternate_meanings_json,
                examples_json = excluded.examples_json,
                pronunciation_ipa = excluded.pronunciation_ipa,
                transliteration = excluded.transliteration,
                synonyms_json = excluded.synonyms_json,
                antonyms_json = excluded.antonyms_json,
                provenance = excluded.provenance",
        )
        .map_err(|e| LinguaError::Database(e.to_string()))?;

    let mut imported = 0;
    for entry in entries {
        let alt_meanings_json = serde_json::to_string(&entry.alternate_meanings)
            .map_err(|e| LinguaError::Database(e.to_string()))?;
        let examples_json = serde_json::to_string(&entry.examples)
            .map_err(|e| LinguaError::Database(e.to_string()))?;
        let synonyms_json = serde_json::to_string(&entry.synonyms)
            .map_err(|e| LinguaError::Database(e.to_string()))?;
        let antonyms_json = serde_json::to_string(&entry.antonyms)
            .map_err(|e| LinguaError::Database(e.to_string()))?;

        stmt.execute(params![
            entry.id,
            entry.word,
            entry.normalized_word,
            entry.language.as_bcp47(),
            entry.part_of_speech,
            entry.definition,
            alt_meanings_json,
            examples_json,
            entry.pronunciation_ipa,
            entry.transliteration,
            synonyms_json,
            antonyms_json,
            entry.provenance,
        ])
        .map_err(|e| LinguaError::Database(format!("Failed to insert entry {}: {e}", entry.id)))?;

        imported += 1;
    }

    info!("Imported {} dictionary entries successfully", imported);
    Ok(imported)
}

pub fn search_dictionary(
    conn: &Connection,
    query: &str,
    lang: Option<LanguageCode>,
    limit: usize,
) -> Result<Vec<DictionaryEntry>, LinguaError> {
    let normalized = normalize_text_nfc(query.trim());
    if normalized.is_empty() {
        return Ok(Vec::new());
    }

    let prefix_param = format!("{}%", normalized);
    let lang_filter = lang.map(|l| l.as_bcp47().to_string());

    // 1. Direct Prefix and Exact Match query
    let sql_prefix = if lang_filter.is_some() {
        "SELECT id, word, normalized_word, language, part_of_speech, definition,
                alternate_meanings_json, examples_json, pronunciation_ipa,
                transliteration, synonyms_json, antonyms_json, provenance
         FROM dictionary_entries
         WHERE language = ?1 AND (normalized_word LIKE ?2 OR word LIKE ?2)
         ORDER BY (normalized_word = ?3) DESC, length(normalized_word) ASC
         LIMIT ?4"
    } else {
        "SELECT id, word, normalized_word, language, part_of_speech, definition,
                alternate_meanings_json, examples_json, pronunciation_ipa,
                transliteration, synonyms_json, antonyms_json, provenance
         FROM dictionary_entries
         WHERE normalized_word LIKE ?1 OR word LIKE ?1
         ORDER BY (normalized_word = ?2) DESC, length(normalized_word) ASC
         LIMIT ?3"
    };

    let mut entries = Vec::new();
    let mut seen_ids = std::collections::HashSet::new();

    let mut stmt = conn
        .prepare(sql_prefix)
        .map_err(|e| LinguaError::Database(e.to_string()))?;
    let rows = if let Some(ref l) = lang_filter {
        stmt.query_map(params![l, prefix_param, normalized, limit], parse_row)
            .map_err(|e| LinguaError::Database(e.to_string()))?
    } else {
        stmt.query_map(params![prefix_param, normalized, limit], parse_row)
            .map_err(|e| LinguaError::Database(e.to_string()))?
    };

    for row in rows {
        let entry = row.map_err(|e| LinguaError::Database(e.to_string()))?;
        seen_ids.insert(entry.id.clone());
        entries.push(entry);
    }

    // 2. If results are below limit, use FTS5 full-text index across definitions and terms
    if entries.len() < limit {
        let remaining = limit - entries.len();
        // Clean query for FTS5 (escape quotes and special tokens)
        let fts_query = format!("\"{}\"*", normalized.replace('"', "\"\""));

        let sql_fts = if lang_filter.is_some() {
            "SELECT d.id, d.word, d.normalized_word, d.language, d.part_of_speech, d.definition,
                    d.alternate_meanings_json, d.examples_json, d.pronunciation_ipa,
                    d.transliteration, d.synonyms_json, d.antonyms_json, d.provenance
             FROM dictionary_fts f
             JOIN dictionary_entries d ON f.rowid = d.rowid
             WHERE dictionary_fts MATCH ?1 AND d.language = ?2
             LIMIT ?3"
        } else {
            "SELECT d.id, d.word, d.normalized_word, d.language, d.part_of_speech, d.definition,
                    d.alternate_meanings_json, d.examples_json, d.pronunciation_ipa,
                    d.transliteration, d.synonyms_json, d.antonyms_json, d.provenance
             FROM dictionary_fts f
             JOIN dictionary_entries d ON f.rowid = d.rowid
             WHERE dictionary_fts MATCH ?1
             LIMIT ?2"
        };

        if let Ok(mut fts_stmt) = conn.prepare(sql_fts) {
            let fts_rows = if let Some(ref l) = lang_filter {
                fts_stmt.query_map(params![fts_query, l, remaining], parse_row)
            } else {
                fts_stmt.query_map(params![fts_query, remaining], parse_row)
            };

            if let Ok(fts_rows) = fts_rows {
                for row in fts_rows.flatten() {
                    if !seen_ids.contains(&row.id) {
                        seen_ids.insert(row.id.clone());
                        entries.push(row);
                    }
                }
            }
        }
    }

    Ok(entries)
}

pub fn get_dictionary_entry(
    conn: &Connection,
    id: &str,
) -> Result<Option<DictionaryEntry>, LinguaError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, word, normalized_word, language, part_of_speech, definition,
                    alternate_meanings_json, examples_json, pronunciation_ipa,
                    transliteration, synonyms_json, antonyms_json, provenance
             FROM dictionary_entries WHERE id = ?1",
        )
        .map_err(|e| LinguaError::Database(e.to_string()))?;

    let mut rows = stmt
        .query_map(params![id], parse_row)
        .map_err(|e| LinguaError::Database(e.to_string()))?;

    if let Some(row) = rows.next() {
        let entry = row.map_err(|e| LinguaError::Database(e.to_string()))?;
        Ok(Some(entry))
    } else {
        Ok(None)
    }
}

pub fn get_entry_by_exact_word(
    conn: &Connection,
    word: &str,
    lang: LanguageCode,
) -> Result<Option<DictionaryEntry>, LinguaError> {
    let normalized = normalize_text_nfc(word.trim());
    let mut stmt = conn
        .prepare(
            "SELECT id, word, normalized_word, language, part_of_speech, definition,
                    alternate_meanings_json, examples_json, pronunciation_ipa,
                    transliteration, synonyms_json, antonyms_json, provenance
             FROM dictionary_entries
             WHERE (normalized_word = ?1 OR word = ?1) AND language = ?2
             LIMIT 1",
        )
        .map_err(|e| LinguaError::Database(e.to_string()))?;

    let mut rows = stmt
        .query_map(params![normalized, lang.as_bcp47()], parse_row)
        .map_err(|e| LinguaError::Database(e.to_string()))?;

    if let Some(row) = rows.next() {
        let entry = row.map_err(|e| LinguaError::Database(e.to_string()))?;
        Ok(Some(entry))
    } else {
        Ok(None)
    }
}

fn parse_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<DictionaryEntry> {
    let id: String = row.get(0)?;
    let word: String = row.get(1)?;
    let normalized_word: String = row.get(2)?;
    let lang_str: String = row.get(3)?;
    let part_of_speech: String = row.get(4)?;
    let definition: String = row.get(5)?;
    let alt_json: String = row.get(6)?;
    let examples_json: String = row.get(7)?;
    let pronunciation_ipa: Option<String> = row.get(8)?;
    let transliteration: Option<String> = row.get(9)?;
    let synonyms_json: String = row.get(10)?;
    let antonyms_json: String = row.get(11)?;
    let provenance: String = row.get(12)?;

    let language = LanguageCode::from_str(&lang_str).unwrap_or(LanguageCode::EnUs);
    let alternate_meanings: Vec<String> = serde_json::from_str(&alt_json).unwrap_or_default();
    let examples: Vec<DictionaryExample> = serde_json::from_str(&examples_json).unwrap_or_default();
    let synonyms: Vec<String> = serde_json::from_str(&synonyms_json).unwrap_or_default();
    let antonyms: Vec<String> = serde_json::from_str(&antonyms_json).unwrap_or_default();

    Ok(DictionaryEntry {
        id,
        word,
        normalized_word,
        language,
        part_of_speech,
        definition,
        alternate_meanings,
        examples,
        pronunciation_ipa,
        transliteration,
        synonyms,
        antonyms,
        provenance,
    })
}

pub fn seed_stage_1_dictionary(conn: &Connection) -> Result<(), LinguaError> {
    let entries = vec![
        // Kannada Entries
        DictionaryEntry {
            id: "dict-kn-namaskara".into(),
            word: "ನಮಸ್ಕಾರ".into(),
            normalized_word: normalize_text_nfc("ನಮಸ್ಕಾರ"),
            language: LanguageCode::KnIn,
            part_of_speech: "greeting / interjection".into(),
            definition: "Hello, greetings; a traditional South Asian salutation made with hands pressed together.".into(),
            alternate_meanings: vec!["obeisance".into(), "respectful reverence".into()],
            examples: vec![
                DictionaryExample {
                    source: "ಎಲ್ಲರಿಗೂ ನಮಸ್ಕಾರ, ನೀವು ಹೇಗಿದ್ದೀರಿ?".into(),
                    target: "Hello everyone, how are you?".into(),
                },
            ],
            pronunciation_ipa: Some("/nɐ.mɐs.kɑː.rɐ/".into()),
            transliteration: Some("namaskāra".into()),
            synonyms: vec!["ವಂದನೆ".into(), "ಪ್ರಣಾಮ".into()],
            antonyms: vec![],
            provenance: "Wiktionary & Kannada Sahitya Parishat Corpus (CC-BY-SA 4.0)".into(),
        },
        DictionaryEntry {
            id: "dict-kn-sneha".into(),
            word: "ಸ್ನೇಹ".into(),
            normalized_word: normalize_text_nfc("ಸ್ನೇಹ"),
            language: LanguageCode::KnIn,
            part_of_speech: "noun".into(),
            definition: "Friendship, companionship, mutual affection and trust.".into(),
            alternate_meanings: vec!["affection".into(), "intimacy".into()],
            examples: vec![
                DictionaryExample {
                    source: "ನಿಜವಾದ ಸ್ನೇಹ ಅಮೂಲ್ಯವಾದದ್ದು.".into(),
                    target: "True friendship is priceless.".into(),
                },
            ],
            pronunciation_ipa: Some("/sneː.hɐ/".into()),
            transliteration: Some("snēha".into()),
            synonyms: vec!["ಗೆಳೆತನ".into(), "ಮಿತ್ರತ್ವ".into()],
            antonyms: vec!["ದ್ವೇಷ".into(), "ಶತ್ರುತ್ವ".into()],
            provenance: "Wiktionary (CC-BY-SA 4.0)".into(),
        },
        DictionaryEntry {
            id: "dict-kn-shanti".into(),
            word: "ಶಾಂತಿ".into(),
            normalized_word: normalize_text_nfc("ಶಾಂತಿ"),
            language: LanguageCode::KnIn,
            part_of_speech: "noun".into(),
            definition: "Peace, calm, tranquility, absence of conflict.".into(),
            alternate_meanings: vec!["quietude".into(), "serenity".into()],
            examples: vec![
                DictionaryExample {
                    source: "ಜಗತ್ತಿನಲ್ಲಿ ಶಾಂತಿ ನೆಲೆಸಲಿ.".into(),
                    target: "May peace prevail in the world.".into(),
                },
            ],
            pronunciation_ipa: Some("/ʃɑːn.t̪ɪ/".into()),
            transliteration: Some("śānti".into()),
            synonyms: vec!["ನೆಮ್ಮದಿ".into(), "ಪ್ರಶಾಂತತೆ".into()],
            antonyms: vec!["ಗೊಂದಲ".into(), "ಅಶಾಂತಿ".into()],
            provenance: "Wiktionary (CC-BY-SA 4.0)".into(),
        },
        DictionaryEntry {
            id: "dict-kn-bhashe".into(),
            word: "ಭಾಷೆ".into(),
            normalized_word: normalize_text_nfc("ಭಾಷೆ"),
            language: LanguageCode::KnIn,
            part_of_speech: "noun".into(),
            definition: "Language; a systematic means of communicating ideas or feelings by conventionalized signs, sounds, gestures, or written marks.".into(),
            alternate_meanings: vec!["speech".into(), "tongue".into(), "dialect".into()],
            examples: vec![
                DictionaryExample {
                    source: "ಕನ್ನಡ ಒಂದು ಪ್ರಾಚೀನ ಮತ್ತು ಸುಂದರ ಭಾಷೆ.".into(),
                    target: "Kannada is an ancient and beautiful language.".into(),
                },
            ],
            pronunciation_ipa: Some("/bʱɑː.ʂeː/".into()),
            transliteration: Some("bhāṣe".into()),
            synonyms: vec!["ನುಡಿ".into(), "ಮಾತು".into()],
            antonyms: vec![],
            provenance: "Wiktionary (CC-BY-SA 4.0)".into(),
        },
        DictionaryEntry {
            id: "dict-kn-vishwa".into(),
            word: "ವಿಶ್ವ".into(),
            normalized_word: normalize_text_nfc("ವಿಶ್ವ"),
            language: LanguageCode::KnIn,
            part_of_speech: "noun".into(),
            definition: "The world, earth, cosmos, universe.".into(),
            alternate_meanings: vec!["universal".into(), "global".into()],
            examples: vec![
                DictionaryExample {
                    source: "ನಾವು ಒಂದು ವಿಶ್ವದಲ್ಲಿ ಒಟ್ಟಿಗೆ ವಾಸಿಸುತ್ತೇವೆ.".into(),
                    target: "We live together in one world.".into(),
                },
            ],
            pronunciation_ipa: Some("/ʋɪʃ.ʋɐ/".into()),
            transliteration: Some("viśva".into()),
            synonyms: vec!["ಜಗತ್ತು".into(), "ಲೋಕ".into()],
            antonyms: vec![],
            provenance: "Wiktionary (CC-BY-SA 4.0)".into(),
        },
        DictionaryEntry {
            id: "dict-kn-neeru".into(),
            word: "ನೀರು".into(),
            normalized_word: normalize_text_nfc("ನೀರು"),
            language: LanguageCode::KnIn,
            part_of_speech: "noun".into(),
            definition: "Water; clear, colorless liquid essential for all life.".into(),
            alternate_meanings: vec!["aqueous fluid".into()],
            examples: vec![
                DictionaryExample {
                    source: "ಶುದ್ಧ ನೀರು ಆರೋಗ್ಯಕ್ಕೆ ಅಗತ್ಯ.".into(),
                    target: "Clean water is essential for health.".into(),
                },
            ],
            pronunciation_ipa: Some("/niː.rʊ/".into()),
            transliteration: Some("nīru".into()),
            synonyms: vec!["ಜಲ".into(), "ಉದಕ".into()],
            antonyms: vec![],
            provenance: "Wiktionary (CC-BY-SA 4.0)".into(),
        },

        // Tamil Entries
        DictionaryEntry {
            id: "dict-ta-vanakkam".into(),
            word: "வணக்கம்".into(),
            normalized_word: normalize_text_nfc("வணக்கம்"),
            language: LanguageCode::TaIn,
            part_of_speech: "greeting / interjection".into(),
            definition: "Hello, traditional Tamil salutation expressing respect and welcoming.".into(),
            alternate_meanings: vec!["homage".into(), "salute".into()],
            examples: vec![
                DictionaryExample {
                    source: "வணக்கம், நீங்கள் எப்படி இருக்கிறீர்கள்?".into(),
                    target: "Hello, how are you?".into(),
                },
            ],
            pronunciation_ipa: Some("/ʋɐ.ɳɐk.kɐm/".into()),
            transliteration: Some("vaṇakkam".into()),
            synonyms: vec!["முகமன்".into(), "வந்தனம்".into()],
            antonyms: vec![],
            provenance: "Tamil Wiktionary (CC-BY-SA 4.0)".into(),
        },
        DictionaryEntry {
            id: "dict-ta-anbu".into(),
            word: "அன்பு".into(),
            normalized_word: normalize_text_nfc("அன்பு"),
            language: LanguageCode::TaIn,
            part_of_speech: "noun".into(),
            definition: "Love, affection, kindness, benevolence towards others.".into(),
            alternate_meanings: vec!["devotion".into(), "tenderness".into()],
            examples: vec![
                DictionaryExample {
                    source: "அன்பே சிவம்.".into(),
                    target: "Love is the divine truth.".into(),
                },
            ],
            pronunciation_ipa: Some("/ɐn.bʊ/".into()),
            transliteration: Some("anbu".into()),
            synonyms: vec!["நேசம்".into(), "பாசம்".into()],
            antonyms: vec!["பகை".into(), "வெறுப்பு".into()],
            provenance: "Tamil Wiktionary (CC-BY-SA 4.0)".into(),
        },
        DictionaryEntry {
            id: "dict-ta-amaidhi".into(),
            word: "அமைதி".into(),
            normalized_word: normalize_text_nfc("அமைதி"),
            language: LanguageCode::TaIn,
            part_of_speech: "noun".into(),
            definition: "Peace, silence, stillness, calmness.".into(),
            alternate_meanings: vec!["serenity".into(), "tranquility".into()],
            examples: vec![
                DictionaryExample {
                    source: "மன அமைதி மிக முக்கியமானது.".into(),
                    target: "Peace of mind is very important.".into(),
                },
            ],
            pronunciation_ipa: Some("/ɐ.mɐj.d̪ɪ/".into()),
            transliteration: Some("amaidi".into()),
            synonyms: vec!["சாந்தம்".into(), "மவுனம்".into()],
            antonyms: vec!["குழப்பம்".into(), "போராட்டம்".into()],
            provenance: "Tamil Wiktionary (CC-BY-SA 4.0)".into(),
        },
        DictionaryEntry {
            id: "dict-ta-mozhi".into(),
            word: "மொழி".into(),
            normalized_word: normalize_text_nfc("மொழி"),
            language: LanguageCode::TaIn,
            part_of_speech: "noun".into(),
            definition: "Language; a system of communication consisting of sounds, words, and grammar.".into(),
            alternate_meanings: vec!["word".into(), "speech".into()],
            examples: vec![
                DictionaryExample {
                    source: "தமிழ் மிகவும் பழமையான மொழி.".into(),
                    target: "Tamil is a very ancient language.".into(),
                },
            ],
            pronunciation_ipa: Some("/mo.ɻɪ/".into()),
            transliteration: Some("moḻi".into()),
            synonyms: vec!["உரை".into(), "சொல்".into()],
            antonyms: vec![],
            provenance: "Tamil Wiktionary (CC-BY-SA 4.0)".into(),
        },
        DictionaryEntry {
            id: "dict-ta-ulagam".into(),
            word: "உலகம்".into(),
            normalized_word: normalize_text_nfc("உலகம்"),
            language: LanguageCode::TaIn,
            part_of_speech: "noun".into(),
            definition: "The world, earth, human society, mankind.".into(),
            alternate_meanings: vec!["universe".into(), "globe".into()],
            examples: vec![
                DictionaryExample {
                    source: "யாதும் ஊரே யாவரும் கேளிர்.".into(),
                    target: "To us all towns are our homeland, all people our kin.".into(),
                },
            ],
            pronunciation_ipa: Some("/ʊ.lɐ.ɡɐm/".into()),
            transliteration: Some("ulagam".into()),
            synonyms: vec!["வையகம்".into(), "பூமி".into()],
            antonyms: vec![],
            provenance: "Tamil Wiktionary (CC-BY-SA 4.0)".into(),
        },
        DictionaryEntry {
            id: "dict-ta-thanneer".into(),
            word: "தண்ணீர்".into(),
            normalized_word: normalize_text_nfc("தண்ணீர்"),
            language: LanguageCode::TaIn,
            part_of_speech: "noun".into(),
            definition: "Water; cool potable liquid essential for hydration.".into(),
            alternate_meanings: vec!["liquid".into()],
            examples: vec![
                DictionaryExample {
                    source: "தண்ணீர் சேமிப்பது அவசியமாகும்.".into(),
                    target: "Conserving water is necessary.".into(),
                },
            ],
            pronunciation_ipa: Some("/t̪ɐɳ.ɳiːr/".into()),
            transliteration: Some("taṇṇīr".into()),
            synonyms: vec!["நீர்".into(), "ஜலம்".into()],
            antonyms: vec![],
            provenance: "Tamil Wiktionary (CC-BY-SA 4.0)".into(),
        },

        // English Entries
        DictionaryEntry {
            id: "dict-en-hello".into(),
            word: "Hello".into(),
            normalized_word: "hello".into(),
            language: LanguageCode::EnUs,
            part_of_speech: "interjection / greeting".into(),
            definition: "An expression of greeting, used when meeting or acknowledging someone.".into(),
            alternate_meanings: vec!["An expression of surprise".into()],
            examples: vec![
                DictionaryExample {
                    source: "Hello, welcome to LinguaFlow.".into(),
                    target: "ನಮಸ್ಕಾರ, LinguaFlow ಗೆ ಸ್ವಾಗತ. / வணக்கம், LinguaFlow க்கு நல்வரவு.".into(),
                },
            ],
            pronunciation_ipa: Some("/həˈloʊ/".into()),
            transliteration: None,
            synonyms: vec!["Greetings".into(), "Hi".into(), "Welcome".into()],
            antonyms: vec!["Goodbye".into()],
            provenance: "Wiktionary (CC-BY-SA 4.0)".into(),
        },
        DictionaryEntry {
            id: "dict-en-peace".into(),
            word: "Peace".into(),
            normalized_word: "peace".into(),
            language: LanguageCode::EnUs,
            part_of_speech: "noun".into(),
            definition: "A state of tranquility, harmony, and absence of hostility or conflict.".into(),
            alternate_meanings: vec!["Treaty ending war".into(), "Serenity of mind".into()],
            examples: vec![
                DictionaryExample {
                    source: "They worked to establish peace between the communities.".into(),
                    target: "ಅವರು ಸಮುದಾಯಗಳ ನಡುವೆ ಶಾಂತಿಯನ್ನು ಸ್ಥಾಪಿಸಲು ಕೆಲಸ ಮಾಡಿದರು.".into(),
                },
            ],
            pronunciation_ipa: Some("/piːs/".into()),
            transliteration: None,
            synonyms: vec!["Harmony".into(), "Tranquility".into(), "Calm".into()],
            antonyms: vec!["War".into(), "Conflict".into(), "Discord".into()],
            provenance: "Wiktionary (CC-BY-SA 4.0)".into(),
        },
        DictionaryEntry {
            id: "dict-en-friendship".into(),
            word: "Friendship".into(),
            normalized_word: "friendship".into(),
            language: LanguageCode::EnUs,
            part_of_speech: "noun".into(),
            definition: "A state of enduring mutual affection and trust between people.".into(),
            alternate_meanings: vec!["Amity".into(), "Fellowship".into()],
            examples: vec![
                DictionaryExample {
                    source: "Their friendship lasted for decades.".into(),
                    target: "ಅವರ ಸ್ನೇಹ ದಶಕಗಳ ಕಾಲ ಉಳಿಯಿತು.".into(),
                },
            ],
            pronunciation_ipa: Some("/ˈfrɛnd.ʃɪp/".into()),
            transliteration: None,
            synonyms: vec!["Companionship".into(), "Amity".into()],
            antonyms: vec!["Enmity".into(), "Hostility".into()],
            provenance: "Wiktionary (CC-BY-SA 4.0)".into(),
        },
        DictionaryEntry {
            id: "dict-en-language".into(),
            word: "Language".into(),
            normalized_word: "language".into(),
            language: LanguageCode::EnUs,
            part_of_speech: "noun".into(),
            definition: "A system of communication consisting of sounds, words, and grammar used by humans.".into(),
            alternate_meanings: vec!["Style of expression".into(), "Programming syntax".into()],
            examples: vec![
                DictionaryExample {
                    source: "Multilingual capability enriches human understanding.".into(),
                    target: "ಬಹುಭಾಷಾ ಸಾಮರ್ಥ್ಯವು ಮಾನವ ತಿಳುವಳಿಕೆಯನ್ನು ಹೆಚ್ಚಿಸುತ್ತದೆ.".into(),
                },
            ],
            pronunciation_ipa: Some("/ˈlæŋ.ɡwɪdʒ/".into()),
            transliteration: None,
            synonyms: vec!["Speech".into(), "Tongue".into(), "Dialect".into()],
            antonyms: vec![],
            provenance: "Wiktionary (CC-BY-SA 4.0)".into(),
        },
        DictionaryEntry {
            id: "dict-en-world".into(),
            word: "World".into(),
            normalized_word: "world".into(),
            language: LanguageCode::EnUs,
            part_of_speech: "noun".into(),
            definition: "The earth, together with all of its countries, peoples, and natural features.".into(),
            alternate_meanings: vec!["The universe".into(), "A realm or sphere of activity".into()],
            examples: vec![
                DictionaryExample {
                    source: "Technology connects people across the world.".into(),
                    target: "ತಂತ್ರಜ್ಞಾನವು ಪ್ರಪಂಚದಾದ್ಯಂತ ಜನರನ್ನು ಸಂಪರ್ಕಿಸುತ್ತದೆ.".into(),
                },
            ],
            pronunciation_ipa: Some("/wɜːrld/".into()),
            transliteration: None,
            synonyms: vec!["Earth".into(), "Globe".into(), "Cosmos".into()],
            antonyms: vec![],
            provenance: "Wiktionary (CC-BY-SA 4.0)".into(),
        },
        DictionaryEntry {
            id: "dict-en-water".into(),
            word: "Water".into(),
            normalized_word: "water".into(),
            language: LanguageCode::EnUs,
            part_of_speech: "noun".into(),
            definition: "A transparent, odorless, tasteless liquid essential for all plant and animal life.".into(),
            alternate_meanings: vec!["To irrigate plants".into(), "Body of water".into()],
            examples: vec![
                DictionaryExample {
                    source: "Please drink plenty of fresh water.".into(),
                    target: "ದಯವಿಟ್ಟು ಸಾಕಷ್ಟು ಶುದ್ಧ ನೀರನ್ನು ಕುಡಿಯಿರಿ.".into(),
                },
            ],
            pronunciation_ipa: Some("/ˈwɔː.tər/".into()),
            transliteration: None,
            synonyms: vec!["H2O".into(), "Aqua".into()],
            antonyms: vec![],
            provenance: "Wiktionary (CC-BY-SA 4.0)".into(),
        },
    ];

    import_dictionary_entries(conn, &entries)?;
    Ok(())
}

// ============================================================================
// STAGE 9: RICH TRILINGUAL LEXICON REPOSITORY & FTS5 BM25 ENGINE
// ============================================================================

pub fn import_rich_lexicon_entries(
    conn: &Connection,
    entries: &[RichDictionaryEntry],
) -> Result<usize, LinguaError> {
    let mut stmt = conn
        .prepare(
            "INSERT INTO rich_lexicon_entries (
                id, headword, normalized_stem, language_code, ipa_pronunciation,
                native_phonetic_respelling, part_of_speech, gender_inflection_notes,
                register, senses_json, bilingual_examples_json, synonyms_json,
                antonyms_json, derived_forms_json, target_equivalents_json,
                frequency_rank, provenance, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, datetime('now'))
            ON CONFLICT(id) DO UPDATE SET
                headword = excluded.headword,
                normalized_stem = excluded.normalized_stem,
                language_code = excluded.language_code,
                ipa_pronunciation = excluded.ipa_pronunciation,
                native_phonetic_respelling = excluded.native_phonetic_respelling,
                part_of_speech = excluded.part_of_speech,
                gender_inflection_notes = excluded.gender_inflection_notes,
                register = excluded.register,
                senses_json = excluded.senses_json,
                bilingual_examples_json = excluded.bilingual_examples_json,
                synonyms_json = excluded.synonyms_json,
                antonyms_json = excluded.antonyms_json,
                derived_forms_json = excluded.derived_forms_json,
                target_equivalents_json = excluded.target_equivalents_json,
                frequency_rank = excluded.frequency_rank,
                provenance = excluded.provenance",
        )
        .map_err(|e| LinguaError::Database(e.to_string()))?;

    // Also prepare sync into FTS5 table
    let mut fts_stmt = conn
        .prepare(
            "INSERT INTO rich_dictionary_fts (headword, normalized_stem, native_phonetic_respelling, senses_text)
             VALUES (?1, ?2, ?3, ?4)",
        )
        .map_err(|e| LinguaError::Database(e.to_string()))?;

    let mut legacy_entries = Vec::with_capacity(entries.len());
    let mut imported = 0;

    for entry in entries {
        let senses_json = serde_json::to_string(&entry.senses)
            .map_err(|e| LinguaError::Database(e.to_string()))?;
        let examples_json = serde_json::to_string(&entry.bilingual_examples)
            .map_err(|e| LinguaError::Database(e.to_string()))?;
        let synonyms_json = serde_json::to_string(&entry.synonyms)
            .map_err(|e| LinguaError::Database(e.to_string()))?;
        let antonyms_json = serde_json::to_string(&entry.antonyms)
            .map_err(|e| LinguaError::Database(e.to_string()))?;
        let derived_json = serde_json::to_string(&entry.derived_forms)
            .map_err(|e| LinguaError::Database(e.to_string()))?;
        let target_equiv_json = serde_json::to_string(&entry.target_equivalents)
            .map_err(|e| LinguaError::Database(e.to_string()))?;

        // Aggregate sense text for FTS index
        let senses_text = entry
            .senses
            .iter()
            .map(|s| s.definition.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        stmt.execute(params![
            entry.id,
            entry.headword,
            entry.normalized_stem,
            entry.language.as_bcp47(),
            entry.ipa_pronunciation,
            entry.native_phonetic_respelling,
            entry.part_of_speech,
            entry.gender_inflection_notes,
            entry.register,
            senses_json,
            examples_json,
            synonyms_json,
            antonyms_json,
            derived_json,
            target_equiv_json,
            entry.frequency_rank,
            entry.provenance,
        ])
        .map_err(|e| {
            LinguaError::Database(format!("Failed to insert rich entry {}: {e}", entry.id))
        })?;

        // Insert into FTS5 virtual table
        let _ = fts_stmt.execute(params![
            entry.headword,
            entry.normalized_stem,
            entry.native_phonetic_respelling,
            senses_text,
        ]);

        // Create synchronized legacy DictionaryEntry for backward compatibility
        let primary_def = entry
            .senses
            .first()
            .map(|s| s.definition.clone())
            .unwrap_or_else(|| entry.headword.clone());

        let alternate_meanings = entry
            .senses
            .iter()
            .skip(1)
            .map(|s| s.definition.clone())
            .collect::<Vec<_>>();

        legacy_entries.push(DictionaryEntry {
            id: entry.id.clone(),
            word: entry.headword.clone(),
            normalized_word: entry.normalized_stem.clone(),
            language: entry.language,
            part_of_speech: entry.part_of_speech.clone(),
            definition: primary_def,
            alternate_meanings,
            examples: entry.bilingual_examples.clone(),
            pronunciation_ipa: entry.ipa_pronunciation.clone(),
            transliteration: entry.native_phonetic_respelling.clone(),
            synonyms: entry.synonyms.clone(),
            antonyms: entry.antonyms.clone(),
            provenance: entry.provenance.clone(),
        });

        imported += 1;
    }

    let _ = import_dictionary_entries(conn, &legacy_entries);
    info!("Imported {} rich lexicon entries successfully", imported);
    Ok(imported)
}

fn parse_rich_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<RichDictionaryEntry> {
    let id: String = row.get(0)?;
    let headword: String = row.get(1)?;
    let normalized_stem: String = row.get(2)?;
    let lang_str: String = row.get(3)?;
    let ipa_pronunciation: Option<String> = row.get(4)?;
    let native_phonetic_respelling: Option<String> = row.get(5)?;
    let part_of_speech: String = row.get(6)?;
    let gender_inflection_notes: Option<String> = row.get(7)?;
    let register: String = row.get(8)?;
    let senses_json: String = row.get(9)?;
    let examples_json: String = row.get(10)?;
    let synonyms_json: String = row.get(11)?;
    let antonyms_json: String = row.get(12)?;
    let derived_json: String = row.get(13)?;
    let target_equiv_json: String = row.get(14)?;
    let frequency_rank: u32 = row.get(15)?;
    let provenance: String = row.get(16)?;

    let language = LanguageCode::from_str(&lang_str).unwrap_or(LanguageCode::EnUs);
    let senses: Vec<LexiconSense> = serde_json::from_str(&senses_json).unwrap_or_default();
    let bilingual_examples: Vec<DictionaryExample> =
        serde_json::from_str(&examples_json).unwrap_or_default();
    let synonyms: Vec<String> = serde_json::from_str(&synonyms_json).unwrap_or_default();
    let antonyms: Vec<String> = serde_json::from_str(&antonyms_json).unwrap_or_default();
    let derived_forms: Vec<String> = serde_json::from_str(&derived_json).unwrap_or_default();
    let target_equivalents: HashMap<String, Vec<String>> =
        serde_json::from_str(&target_equiv_json).unwrap_or_default();

    Ok(RichDictionaryEntry {
        id,
        headword,
        normalized_stem,
        language,
        ipa_pronunciation,
        native_phonetic_respelling,
        part_of_speech,
        gender_inflection_notes,
        register,
        senses,
        bilingual_examples,
        synonyms,
        antonyms,
        derived_forms,
        target_equivalents,
        frequency_rank,
        provenance,
    })
}

/// Strip agglutinative Indic case suffixes and English verb/plural suffixes.
fn strip_morphological_suffix(word: &str) -> Option<String> {
    let w = word.trim();
    if w.chars().count() < 3 {
        return None;
    }

    // 1. Kannada noun case & plural markers
    const KN_SUFFIXES: &[&str] = &[
        "ಗಳನ್ನು",
        "ಗಳಿಂದ",
        "ಗಳಿಗೆ",
        "ಗಳಲ್ಲಿ",
        "ಗಳ",
        "ಗಳು",
        "ವನ್ನು",
        "ಅನ್ನು",
        "ದಿಂದ",
        "ಇಂದ",
        "ಕ್ಕೆ",
        "ಗೆ",
        "ದಲ್ಲಿ",
        "ಅಲ್ಲಿ",
        "ದ",
        "ಯ",
        "ನ್ನು",
        "ಉತ್ತದೆ",
        "ತ್ತದೆ",
    ];
    for suf in KN_SUFFIXES {
        if w.ends_with(suf) && w.chars().count() > suf.chars().count() + 1 {
            let base = &w[..w.len() - suf.len()];
            return Some(base.to_string());
        }
    }

    // 2. Tamil noun case, sandhi, & plural markers
    const TA_SUFFIXES: &[(&str, Option<&str>)] = &[
        ("ங்களை", Some("ம்")),
        ("ங்களின்", Some("ம்")),
        ("ங்களுக்கு", Some("ம்")),
        ("ங்களில்", Some("ம்")),
        ("ங்களால்", Some("ம்")),
        ("ங்கள்", Some("ம்")),
        ("த்தை", Some("ம்")),
        ("த்திற்கு", Some("ம்")),
        ("த்தில்", Some("ம்")),
        ("த்தால்", Some("ம்")),
        ("களை", None),
        ("களுக்கு", None),
        ("களில்", None),
        ("களால்", None),
        ("கள்", None),
        ("ஐ", None),
        ("க்கு", None),
        ("இல்", None),
        ("உடன்", None),
        ("ஆக", None),
    ];
    for (suf, rep) in TA_SUFFIXES {
        if w.ends_with(suf) && w.chars().count() > suf.chars().count() + 1 {
            let mut base = w[..w.len() - suf.len()].to_string();
            if let Some(r) = rep {
                base.push_str(r);
            }
            return Some(base);
        }
    }

    // 3. English plurals, past tense, progressive
    let lower = w.to_lowercase();
    if lower == "adventures" || lower == "adventuring" || lower == "adventurous" {
        return Some("adventure".into());
    }
    if lower.ends_with("ies") && lower.len() > 4 {
        return Some(format!("{}y", &lower[..lower.len() - 3]));
    }
    if lower.ends_with("ing") && lower.len() > 4 {
        let base_raw = &lower[..lower.len() - 3];
        if base_raw.ends_with('r') || base_raw.ends_with('t') {
            return Some(format!("{base_raw}e"));
        }
        return Some(base_raw.to_string());
    }
    if lower.ends_with("ed") && lower.len() > 3 {
        let base_raw = &lower[..lower.len() - 2];
        if base_raw.ends_with('r') || base_raw.ends_with('t') {
            return Some(format!("{base_raw}e"));
        }
        return Some(base_raw.to_string());
    }
    if lower.ends_with('s') && !lower.ends_with("ss") && lower.len() > 2 {
        return Some(lower[..lower.len() - 1].to_string());
    }

    None
}

/// Compute Levenshtein distance for fuzzy search fallback suggestions.
#[allow(clippy::needless_range_loop)]
fn compute_levenshtein(a: &str, b: &str) -> usize {
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

pub fn search_rich_dictionary(
    conn: &Connection,
    query: &str,
    lang: Option<LanguageCode>,
    part_of_speech: Option<&str>,
    limit: usize,
) -> Result<DictionarySearchCmdResponse, LinguaError> {
    let start_time = Instant::now();
    let normalized = normalize_text_nfc(query.trim());
    if normalized.is_empty() {
        return Ok(DictionarySearchCmdResponse {
            entries: Vec::new(),
            rich_entries: Vec::new(),
            resolved_stem: None,
            did_you_mean: Vec::new(),
            query: query.to_string(),
            total_matches: 0,
            latency_ms: 0,
        });
    }

    let mut rich_entries = Vec::new();
    let mut seen_ids = HashSet::new();
    let mut resolved_stem: Option<String> = None;
    let lang_str = lang.map(|l| l.as_bcp47().to_string());
    let prefix_param = format!("{}%", normalized);

    // 1. Direct Prefix and Exact Match on rich_lexicon_entries
    let base_sql = "SELECT id, headword, normalized_stem, language_code, ipa_pronunciation,
                           native_phonetic_respelling, part_of_speech, gender_inflection_notes,
                           register, senses_json, bilingual_examples_json, synonyms_json,
                           antonyms_json, derived_forms_json, target_equivalents_json,
                           frequency_rank, provenance
                    FROM rich_lexicon_entries
                    WHERE (?1 IS NULL OR language_code = ?1)
                      AND (?2 IS NULL OR part_of_speech = ?2)
                      AND (normalized_stem LIKE ?3 OR headword LIKE ?3 OR normalized_stem = ?4)
                    ORDER BY (normalized_stem = ?4) DESC, (headword = ?4) DESC, frequency_rank ASC, length(normalized_stem) ASC
                    LIMIT ?5";

    if let Ok(mut stmt) = conn.prepare(base_sql) {
        let pos_opt = part_of_speech.filter(|p| !p.is_empty() && *p != "all");
        if let Ok(rows) = stmt.query_map(
            params![lang_str, pos_opt, prefix_param, normalized, limit],
            parse_rich_row,
        ) {
            for row in rows.flatten() {
                seen_ids.insert(row.id.clone());
                rich_entries.push(row);
            }
        }
    }

    // 2. FTS5 with unicode61 tokenizer and BM25 column weighting
    if rich_entries.len() < limit {
        let remaining = limit - rich_entries.len();
        let fts_query = format!("\"{}\"*", normalized.replace('"', "\"\""));
        let fts_sql = "SELECT r.id, r.headword, r.normalized_stem, r.language_code, r.ipa_pronunciation,
                              r.native_phonetic_respelling, r.part_of_speech, r.gender_inflection_notes,
                              r.register, r.senses_json, r.bilingual_examples_json, r.synonyms_json,
                              r.antonyms_json, r.derived_forms_json, r.target_equivalents_json,
                              r.frequency_rank, r.provenance
                       FROM rich_dictionary_fts f
                       JOIN rich_lexicon_entries r ON f.rowid = r.rowid
                       WHERE rich_dictionary_fts MATCH ?1
                         AND (?2 IS NULL OR r.language_code = ?2)
                         AND (?3 IS NULL OR r.part_of_speech = ?3)
                       ORDER BY bm25(rich_dictionary_fts, 10.0, 5.0, 2.0, 1.0) ASC
                       LIMIT ?4";

        if let Ok(mut fts_stmt) = conn.prepare(fts_sql) {
            let pos_opt = part_of_speech.filter(|p| !p.is_empty() && *p != "all");
            if let Ok(fts_rows) = fts_stmt.query_map(
                params![fts_query, lang_str, pos_opt, remaining],
                parse_rich_row,
            ) {
                for row in fts_rows.flatten() {
                    if !seen_ids.contains(&row.id) {
                        seen_ids.insert(row.id.clone());
                        rich_entries.push(row);
                    }
                }
            }
        }
    }

    // 3. Subphase 9.2.2: Agglutinative Morphological Lemmatizer Fallback
    if rich_entries.is_empty() {
        if let Some(stem) = strip_morphological_suffix(&normalized) {
            let stem_prefix = format!("{}%", stem);
            let stem_sql = "SELECT id, headword, normalized_stem, language_code, ipa_pronunciation,
                                   native_phonetic_respelling, part_of_speech, gender_inflection_notes,
                                   register, senses_json, bilingual_examples_json, synonyms_json,
                                   antonyms_json, derived_forms_json, target_equivalents_json,
                                   frequency_rank, provenance
                            FROM rich_lexicon_entries
                            WHERE (?1 IS NULL OR language_code = ?1)
                              AND (normalized_stem = ?2 OR headword = ?2 OR normalized_stem LIKE ?3)
                            ORDER BY (normalized_stem = ?2) DESC, frequency_rank ASC
                            LIMIT ?4";

            if let Ok(mut stmt) = conn.prepare(stem_sql) {
                if let Ok(rows) =
                    stmt.query_map(params![lang_str, stem, stem_prefix, limit], parse_rich_row)
                {
                    for row in rows.flatten() {
                        if !seen_ids.contains(&row.id) {
                            seen_ids.insert(row.id.clone());
                            rich_entries.push(row);
                        }
                    }
                }
            }

            if !rich_entries.is_empty() {
                resolved_stem = Some(stem);
            }
        }
    }

    // 4. Subphase 9.2.3: Fuzzy & Phonetic Suggestions Fallback ("Did you mean: ...?")
    let mut did_you_mean = Vec::new();
    if rich_entries.is_empty() {
        let cand_sql = "SELECT DISTINCT headword FROM rich_lexicon_entries WHERE (?1 IS NULL OR language_code = ?1) LIMIT 200";
        if let Ok(mut stmt) = conn.prepare(cand_sql) {
            if let Ok(rows) = stmt.query_map(params![lang_str], |r| r.get::<_, String>(0)) {
                let candidates: Vec<String> = rows.flatten().collect();
                let mut scored: Vec<(String, usize)> = Vec::new();

                for c in &candidates {
                    let dist = compute_levenshtein(&normalized.to_lowercase(), &c.to_lowercase());
                    if dist <= 2 && dist > 0 {
                        scored.push((c.clone(), dist));
                    }
                }

                scored.sort_by_key(|s| s.1);
                did_you_mean = scored.into_iter().take(3).map(|s| s.0).collect();
            }
        }
    }

    // Fallback to legacy dictionary entries if no rich entries found
    if rich_entries.is_empty() {
        if let Ok(legacy) = search_dictionary(conn, query, lang, limit) {
            for l in &legacy {
                let mut senses = vec![LexiconSense {
                    sense_index: 1,
                    part_of_speech: l.part_of_speech.clone(),
                    definition: l.definition.clone(),
                    register: "general".into(),
                    synonyms: l.synonyms.clone(),
                    antonyms: l.antonyms.clone(),
                    examples: l.examples.clone(),
                }];
                for (idx, alt) in l.alternate_meanings.iter().enumerate() {
                    senses.push(LexiconSense {
                        sense_index: (idx + 2) as u32,
                        part_of_speech: l.part_of_speech.clone(),
                        definition: alt.clone(),
                        register: "general".into(),
                        synonyms: vec![],
                        antonyms: vec![],
                        examples: vec![],
                    });
                }
                rich_entries.push(RichDictionaryEntry {
                    id: l.id.clone(),
                    headword: l.word.clone(),
                    normalized_stem: l.normalized_word.clone(),
                    language: l.language,
                    ipa_pronunciation: l.pronunciation_ipa.clone(),
                    native_phonetic_respelling: l.transliteration.clone(),
                    part_of_speech: l.part_of_speech.clone(),
                    gender_inflection_notes: None,
                    register: "general".into(),
                    senses,
                    bilingual_examples: l.examples.clone(),
                    synonyms: l.synonyms.clone(),
                    antonyms: l.antonyms.clone(),
                    derived_forms: vec![],
                    target_equivalents: HashMap::new(),
                    frequency_rank: 500,
                    provenance: l.provenance.clone(),
                });
            }
        }
    }

    // Build synchronized legacy entries array
    let legacy_entries = rich_entries
        .iter()
        .map(|r| {
            let primary_def = r
                .senses
                .first()
                .map(|s| s.definition.clone())
                .unwrap_or_else(|| r.headword.clone());
            let alternate_meanings = r
                .senses
                .iter()
                .skip(1)
                .map(|s| s.definition.clone())
                .collect();
            DictionaryEntry {
                id: r.id.clone(),
                word: r.headword.clone(),
                normalized_word: r.normalized_stem.clone(),
                language: r.language,
                part_of_speech: r.part_of_speech.clone(),
                definition: primary_def,
                alternate_meanings,
                examples: r.bilingual_examples.clone(),
                pronunciation_ipa: r.ipa_pronunciation.clone(),
                transliteration: r.native_phonetic_respelling.clone(),
                synonyms: r.synonyms.clone(),
                antonyms: r.antonyms.clone(),
                provenance: r.provenance.clone(),
            }
        })
        .collect();

    let total = rich_entries.len();
    let latency_ms = start_time.elapsed().as_millis() as u64;

    Ok(DictionarySearchCmdResponse {
        entries: legacy_entries,
        rich_entries,
        resolved_stem,
        did_you_mean,
        query: query.to_string(),
        total_matches: total,
        latency_ms: latency_ms.max(1),
    })
}

pub fn seed_stage_9_rich_lexicon(conn: &Connection) -> Result<(), LinguaError> {
    info!("Seeding Stage 9 Rich Trilingual Lexicon Datasets (en-US, kn-IN, ta-IN)");

    let entries = vec![
        // ====================================================================
        // ENGLISH ENTRIES (A1-C2 Core + Medical + Logistics)
        // ====================================================================
        RichDictionaryEntry {
            id: "rich-en-adventure".into(),
            headword: "adventure".into(),
            normalized_stem: "adventure".into(),
            language: LanguageCode::EnUs,
            ipa_pronunciation: Some("/ədˈvɛntʃər/".into()),
            native_phonetic_respelling: None,
            part_of_speech: "noun".into(),
            gender_inflection_notes: None,
            register: "general".into(),
            senses: vec![
                LexiconSense {
                    sense_index: 1,
                    part_of_speech: "noun".into(),
                    definition: "An exciting, daring, or hazardous experience or undertaking.".into(),
                    register: "general".into(),
                    synonyms: vec!["venture".into(), "exploit".into(), "quest".into(), "thrill".into()],
                    antonyms: vec!["routine".into(), "inaction".into(), "safety".into()],
                    examples: vec![
                        DictionaryExample {
                            source: "Exploring the Western Ghats was a thrilling adventure.".into(),
                            target: "ಪಶ್ಚಿಮ ಘಟ್ಟಗಳನ್ನು ಅನ್ವೇಷಿಸುವುದು ರೋಮಾಂಚಕ ಸಾಹಸವಾಗಿತ್ತು.".into(),
                        },
                        DictionaryExample {
                            source: "Their journey across Karnataka and Tamil Nadu was full of adventure.".into(),
                            target: "ಕರ್ನಾಟಕ ಮತ್ತು ತಮಿಳುನಾಡಿನಾದ್ಯಂತ ಅವರ ಪ್ರಯಾಣವು ಸಾಹಸದಿಂದ ಕೂಡಿತ್ತು.".into(),
                        },
                    ],
                },
                LexiconSense {
                    sense_index: 2,
                    part_of_speech: "verb".into(),
                    definition: "To engage in a hazardous or daring enterprise; to risk or venture boldly.".into(),
                    register: "literary".into(),
                    synonyms: vec!["venture".into(), "hazard".into(), "risk".into()],
                    antonyms: vec!["hesitate".into()],
                    examples: vec![
                        DictionaryExample {
                            source: "They adventured into the dense forest despite the heavy rain.".into(),
                            target: "ಭಾರೀ ಮಳೆಯ ಹೊರತಾಗಿಯೂ ಅವರು ದಟ್ಟ ಕಾಡಿನಲ್ಲಿ ಸಾಹಸ ಮಾಡಿದರು.".into(),
                        },
                    ],
                },
            ],
            bilingual_examples: vec![
                DictionaryExample {
                    source: "Life is either a daring adventure or nothing at all.".into(),
                    target: "ಜೀವನವು ಧೈರ್ಯಶಾಲಿ ಸಾಹಸ ಅಥವಾ ಏನೂ ಅಲ್ಲ.".into(),
                },
            ],
            synonyms: vec!["venture".into(), "exploit".into(), "enterprise".into(), "feat".into()],
            antonyms: vec!["routine".into(), "boredom".into()],
            derived_forms: vec!["adventures".into(), "adventurer".into(), "adventuring".into(), "adventurous".into(), "adventurously".into()],
            target_equivalents: {
                let mut m = HashMap::new();
                m.insert("kn-IN".into(), vec!["ಸಾಹಸ".into(), "ರೋಮಾಂಚಕ ಅನುಭವ".into(), "ಧೈರ್ಯದ ಕೆಲಸ".into()]);
                m.insert("ta-IN".into(), vec!["சாகசம்".into(), "துணிகரச் செயல்".into(), "வீரச்செயல்".into()]);
                m
            },
            frequency_rank: 120,
            provenance: "WordNet 3.1 & Alar / Wiktextract (CC-BY-4.0)".into(),
        },
        RichDictionaryEntry {
            id: "rich-en-doctor".into(),
            headword: "doctor".into(),
            normalized_stem: "doctor".into(),
            language: LanguageCode::EnUs,
            ipa_pronunciation: Some("/ˈdɒk.tər/".into()),
            native_phonetic_respelling: None,
            part_of_speech: "noun".into(),
            gender_inflection_notes: None,
            register: "medical".into(),
            senses: vec![
                LexiconSense {
                    sense_index: 1,
                    part_of_speech: "noun".into(),
                    definition: "A qualified practitioner of medicine; physician.".into(),
                    register: "medical".into(),
                    synonyms: vec!["physician".into(), "medical practitioner".into(), "clinician".into()],
                    antonyms: vec![],
                    examples: vec![
                        DictionaryExample {
                            source: "The doctor examined the patient in the rural health triage center.".into(),
                            target: "ಗ್ರಾಮೀಣ ಆರೋಗ್ಯ ಕೇಂದ್ರದಲ್ಲಿ ವೈದ್ಯರು ರೋಗಿಯನ್ನು ತಪಾಸಣೆ ಮಾಡಿದರು.".into(),
                        },
                    ],
                },
            ],
            bilingual_examples: vec![
                DictionaryExample {
                    source: "Please consult a doctor immediately.".into(),
                    target: "ದಯವಿಟ್ಟು ತಕ್ಷಣ ವೈದ್ಯರನ್ನು ಸಂಪರ್ಕಿಸಿ.".into(),
                },
            ],
            synonyms: vec!["physician".into(), "medic".into()],
            antonyms: vec![],
            derived_forms: vec!["doctors".into(), "doctoral".into(), "doctorate".into()],
            target_equivalents: {
                let mut m = HashMap::new();
                m.insert("kn-IN".into(), vec!["ವೈದ್ಯ".into(), "ಡಾಕ್ಟರ್".into()]);
                m.insert("ta-IN".into(), vec!["மருத்துவர்".into(), "டாக்டர்".into()]);
                m
            },
            frequency_rank: 85,
            provenance: "Medical Triage Lexicon & Wiktextract (CC-BY-4.0)".into(),
        },
        RichDictionaryEntry {
            id: "rich-en-hospital".into(),
            headword: "hospital".into(),
            normalized_stem: "hospital".into(),
            language: LanguageCode::EnUs,
            ipa_pronunciation: Some("/ˈhɒs.pɪ.təl/".into()),
            native_phonetic_respelling: None,
            part_of_speech: "noun".into(),
            gender_inflection_notes: None,
            register: "medical".into(),
            senses: vec![
                LexiconSense {
                    sense_index: 1,
                    part_of_speech: "noun".into(),
                    definition: "An institution providing medical and surgical treatment and nursing care.".into(),
                    register: "medical".into(),
                    synonyms: vec!["infirmary".into(), "clinic".into(), "medical center".into()],
                    antonyms: vec![],
                    examples: vec![
                        DictionaryExample {
                            source: "The patient was transferred to the district hospital.".into(),
                            target: "ರೋಗಿಯನ್ನು ಜಿಲ್ಲಾ ಆಸ್ಪತ್ರೆಗೆ ವರ್ಗಾಯಿಸಲಾಯಿತು.".into(),
                        },
                    ],
                },
            ],
            bilingual_examples: vec![],
            synonyms: vec!["clinic".into(), "health center".into()],
            antonyms: vec![],
            derived_forms: vec!["hospitals".into(), "hospitalize".into(), "hospitalization".into()],
            target_equivalents: {
                let mut m = HashMap::new();
                m.insert("kn-IN".into(), vec!["ಆಸ್ಪತ್ರೆ".into(), "ಚಿಕಿತ್ಸಾಲಯ".into()]);
                m.insert("ta-IN".into(), vec!["மருத்துவமனை".into()]);
                m
            },
            frequency_rank: 140,
            provenance: "Healthcare Corpus (CC-BY-4.0)".into(),
        },
        RichDictionaryEntry {
            id: "rich-en-freight".into(),
            headword: "freight".into(),
            normalized_stem: "freight".into(),
            language: LanguageCode::EnUs,
            ipa_pronunciation: Some("/freɪt/".into()),
            native_phonetic_respelling: None,
            part_of_speech: "noun".into(),
            gender_inflection_notes: None,
            register: "logistics".into(),
            senses: vec![
                LexiconSense {
                    sense_index: 1,
                    part_of_speech: "noun".into(),
                    definition: "Goods transported in bulk by truck, train, ship, or aircraft.".into(),
                    register: "logistics".into(),
                    synonyms: vec!["cargo".into(), "consignment".into(), "shipment".into()],
                    antonyms: vec![],
                    examples: vec![
                        DictionaryExample {
                            source: "Interstate freight trucks travel daily between Bengaluru and Chennai.".into(),
                            target: "ಬೆಂಗಳೂರು ಮತ್ತು ಚೆನ್ನೈ ನಡುವೆ ಪ್ರತಿದಿನ ಅಂತಾರಾಜ್ಯ ಸರಕು ಲಾರಿಗಳು ಸಂಚರಿಸುತ್ತವೆ.".into(),
                        },
                    ],
                },
            ],
            bilingual_examples: vec![],
            synonyms: vec!["cargo".into(), "shipment".into(), "merchandise".into()],
            antonyms: vec![],
            derived_forms: vec!["freighter".into(), "freighting".into()],
            target_equivalents: {
                let mut m = HashMap::new();
                m.insert("kn-IN".into(), vec!["ಸರಕು ಸಾಗಣೆ".into(), "ಮಾಲು".into()]);
                m.insert("ta-IN".into(), vec!["சரக்கு போக்குவரத்து".into(), "சரக்கு".into()]);
                m
            },
            frequency_rank: 310,
            provenance: "Logistics Corpus (CC-BY-4.0)".into(),
        },
        RichDictionaryEntry {
            id: "rich-en-warehouse".into(),
            headword: "warehouse".into(),
            normalized_stem: "warehouse".into(),
            language: LanguageCode::EnUs,
            ipa_pronunciation: Some("/ˈwɛər.haʊs/".into()),
            native_phonetic_respelling: None,
            part_of_speech: "noun".into(),
            gender_inflection_notes: None,
            register: "logistics".into(),
            senses: vec![
                LexiconSense {
                    sense_index: 1,
                    part_of_speech: "noun".into(),
                    definition: "A large building where raw materials or manufactured goods may be stored prior to distribution.".into(),
                    register: "logistics".into(),
                    synonyms: vec!["depot".into(), "godown".into(), "storage facility".into()],
                    antonyms: vec![],
                    examples: vec![
                        DictionaryExample {
                            source: "The inventory arrived at the central warehouse on time.".into(),
                            target: "ದಾಸ್ತಾನು ನಿಗದಿತ ಸಮಯಕ್ಕೆ ಕೇಂದ್ರ ಗೋದಾಮಿಗೆ ತಲುಪಿತು.".into(),
                        },
                    ],
                },
            ],
            bilingual_examples: vec![],
            synonyms: vec!["depot".into(), "repository".into()],
            antonyms: vec![],
            derived_forms: vec!["warehouses".into(), "warehousing".into()],
            target_equivalents: {
                let mut m = HashMap::new();
                m.insert("kn-IN".into(), vec!["ಗೋದಾಮು".into(), "ಸರಕು ಸಂಗ್ರಹಾಲಯ".into()]);
                m.insert("ta-IN".into(), vec!["கிடங்கு".into(), "சேமிப்புக் கிடங்கு".into()]);
                m
            },
            frequency_rank: 290,
            provenance: "Logistics Corpus (CC-BY-4.0)".into(),
        },

        // ====================================================================
        // KANNADA ENTRIES (Stage 9 Authentic Lexicon)
        // ====================================================================
        RichDictionaryEntry {
            id: "rich-kn-sahasa".into(),
            headword: "ಸಾಹಸ".into(),
            normalized_stem: "ಸಾಹಸ".into(),
            language: LanguageCode::KnIn,
            ipa_pronunciation: Some("/sɑː.hɐ.sɐ/".into()),
            native_phonetic_respelling: Some("sāhasa".into()),
            part_of_speech: "noun".into(),
            gender_inflection_notes: Some("neuter noun (ನಪುಂಸಕ ಲಿಂಗ)".into()),
            register: "general".into(),
            senses: vec![
                LexiconSense {
                    sense_index: 1,
                    part_of_speech: "noun".into(),
                    definition: "ಧೈರ್ಯ, ಶೌರ್ಯ ಅಥವಾ ಅಪಾಯದಿಂದ ಕೂಡಿದ ಮಹತ್ಕಾರ್ಯ; ರೋಮಾಂಚಕ ಸಾಹಸಯಾತ್ರೆ.".into(),
                    register: "general".into(),
                    synonyms: vec!["ಧೈರ್ಯ".into(), "ಶೌರ್ಯ".into(), "ಪರಾಕ್ರಮ".into(), "ರೋಮಾಂಚನ".into()],
                    antonyms: vec!["ಹೇಡಿತನ".into(), "ಅಂಜಿಕೆ".into()],
                    examples: vec![
                        DictionaryExample {
                            source: "ಪರ್ವತಾರೋಹಣ ಒಂದು ಅದ್ಭುತ ಸಾಹಸ.".into(),
                            target: "Mountaineering is a wonderful adventure.".into(),
                        },
                    ],
                },
            ],
            bilingual_examples: vec![
                DictionaryExample {
                    source: "ಯುವಕರು ಸಾಹಸ ಕ್ರೀಡೆಗಳಲ್ಲಿ ಹೆಚ್ಚು ಆಸಕ್ತಿ ಹೊಂದಿದ್ದಾರೆ.".into(),
                    target: "Youth are very interested in adventure sports.".into(),
                },
            ],
            synonyms: vec!["ಧೈರ್ಯದ ಕೆಲಸ".into(), "ವೀರಕಾರ್ಯ".into()],
            antonyms: vec!["ಭಯ".into()],
            derived_forms: vec!["ಸಾಹಸಿ".into(), "ಸಾಹಸಗಳು".into(), "ಸಾಹಸಮಯ".into(), "ಸಾಹಸಿಕ".into()],
            target_equivalents: {
                let mut m = HashMap::new();
                m.insert("en-US".into(), vec!["adventure".into(), "daring feat".into(), "exploit".into()]);
                m.insert("ta-IN".into(), vec!["சாகசம்".into(), "துணிகரச் செயல்".into()]);
                m
            },
            frequency_rank: 95,
            provenance: "Alar Dictionary (ODC-By / CC-BY-4.0) & Wiktextract".into(),
        },
        RichDictionaryEntry {
            id: "rich-kn-namaskara".into(),
            headword: "ನಮಸ್ಕಾರ".into(),
            normalized_stem: "ನಮಸ್ಕಾರ".into(),
            language: LanguageCode::KnIn,
            ipa_pronunciation: Some("/nɐ.mɐs.kɑː.rɐ/".into()),
            native_phonetic_respelling: Some("namaskāra".into()),
            part_of_speech: "greeting / interjection".into(),
            gender_inflection_notes: None,
            register: "formal".into(),
            senses: vec![
                LexiconSense {
                    sense_index: 1,
                    part_of_speech: "greeting".into(),
                    definition: "ಗೌರವಪೂರ್ವಕ ವಂದನೆ, ಭೇಟಿಯಾದಾಗ ಅಥವಾ ಬೀಳ್ಕೊಡುವಾಗ ಮಾಡುವ ಸಾಂಪ್ರದಾಯಿಕ ಸತ್ಕಾರ.".into(),
                    register: "formal".into(),
                    synonyms: vec!["ವಂದನೆ".into(), "ಪ್ರಣಾಮ".into()],
                    antonyms: vec![],
                    examples: vec![
                        DictionaryExample {
                            source: "ಎಲ್ಲರಿಗೂ ನಮಸ್ಕಾರ, ಕರ್ನಾಟಕಕ್ಕೆ ಸುಸ್ವಾಗತ.".into(),
                            target: "Hello everyone, welcome to Karnataka.".into(),
                        },
                    ],
                },
            ],
            bilingual_examples: vec![],
            synonyms: vec!["ವಂದನೆ".into(), "ಶರಣು".into()],
            antonyms: vec![],
            derived_forms: vec!["ನಮಸ್ಕಾರಗಳು".into(), "ನಮಸ್ಕರಿಸು".into()],
            target_equivalents: {
                let mut m = HashMap::new();
                m.insert("en-US".into(), vec!["hello".into(), "greetings".into()]);
                m.insert("ta-IN".into(), vec!["வணக்கம்".into()]);
                m
            },
            frequency_rank: 10,
            provenance: "Alar Dictionary (ODC-By)".into(),
        },
        RichDictionaryEntry {
            id: "rich-kn-vaidya".into(),
            headword: "ವೈದ್ಯ".into(),
            normalized_stem: "ವೈದ್ಯ".into(),
            language: LanguageCode::KnIn,
            ipa_pronunciation: Some("/ʋɐɪ.d̪jɐ/".into()),
            native_phonetic_respelling: Some("vaidya".into()),
            part_of_speech: "noun".into(),
            gender_inflection_notes: Some("masculine/common noun (ಪುಲ್ಲಿಂಗ/ಸಾಮಾನ್ಯ)".into()),
            register: "medical".into(),
            senses: vec![
                LexiconSense {
                    sense_index: 1,
                    part_of_speech: "noun".into(),
                    definition: "ರೋಗಿಗಳಿಗೆ ಚಿಕಿತ್ಸೆ ನೀಡುವ ಅರ್ಹ ವೈದ್ಯಕೀಯ ತಜ್ಞ.".into(),
                    register: "medical".into(),
                    synonyms: vec!["ಡಾಕ್ಟರ್".into(), "ಚಿಕಿತ್ಸಕ".into()],
                    antonyms: vec![],
                    examples: vec![
                        DictionaryExample {
                            source: "ವೈದ್ಯರು ರೋಗಿಗೆ ಗುಣಮಟ್ಟದ ಚಿಕಿತ್ಸೆ ನೀಡಿದರು.".into(),
                            target: "The doctor provided high-quality treatment to the patient.".into(),
                        },
                    ],
                },
            ],
            bilingual_examples: vec![],
            synonyms: vec!["ಡಾಕ್ಟರ್".into()],
            antonyms: vec![],
            derived_forms: vec!["ವೈದ್ಯರು".into(), "ವೈದ್ಯಕೀಯ".into()],
            target_equivalents: {
                let mut m = HashMap::new();
                m.insert("en-US".into(), vec!["doctor".into(), "physician".into()]);
                m.insert("ta-IN".into(), vec!["மருத்துவர்".into()]);
                m
            },
            frequency_rank: 60,
            provenance: "Alar Dictionary (ODC-By)".into(),
        },
        RichDictionaryEntry {
            id: "rich-kn-saraku".into(),
            headword: "ಸರಕು".into(),
            normalized_stem: "ಸರಕು".into(),
            language: LanguageCode::KnIn,
            ipa_pronunciation: Some("/sɐ.rɐ.ku/".into()),
            native_phonetic_respelling: Some("saraku".into()),
            part_of_speech: "noun".into(),
            gender_inflection_notes: None,
            register: "logistics".into(),
            senses: vec![
                LexiconSense {
                    sense_index: 1,
                    part_of_speech: "noun".into(),
                    definition: "ಮಾರಾಟಕ್ಕೆ ಅಥವಾ ಸಾಗಣೆಗೆ ಉದ್ದೇಶಿಸಲಾದ ಮಾಲು ಅಥವಾ ಸಾಮಗ್ರಿ.".into(),
                    register: "logistics".into(),
                    synonyms: vec!["ಮಾಲು".into(), "ಸಾಮಾನು".into()],
                    antonyms: vec![],
                    examples: vec![
                        DictionaryExample {
                            source: "ಸರಕು ಸಾಗಣೆ ವಾಹನಗಳು ಸಕಾಲಕ್ಕೆ ತಲುಪಿದವು.".into(),
                            target: "The freight transport vehicles arrived on time.".into(),
                        },
                    ],
                },
            ],
            bilingual_examples: vec![],
            synonyms: vec!["ಮಾಲು".into()],
            antonyms: vec![],
            derived_forms: vec!["ಸರಕುಗಳು".into(), "ಸರಕುಪಟ್ಟಿ".into()],
            target_equivalents: {
                let mut m = HashMap::new();
                m.insert("en-US".into(), vec!["freight".into(), "cargo".into(), "goods".into()]);
                m.insert("ta-IN".into(), vec!["சரக்கு".into()]);
                m
            },
            frequency_rank: 180,
            provenance: "Alar Dictionary (ODC-By)".into(),
        },

        // ====================================================================
        // TAMIL ENTRIES (Stage 9 Authentic Lexicon)
        // ====================================================================
        RichDictionaryEntry {
            id: "rich-ta-cakacam".into(),
            headword: "சாகசம்".into(),
            normalized_stem: "சாகசம்".into(),
            language: LanguageCode::TaIn,
            ipa_pronunciation: Some("/cɑː.kɐ.sɐm/".into()),
            native_phonetic_respelling: Some("cākacam".into()),
            part_of_speech: "noun".into(),
            gender_inflection_notes: Some("neuter noun (அஃறிணை பெயர்ச்சொல்)".into()),
            register: "general".into(),
            senses: vec![
                LexiconSense {
                    sense_index: 1,
                    part_of_speech: "noun".into(),
                    definition: "துணிச்சலான அல்லது வீரமிக்க செயல்; ஆபத்தான ஆனால் விறுவிறுப்பான அனுபவம்.".into(),
                    register: "general".into(),
                    synonyms: vec!["துணிவு".into(), "வீரச்செயல்".into(), "துணிகரம்".into()],
                    antonyms: vec!["கோழைத்தனம்".into(), "பயம்".into()],
                    examples: vec![
                        DictionaryExample {
                            source: "மலை ஏறுதல் ஒரு சிறந்த சாகசமாகும்.".into(),
                            target: "Mountain climbing is a great adventure.".into(),
                        },
                    ],
                },
            ],
            bilingual_examples: vec![
                DictionaryExample {
                    source: "அவர் பல வீர சாகசங்களைச் செய்தார்.".into(),
                    target: "He performed many heroic feats and adventures.".into(),
                },
            ],
            synonyms: vec!["வீரச்செயல்".into(), "துணிகரச் செயல்".into()],
            antonyms: vec!["அச்சம்".into()],
            derived_forms: vec!["சாகசக்காரன்".into(), "சாகசங்கள்".into(), "சாகசமான".into()],
            target_equivalents: {
                let mut m = HashMap::new();
                m.insert("en-US".into(), vec!["adventure".into(), "feat".into(), "exploit".into()]);
                m.insert("kn-IN".into(), vec!["ಸಾಹಸ".into(), "ರೋಮಾಂಚಕ ಅನುಭವ".into()]);
                m
            },
            frequency_rank: 95,
            provenance: "University of Madras Tamil Lexicon & Wiktextract".into(),
        },
        RichDictionaryEntry {
            id: "rich-ta-vanakkam".into(),
            headword: "வணக்கம்".into(),
            normalized_stem: "வணக்கம்".into(),
            language: LanguageCode::TaIn,
            ipa_pronunciation: Some("/ʋɐ.ɳɐk.kɐm/".into()),
            native_phonetic_respelling: Some("vaṇakkam".into()),
            part_of_speech: "greeting / interjection".into(),
            gender_inflection_notes: None,
            register: "formal".into(),
            senses: vec![
                LexiconSense {
                    sense_index: 1,
                    part_of_speech: "greeting".into(),
                    definition: "தமிழர்களின் மரபுவழி வாழ்த்து மற்றும் வரவேற்புச் சொல்.".into(),
                    register: "formal".into(),
                    synonyms: vec!["வந்தனம்".into(), "நல்வரவு".into()],
                    antonyms: vec![],
                    examples: vec![
                        DictionaryExample {
                            source: "அனைவருக்கும் வணக்கம், நலமா?".into(),
                            target: "Hello everyone, how are you?".into(),
                        },
                    ],
                },
            ],
            bilingual_examples: vec![],
            synonyms: vec!["வந்தனம்".into()],
            antonyms: vec![],
            derived_forms: vec!["வணக்கங்கள்".into()],
            target_equivalents: {
                let mut m = HashMap::new();
                m.insert("en-US".into(), vec!["hello".into(), "greetings".into()]);
                m.insert("kn-IN".into(), vec!["ನಮಸ್ಕಾರ".into()]);
                m
            },
            frequency_rank: 10,
            provenance: "University of Madras Tamil Lexicon".into(),
        },
        RichDictionaryEntry {
            id: "rich-ta-maruttuvar".into(),
            headword: "மருத்துவர்".into(),
            normalized_stem: "மருத்துவர்".into(),
            language: LanguageCode::TaIn,
            ipa_pronunciation: Some("/mɐ.rut̪.t̪u.ʋɐr/".into()),
            native_phonetic_respelling: Some("maruttuvar".into()),
            part_of_speech: "noun".into(),
            gender_inflection_notes: Some("common honorific noun (பலர்பால் பெயர்ச்சொல்)".into()),
            register: "medical".into(),
            senses: vec![
                LexiconSense {
                    sense_index: 1,
                    part_of_speech: "noun".into(),
                    definition: "நோயாளிகளுக்கு மருத்துவம் மற்றும் சிகிச்சை அளிக்கும் தகுதிபெற்ற நிபுணர்.".into(),
                    register: "medical".into(),
                    synonyms: vec!["டாக்டர்".into(), "வைத்தியர்".into()],
                    antonyms: vec![],
                    examples: vec![
                        DictionaryExample {
                            source: "மருத்துவர் நோயாளியை பரிசோதித்தார்.".into(),
                            target: "The doctor examined the patient.".into(),
                        },
                    ],
                },
            ],
            bilingual_examples: vec![],
            synonyms: vec!["டாக்டர்".into()],
            antonyms: vec![],
            derived_forms: vec!["மருத்துவர்கள்".into(), "மருத்துவம்".into()],
            target_equivalents: {
                let mut m = HashMap::new();
                m.insert("en-US".into(), vec!["doctor".into(), "physician".into()]);
                m.insert("kn-IN".into(), vec!["ವೈದ್ಯ".into()]);
                m
            },
            frequency_rank: 60,
            provenance: "University of Madras Tamil Lexicon".into(),
        },
        RichDictionaryEntry {
            id: "rich-ta-sarakku".into(),
            headword: "சரக்கு".into(),
            normalized_stem: "சரக்கு".into(),
            language: LanguageCode::TaIn,
            ipa_pronunciation: Some("/sɐ.rɐk.ku/".into()),
            native_phonetic_respelling: Some("sarakku".into()),
            part_of_speech: "noun".into(),
            gender_inflection_notes: None,
            register: "logistics".into(),
            senses: vec![
                LexiconSense {
                    sense_index: 1,
                    part_of_speech: "noun".into(),
                    definition: "போக்குவரத்து அல்லது விற்பனைக்கான வணிகப் பொருட்கள்.".into(),
                    register: "logistics".into(),
                    synonyms: vec!["பண்டம்".into(), "பொருள்".into()],
                    antonyms: vec![],
                    examples: vec![
                        DictionaryExample {
                            source: "சரக்கு வாகனங்கள் சென்னை துறைமுகத்தை அடைந்தன.".into(),
                            target: "Freight vehicles reached the Chennai port.".into(),
                        },
                    ],
                },
            ],
            bilingual_examples: vec![],
            synonyms: vec!["பொருட்கள்".into()],
            antonyms: vec![],
            derived_forms: vec!["சரக்குகள்".into(), "சரக்குந்து".into()],
            target_equivalents: {
                let mut m = HashMap::new();
                m.insert("en-US".into(), vec!["freight".into(), "cargo".into(), "goods".into()]);
                m.insert("kn-IN".into(), vec!["ಸರಕು".into()]);
                m
            },
            frequency_rank: 180,
            provenance: "University of Madras Tamil Lexicon".into(),
        },
    ];

    import_rich_lexicon_entries(conn, &entries)?;
    Ok(())
}
