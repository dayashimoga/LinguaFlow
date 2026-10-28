use lingua_core::{LanguageCode, LinguaError, ModelPackInfo};
use rusqlite::{params, Connection};
use std::str::FromStr;

pub fn seed_initial_model_catalog(conn: &Connection) -> Result<(), LinguaError> {
    let initial_packs = vec![
        ModelPackInfo {
            id: "model-kn-en-indictrans2".into(),
            name: "Kannada <-> English Neural Translation (IndicTrans2)".into(),
            source: LanguageCode::KnIn,
            target: LanguageCode::EnUs,
            size_bytes: 185_000_000,
            version: "1.0.0".into(),
            sha256: "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9".into(),
            is_installed: false,
            scheduled_stage: 2,
        },
        ModelPackInfo {
            id: "model-ta-en-indictrans2".into(),
            name: "Tamil <-> English Neural Translation (IndicTrans2)".into(),
            source: LanguageCode::TaIn,
            target: LanguageCode::EnUs,
            size_bytes: 185_000_000,
            version: "1.0.0".into(),
            sha256: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".into(),
            is_installed: false,
            scheduled_stage: 2,
        },
        ModelPackInfo {
            id: "model-kn-ta-direct".into(),
            name: "Kannada <-> Tamil Direct Neural Translation".into(),
            source: LanguageCode::KnIn,
            target: LanguageCode::TaIn,
            size_bytes: 195_000_000,
            version: "1.0.0".into(),
            sha256: "5994471abb01112afcc18159f6cc74b4f511b99806da59b3caf5a9c173cacfc5".into(),
            is_installed: false,
            scheduled_stage: 2,
        },
        ModelPackInfo {
            id: "asr-kn-streaming-sherpa".into(),
            name: "Kannada Streaming ASR (Sherpa ONNX)".into(),
            source: LanguageCode::KnIn,
            target: LanguageCode::KnIn,
            size_bytes: 84_000_000,
            version: "0.9.2".into(),
            sha256: "11a508955ea1050a300c638e2b6ab3e634b07be52f505bc55a38812ee69f04de".into(),
            is_installed: false,
            scheduled_stage: 3,
        },
        ModelPackInfo {
            id: "asr-ta-streaming-sherpa".into(),
            name: "Tamil Streaming ASR (Sherpa ONNX)".into(),
            source: LanguageCode::TaIn,
            target: LanguageCode::TaIn,
            size_bytes: 84_000_000,
            version: "0.9.2".into(),
            sha256: "2d711641b726b0295014ac619f8097b6bad924f88604b768f3047d43d335f116".into(),
            is_installed: false,
            scheduled_stage: 3,
        },
    ];

    for pack in initial_packs {
        conn.execute(
            "INSERT OR IGNORE INTO model_packs (id, name, source_language, target_language,
                                              size_bytes, version, sha256, is_installed, scheduled_stage)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                pack.id,
                pack.name,
                pack.source.as_bcp47(),
                pack.target.as_bcp47(),
                pack.size_bytes as i64,
                pack.version,
                pack.sha256,
                pack.is_installed as i64,
                pack.scheduled_stage as i64,
            ],
        )
        .map_err(|e| LinguaError::Database(format!("Failed to seed model pack: {e}")))?;
    }

    Ok(())
}

pub fn list_model_packs(conn: &Connection) -> Result<Vec<ModelPackInfo>, LinguaError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, source_language, target_language, size_bytes,
                    version, sha256, is_installed, scheduled_stage
             FROM model_packs ORDER BY scheduled_stage ASC, name ASC",
        )
        .map_err(|e| LinguaError::Database(e.to_string()))?;

    let rows = stmt
        .query_map([], |row| {
            let id: String = row.get(0)?;
            let name: String = row.get(1)?;
            let src_str: String = row.get(2)?;
            let tgt_str: String = row.get(3)?;
            let size_bytes: i64 = row.get(4)?;
            let version: String = row.get(5)?;
            let sha256: String = row.get(6)?;
            let is_installed: i64 = row.get(7)?;
            let scheduled_stage: i64 = row.get(8)?;

            let source = LanguageCode::from_str(&src_str).unwrap_or(LanguageCode::EnUs);
            let target = LanguageCode::from_str(&tgt_str).unwrap_or(LanguageCode::KnIn);

            Ok(ModelPackInfo {
                id,
                name,
                source,
                target,
                size_bytes: size_bytes as u64,
                version,
                sha256,
                is_installed: is_installed != 0,
                scheduled_stage: scheduled_stage as u8,
            })
        })
        .map_err(|e| LinguaError::Database(e.to_string()))?;

    let mut packs = Vec::new();
    for row in rows {
        packs.push(row.map_err(|e| LinguaError::Database(e.to_string()))?);
    }

    Ok(packs)
}
