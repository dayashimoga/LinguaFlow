use chrono::{DateTime, Utc};
use lingua_core::{
    LanguageCode, LinguaError, ModelInstallState, ModelPackDetailedInfo, ModelPackInfo,
    ModelPackManifest,
};
use rusqlite::{params, Connection, OptionalExtension};
use std::str::FromStr;

pub fn seed_initial_model_catalog(conn: &Connection) -> Result<(), LinguaError> {
    let initial_packs = vec![
        (
            "model-kn-en-indictrans2",
            "Kannada <-> English Neural Translation (IndicTrans2)",
            LanguageCode::KnIn,
            LanguageCode::EnUs,
            185_000_000u64,
            "1.0.0",
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9",
            2u8,
            "indictrans2-quantized-onnx",
        ),
        (
            "model-ta-en-indictrans2",
            "Tamil <-> English Neural Translation (IndicTrans2)",
            LanguageCode::TaIn,
            LanguageCode::EnUs,
            185_000_000u64,
            "1.0.0",
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            2u8,
            "indictrans2-quantized-onnx",
        ),
        (
            "model-kn-ta-direct",
            "Kannada <-> Tamil Direct Neural Translation",
            LanguageCode::KnIn,
            LanguageCode::TaIn,
            195_000_000u64,
            "1.0.0",
            "5994471abb01112afcc18159f6cc74b4f511b99806da59b3caf5a9c173cacfc5",
            2u8,
            "indictrans2-quantized-onnx",
        ),
        (
            "asr-kn-streaming-sherpa",
            "Kannada Streaming ASR (Sherpa ONNX)",
            LanguageCode::KnIn,
            LanguageCode::KnIn,
            84_000_000u64,
            "0.9.2",
            "11a508955ea1050a300c638e2b6ab3e634b07be52f505bc55a38812ee69f04de",
            3u8,
            "zipformer-streaming-onnx",
        ),
        (
            "asr-ta-streaming-sherpa",
            "Tamil Streaming ASR (Sherpa ONNX)",
            LanguageCode::TaIn,
            LanguageCode::TaIn,
            84_000_000u64,
            "0.9.2",
            "2d711641b726b0295014ac619f8097b6bad924f88604b768f3047d43d335f116",
            3u8,
            "zipformer-streaming-onnx",
        ),
        (
            "asr-en-streaming-sherpa",
            "English Streaming ASR (Sherpa ONNX Zipformer)",
            LanguageCode::EnUs,
            LanguageCode::EnUs,
            75_000_000u64,
            "0.9.2",
            "4f8a9c33bb01112afcc18159f6cc74b4f511b99806da59b3caf5a9c173cacfc5",
            3u8,
            "zipformer-streaming-onnx",
        ),
        (
            "tts-kn-vits",
            "Kannada Neural TTS Voice Pack (MMS / VITS)",
            LanguageCode::KnIn,
            LanguageCode::KnIn,
            42_000_000u64,
            "1.0.0",
            "7b1c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            4u8,
            "vits-neural-tts",
        ),
        (
            "tts-ta-vits",
            "Tamil Neural TTS Voice Pack (MMS / VITS)",
            LanguageCode::TaIn,
            LanguageCode::TaIn,
            42_000_000u64,
            "1.0.0",
            "8c2d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9",
            4u8,
            "vits-neural-tts",
        ),
        (
            "tts-en-vits",
            "English Neural TTS Voice Pack (Piper / VITS)",
            LanguageCode::EnUs,
            LanguageCode::EnUs,
            38_000_000u64,
            "1.0.0",
            "9d3e5ea1050a300c638e2b6ab3e634b07be52f505bc55a38812ee69f04de11a5",
            4u8,
            "vits-neural-tts",
        ),
    ];

    for (id, name, src, tgt, size, ver, sha, stage, arch) in initial_packs {
        conn.execute(
            "INSERT INTO model_packs (id, name, source_language, target_language,
                                      size_bytes, version, sha256, is_installed,
                                      scheduled_stage, architecture, status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0, ?8, ?9, 'not_installed')
             ON CONFLICT(id) DO UPDATE SET
                architecture = excluded.architecture,
                scheduled_stage = excluded.scheduled_stage",
            params![
                id,
                name,
                src.as_bcp47(),
                tgt.as_bcp47(),
                size as i64,
                ver,
                sha,
                stage as i64,
                arch,
            ],
        )
        .map_err(|e| LinguaError::Database(format!("Failed to seed model pack {id}: {e}")))?;
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

pub fn list_detailed_model_packs(
    conn: &Connection,
) -> Result<Vec<ModelPackDetailedInfo>, LinguaError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, source_language, target_language, size_bytes,
                    version, sha256, is_installed, scheduled_stage,
                    COALESCE(architecture, 'indictrans2-quantized-onnx'),
                    install_path, installed_at,
                    COALESCE(status, 'not_installed')
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
            let is_installed_int: i64 = row.get(7)?;
            let scheduled_stage: i64 = row.get(8)?;
            let architecture: String = row.get(9)?;
            let install_path: Option<String> = row.get(10)?;
            let installed_at_str: Option<String> = row.get(11)?;
            let status_str: String = row.get(12)?;

            let source = LanguageCode::from_str(&src_str).unwrap_or(LanguageCode::EnUs);
            let target = LanguageCode::from_str(&tgt_str).unwrap_or(LanguageCode::KnIn);

            let state = if is_installed_int != 0 {
                let installed_at = installed_at_str
                    .as_deref()
                    .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(Utc::now);

                ModelInstallState::Installed {
                    installed_at,
                    install_path: install_path.clone().unwrap_or_default(),
                }
            } else if status_str == "corrupted" {
                ModelInstallState::Corrupted {
                    reason: "Integrity verification failed".into(),
                }
            } else {
                ModelInstallState::NotInstalled
            };

            Ok(ModelPackDetailedInfo {
                id,
                name,
                source,
                target,
                size_bytes: size_bytes as u64,
                version,
                sha256,
                state,
                architecture,
                storage_path: install_path,
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

pub fn get_detailed_model_pack(
    conn: &Connection,
    id: &str,
) -> Result<Option<ModelPackDetailedInfo>, LinguaError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, source_language, target_language, size_bytes,
                    version, sha256, is_installed, scheduled_stage,
                    COALESCE(architecture, 'indictrans2-quantized-onnx'),
                    install_path, installed_at,
                    COALESCE(status, 'not_installed')
             FROM model_packs WHERE id = ?1",
        )
        .map_err(|e| LinguaError::Database(e.to_string()))?;

    let result = stmt
        .query_row(params![id], |row| {
            let id: String = row.get(0)?;
            let name: String = row.get(1)?;
            let src_str: String = row.get(2)?;
            let tgt_str: String = row.get(3)?;
            let size_bytes: i64 = row.get(4)?;
            let version: String = row.get(5)?;
            let sha256: String = row.get(6)?;
            let is_installed_int: i64 = row.get(7)?;
            let scheduled_stage: i64 = row.get(8)?;
            let architecture: String = row.get(9)?;
            let install_path: Option<String> = row.get(10)?;
            let installed_at_str: Option<String> = row.get(11)?;
            let status_str: String = row.get(12)?;

            let source = LanguageCode::from_str(&src_str).unwrap_or(LanguageCode::EnUs);
            let target = LanguageCode::from_str(&tgt_str).unwrap_or(LanguageCode::KnIn);

            let state = if is_installed_int != 0 {
                let installed_at = installed_at_str
                    .as_deref()
                    .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(Utc::now);

                ModelInstallState::Installed {
                    installed_at,
                    install_path: install_path.clone().unwrap_or_default(),
                }
            } else if status_str == "corrupted" {
                ModelInstallState::Corrupted {
                    reason: "Integrity verification failed".into(),
                }
            } else {
                ModelInstallState::NotInstalled
            };

            Ok(ModelPackDetailedInfo {
                id,
                name,
                source,
                target,
                size_bytes: size_bytes as u64,
                version,
                sha256,
                state,
                architecture,
                storage_path: install_path,
                scheduled_stage: scheduled_stage as u8,
            })
        })
        .optional()
        .map_err(|e| LinguaError::Database(e.to_string()))?;

    Ok(result)
}

pub fn update_model_pack_install_state(
    conn: &Connection,
    id: &str,
    state: &ModelInstallState,
    checksum_verified: bool,
) -> Result<(), LinguaError> {
    match state {
        ModelInstallState::Installed {
            installed_at,
            install_path,
        } => {
            conn.execute(
                "UPDATE model_packs
                 SET is_installed = 1,
                     status = 'installed',
                     installed_at = ?1,
                     install_path = ?2,
                     checksum_verified = ?3
                 WHERE id = ?4",
                params![
                    installed_at.to_rfc3339(),
                    install_path,
                    if checksum_verified { 1 } else { 0 },
                    id
                ],
            )
            .map_err(|e| {
                LinguaError::Database(format!("Failed to update model install state: {e}"))
            })?;
        }
        ModelInstallState::Corrupted { .. } => {
            conn.execute(
                "UPDATE model_packs
                 SET is_installed = 0,
                     status = 'corrupted',
                     checksum_verified = 0
                 WHERE id = ?1",
                params![id],
            )
            .map_err(|e| {
                LinguaError::Database(format!("Failed to update model corrupt state: {e}"))
            })?;
        }
        ModelInstallState::NotInstalled => {
            conn.execute(
                "UPDATE model_packs
                 SET is_installed = 0,
                     status = 'not_installed',
                     installed_at = NULL,
                     install_path = NULL,
                     checksum_verified = 0
                 WHERE id = ?1",
                params![id],
            )
            .map_err(|e| {
                LinguaError::Database(format!("Failed to update model uninstall state: {e}"))
            })?;
        }
        ModelInstallState::Downloading { .. } => {
            conn.execute(
                "UPDATE model_packs
                 SET status = 'downloading'
                 WHERE id = ?1",
                params![id],
            )
            .map_err(|e| {
                LinguaError::Database(format!("Failed to update model downloading state: {e}"))
            })?;
        }
    }

    Ok(())
}

pub fn update_model_pack_sha256(
    conn: &Connection,
    id: &str,
    sha256: &str,
) -> Result<(), LinguaError> {
    conn.execute(
        "UPDATE model_packs SET sha256 = ?1 WHERE id = ?2",
        params![sha256, id],
    )
    .map_err(|e| LinguaError::Database(format!("Failed to update model pack sha256: {e}")))?;
    Ok(())
}

pub fn save_model_manifest(
    conn: &Connection,
    manifest: &ModelPackManifest,
) -> Result<(), LinguaError> {
    let json = serde_json::to_string(manifest)
        .map_err(|e| LinguaError::Validation(format!("Failed to serialize manifest: {e}")))?;

    conn.execute(
        "INSERT INTO model_manifests (id, version, manifest_json, signature, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(id) DO UPDATE SET
            version = excluded.version,
            manifest_json = excluded.manifest_json,
            signature = excluded.signature,
            updated_at = excluded.updated_at",
        params![
            manifest.id,
            manifest.version,
            json,
            manifest.signature,
            Utc::now().to_rfc3339(),
        ],
    )
    .map_err(|e| LinguaError::Database(format!("Failed to save model manifest: {e}")))?;

    Ok(())
}

pub fn get_model_manifest(
    conn: &Connection,
    id: &str,
) -> Result<Option<ModelPackManifest>, LinguaError> {
    let mut stmt = conn
        .prepare("SELECT manifest_json FROM model_manifests WHERE id = ?1")
        .map_err(|e| LinguaError::Database(e.to_string()))?;

    let result = stmt
        .query_row(params![id], |row| {
            let json: String = row.get(0)?;
            Ok(json)
        })
        .optional()
        .map_err(|e| LinguaError::Database(e.to_string()))?;

    match result {
        Some(json) => {
            let manifest: ModelPackManifest = serde_json::from_str(&json)
                .map_err(|e| LinguaError::Database(format!("Failed to parse manifest: {e}")))?;
            Ok(Some(manifest))
        }
        None => Ok(None),
    }
}
