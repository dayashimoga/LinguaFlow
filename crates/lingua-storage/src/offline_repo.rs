use chrono::{DateTime, Utc};
use lingua_core::{
    normalize_text_nfc, CachedTranslation, LanguageCode, LinguaError, MorphologyEntry,
};
use rusqlite::{params, Connection, OptionalExtension};
use std::str::FromStr;

pub fn record_translation_cache(
    conn: &Connection,
    cached: &CachedTranslation,
) -> Result<(), LinguaError> {
    let source_hash = compute_text_hash(&cached.source_text);
    conn.execute(
        "INSERT INTO cached_translations (id, source_language, target_language, source_text_hash,
                                        source_text, translated_text, engine_id, is_offline,
                                        created_at, hit_count)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 1)
         ON CONFLICT(id) DO UPDATE SET
            hit_count = hit_count + 1,
            created_at = excluded.created_at",
        params![
            cached.id,
            cached.source_language.as_bcp47(),
            cached.target_language.as_bcp47(),
            source_hash,
            cached.source_text,
            cached.translated_text,
            cached.engine_id,
            if cached.is_offline { 1 } else { 0 },
            cached.created_at.to_rfc3339(),
        ],
    )
    .map_err(|e| LinguaError::Database(format!("Failed to record translation cache: {e}")))?;

    Ok(())
}

pub fn lookup_translation_cache(
    conn: &Connection,
    source_lang: LanguageCode,
    target_lang: LanguageCode,
    source_text: &str,
) -> Result<Option<CachedTranslation>, LinguaError> {
    let source_hash = compute_text_hash(source_text);
    let mut stmt = conn
        .prepare(
            "SELECT id, source_language, target_language, source_text, translated_text,
                    engine_id, is_offline, created_at, hit_count
             FROM cached_translations
             WHERE source_language = ?1 AND target_language = ?2 AND source_text_hash = ?3
             ORDER BY created_at DESC LIMIT 1",
        )
        .map_err(|e| LinguaError::Database(e.to_string()))?;

    let result = stmt
        .query_row(
            params![source_lang.as_bcp47(), target_lang.as_bcp47(), source_hash],
            |row| {
                let id: String = row.get(0)?;
                let src_str: String = row.get(1)?;
                let tgt_str: String = row.get(2)?;
                let source_text: String = row.get(3)?;
                let translated_text: String = row.get(4)?;
                let engine_id: String = row.get(5)?;
                let is_offline_int: i64 = row.get(6)?;
                let created_at_str: String = row.get(7)?;
                let hit_count: u32 = row.get(8)?;

                let source_language =
                    LanguageCode::from_str(&src_str).unwrap_or(LanguageCode::EnUs);
                let target_language =
                    LanguageCode::from_str(&tgt_str).unwrap_or(LanguageCode::KnIn);
                let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());

                Ok(CachedTranslation {
                    id,
                    source_language,
                    target_language,
                    source_text,
                    translated_text,
                    engine_id,
                    is_offline: is_offline_int != 0,
                    created_at,
                    hit_count,
                })
            },
        )
        .optional()
        .map_err(|e| LinguaError::Database(e.to_string()))?;

    Ok(result)
}

pub fn compute_text_hash(text: &str) -> String {
    use sha2::{Digest, Sha256};
    let normalized = normalize_text_nfc(text.trim());
    let mut hasher = Sha256::new();
    hasher.update(normalized.as_bytes());
    hex::encode(hasher.finalize())
}

pub fn seed_initial_morphology(conn: &Connection) -> Result<(), LinguaError> {
    let stems = vec![
        // Kannada Stems and Inflections
        (
            "morph-kn-01",
            LanguageCode::KnIn,
            "ಮಾಡುತ್ತಾನೆ",
            "ಮಾಡು",
            "Verb",
            "Present continuous 3rd person masculine: he does / makes",
        ),
        (
            "morph-kn-02",
            LanguageCode::KnIn,
            "ಮಾಡುತ್ತಾಳೆ",
            "ಮಾಡು",
            "Verb",
            "Present continuous 3rd person feminine: she does / makes",
        ),
        (
            "morph-kn-03",
            LanguageCode::KnIn,
            "ಮಾಡುತ್ತಾರೆ",
            "ಮಾಡು",
            "Verb",
            "Present continuous 3rd person plural / honorific: they do",
        ),
        (
            "morph-kn-04",
            LanguageCode::KnIn,
            "ಮಾಡಿದೆ",
            "ಮಾಡು",
            "Verb",
            "Past tense 3rd person neuter: it did / made",
        ),
        (
            "morph-kn-05",
            LanguageCode::KnIn,
            "ಹೋಗುತ್ತಾನೆ",
            "ಹೋಗು",
            "Verb",
            "Present continuous: he goes",
        ),
        (
            "morph-kn-06",
            LanguageCode::KnIn,
            "ಹೋದರು",
            "ಹೋಗು",
            "Verb",
            "Past tense plural: they went",
        ),
        (
            "morph-kn-07",
            LanguageCode::KnIn,
            "ಬರುತ್ತಾನೆ",
            "ಬರು",
            "Verb",
            "Present continuous: he comes",
        ),
        (
            "morph-kn-08",
            LanguageCode::KnIn,
            "ಬಂದರು",
            "ಬರು",
            "Verb",
            "Past tense plural: they came",
        ),
        (
            "morph-kn-09",
            LanguageCode::KnIn,
            "ಮನೆಗಳು",
            "ಮನೆ",
            "Noun",
            "Plural form: houses",
        ),
        (
            "morph-kn-10",
            LanguageCode::KnIn,
            "ನೀರಿಗೆ",
            "ನೀರು",
            "Noun",
            "Dative case: to / for water",
        ),
        // Tamil Stems and Inflections
        (
            "morph-ta-01",
            LanguageCode::TaIn,
            "செய்கிறான்",
            "செய்",
            "Verb",
            "Present tense 3rd person masculine: he does",
        ),
        (
            "morph-ta-02",
            LanguageCode::TaIn,
            "செய்கிறாள்",
            "செய்",
            "Verb",
            "Present tense 3rd person feminine: she does",
        ),
        (
            "morph-ta-03",
            LanguageCode::TaIn,
            "செய்தார்",
            "செய்",
            "Verb",
            "Past tense honorific: he/she did",
        ),
        (
            "morph-ta-04",
            LanguageCode::TaIn,
            "போகிறான்",
            "போ",
            "Verb",
            "Present tense masculine: he goes",
        ),
        (
            "morph-ta-05",
            LanguageCode::TaIn,
            "போனார்",
            "போ",
            "Verb",
            "Past tense honorific: he went",
        ),
        (
            "morph-ta-06",
            LanguageCode::TaIn,
            "வருகிறான்",
            "வா",
            "Verb",
            "Present tense masculine: he comes",
        ),
        (
            "morph-ta-07",
            LanguageCode::TaIn,
            "வந்தார்",
            "வா",
            "Verb",
            "Past tense honorific: he came",
        ),
        (
            "morph-ta-08",
            LanguageCode::TaIn,
            "வீடுகள்",
            "வீடு",
            "Noun",
            "Plural form: houses",
        ),
        (
            "morph-ta-09",
            LanguageCode::TaIn,
            "தண்ணீருக்கு",
            "தண்ணீர்",
            "Noun",
            "Dative case: to / for water",
        ),
    ];

    for (id, lang, surface, stem, pos, notes) in stems {
        let norm_surface = normalize_text_nfc(surface);
        conn.execute(
            "INSERT OR IGNORE INTO morphological_stems (id, language, surface_form, normalized_surface, stem_form, part_of_speech, grammar_notes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                id,
                lang.as_bcp47(),
                surface,
                norm_surface,
                stem,
                pos,
                notes,
            ],
        )
        .map_err(|e| LinguaError::Database(format!("Failed to seed morphology: {e}")))?;
    }

    Ok(())
}

pub fn lookup_morphology(
    conn: &Connection,
    lang: LanguageCode,
    surface: &str,
) -> Result<Option<MorphologyEntry>, LinguaError> {
    let norm = normalize_text_nfc(surface.trim());
    let mut stmt = conn
        .prepare(
            "SELECT id, language, surface_form, stem_form, part_of_speech, grammar_notes
             FROM morphological_stems
             WHERE language = ?1 AND normalized_surface = ?2
             LIMIT 1",
        )
        .map_err(|e| LinguaError::Database(e.to_string()))?;

    let res = stmt
        .query_row(params![lang.as_bcp47(), norm], |row| {
            let id: String = row.get(0)?;
            let lang_str: String = row.get(1)?;
            let surface_form: String = row.get(2)?;
            let stem_form: String = row.get(3)?;
            let part_of_speech: Option<String> = row.get(4)?;
            let grammar_notes: Option<String> = row.get(5)?;

            let language = LanguageCode::from_str(&lang_str).unwrap_or(lang);

            Ok(MorphologyEntry {
                id,
                language,
                surface_form,
                stem_form,
                part_of_speech,
                grammar_notes,
            })
        })
        .optional()
        .map_err(|e| LinguaError::Database(e.to_string()))?;

    Ok(res)
}
