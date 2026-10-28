use lingua_core::{
    normalize_text_nfc, DictionaryEntry, DictionaryExample, LanguageCode, LinguaError,
};
use rusqlite::{params, Connection};
use std::str::FromStr;
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
