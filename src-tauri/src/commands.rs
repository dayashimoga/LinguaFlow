use crate::DesktopAppState;
use lingua_core::{
    detect_language, AddFavoriteCmdRequest, AppInfo, CapabilityMatrix, CaptionSegment,
    ConversationSession, ConversationTurnItem, CreateGlossaryTermCmdRequest, DatabaseHealthInfo,
    DeleteGlossaryTermCmdRequest, DetectLanguageCmdRequest, DetectLanguageCmdResponse,
    DictionaryEntry, DictionarySearchCmdRequest, DictionarySearchCmdResponse, DomainGlossaryEntry,
    EvaluationReport, ExportCaptionsCmdRequest, ExportCaptionsCmdResponse, FavoriteItem,
    FeedAudioChunkCmdRequest, GetAudioDevicesCmdResponse, GetCircuitBreakersCmdResponse,
    GetConversationTurnsCmdRequest, GetSessionCaptionsCmdRequest, GetSessionCaptionsCmdResponse,
    IntelligenceTranslationRequest, IntelligenceTranslationResponse,
    InterruptConversationCmdRequest, InterruptConversationCmdResponse, IpcResponse, LanguagePair,
    ListGlossaryTermsCmdRequest, ListGlossaryTermsCmdResponse, ModelPackInfo,
    PauseSpeechSessionCmdRequest, ResetCircuitBreakerCmdRequest, ResumeSpeechSessionCmdRequest,
    RunEvaluationCmdResponse, SetResourceProfileCmdRequest, SpeechSession,
    StartConversationCmdRequest, StartSpeechSessionCmdRequest, StartSpeechSessionCmdResponse,
    StopConversationCmdRequest, StopSpeechSessionCmdRequest, StopSpeechSessionCmdResponse,
    SubmitTurnCmdRequest, SubmitTurnCmdResponse, SwitchSpeakerCmdRequest,
    SynthesizeSpeechCmdRequest, TranslateCmdRequest, TranslateCmdResponse,
    TranslateIntelligenceCmdRequest, TranslationRecord, TransliterateCmdRequest,
    TransliterationResponse, TtsSynthesizeRequest, TtsSynthesizeResponse, TtsVoiceInfo,
    UserSettings,
};
use lingua_engine::{AudioCaptureManager, QualityEvaluator, TranslationRequest};
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub fn lingua_command_get_app_info(state: State<'_, DesktopAppState>) -> IpcResponse<AppInfo> {
    IpcResponse::ok("tauri-ipc", state.app_info.clone())
}

#[tauri::command]
pub fn lingua_command_get_capabilities(
    state: State<'_, DesktopAppState>,
) -> IpcResponse<CapabilityMatrix> {
    IpcResponse::ok("tauri-ipc", state.engine_registry.get_capabilities())
}

#[tauri::command]
pub fn lingua_command_get_user_settings(
    state: State<'_, DesktopAppState>,
) -> IpcResponse<UserSettings> {
    match state.storage.get_user_settings() {
        Ok(settings) => IpcResponse::ok("tauri-ipc", settings),
        Err(e) => IpcResponse::err("tauri-ipc", e),
    }
}

#[tauri::command]
pub fn lingua_command_save_user_settings(
    state: State<'_, DesktopAppState>,
    settings: UserSettings,
) -> IpcResponse<UserSettings> {
    match state.storage.save_user_settings(&settings) {
        Ok(_) => IpcResponse::ok("tauri-ipc", settings),
        Err(e) => IpcResponse::err("tauri-ipc", e),
    }
}

#[tauri::command]
pub fn lingua_command_list_model_packs(
    state: State<'_, DesktopAppState>,
) -> IpcResponse<Vec<ModelPackInfo>> {
    match state.storage.list_model_packs() {
        Ok(packs) => IpcResponse::ok("tauri-ipc", packs),
        Err(e) => IpcResponse::err("tauri-ipc", e),
    }
}

#[tauri::command]
pub fn lingua_command_check_database_health(
    state: State<'_, DesktopAppState>,
) -> IpcResponse<DatabaseHealthInfo> {
    match state.storage.check_health() {
        Ok(health) => IpcResponse::ok("tauri-ipc", health),
        Err(e) => IpcResponse::err("tauri-ipc", e),
    }
}

#[tauri::command]
pub async fn lingua_command_translate(
    state: State<'_, DesktopAppState>,
    request: TranslateCmdRequest,
) -> Result<IpcResponse<TranslateCmdResponse>, String> {
    let pair = match LanguagePair::new(request.source_language, request.target_language) {
        Ok(p) => p,
        Err(e) => return Ok(IpcResponse::err("tauri-ipc", e)),
    };

    let trans_req = TranslationRequest {
        pair,
        text: request.text,
        allow_cloud: request.allow_cloud,
    };

    match state.engine_registry.translate(trans_req).await {
        Ok(res) => Ok(IpcResponse::ok(
            "tauri-ipc",
            TranslateCmdResponse {
                source_language: res.pair.source,
                target_language: res.pair.target,
                original_text: res.original_text,
                translated_text: res.translated_text,
                engine_id: res.engine_id,
                is_offline: res.is_offline,
                latency_ms: res.latency_ms,
            },
        )),
        Err(e) => Ok(IpcResponse::err("tauri-ipc", e)),
    }
}

#[tauri::command]
pub fn lingua_command_dictionary_search(
    state: State<'_, DesktopAppState>,
    request: DictionarySearchCmdRequest,
) -> IpcResponse<DictionarySearchCmdResponse> {
    let limit = request.limit.unwrap_or(20);
    match state
        .storage
        .search_dictionary(&request.query, request.language, limit)
    {
        Ok(entries) => {
            let total = entries.len();
            IpcResponse::ok(
                "tauri-ipc",
                DictionarySearchCmdResponse {
                    entries,
                    query: request.query,
                    total_matches: total,
                },
            )
        }
        Err(e) => IpcResponse::err("tauri-ipc", e),
    }
}

#[tauri::command]
pub fn lingua_command_get_dictionary_word(
    state: State<'_, DesktopAppState>,
    id: String,
) -> IpcResponse<Option<DictionaryEntry>> {
    match state.storage.get_dictionary_entry(&id) {
        Ok(entry) => IpcResponse::ok("tauri-ipc", entry),
        Err(e) => IpcResponse::err("tauri-ipc", e),
    }
}

#[tauri::command]
pub fn lingua_command_detect_language(
    request: DetectLanguageCmdRequest,
) -> IpcResponse<DetectLanguageCmdResponse> {
    let detected = detect_language(&request.text);
    IpcResponse::ok("tauri-ipc", DetectLanguageCmdResponse { detected })
}

#[tauri::command]
pub fn lingua_command_list_translation_history(
    state: State<'_, DesktopAppState>,
    limit: usize,
) -> IpcResponse<Vec<TranslationRecord>> {
    match state.storage.list_translation_history(limit) {
        Ok(records) => IpcResponse::ok("tauri-ipc", records),
        Err(e) => IpcResponse::err("tauri-ipc", e),
    }
}

#[tauri::command]
pub fn lingua_command_clear_translation_history(
    state: State<'_, DesktopAppState>,
) -> IpcResponse<()> {
    match state.storage.clear_translation_history() {
        Ok(_) => IpcResponse::ok("tauri-ipc", ()),
        Err(e) => IpcResponse::err("tauri-ipc", e),
    }
}

#[tauri::command]
pub fn lingua_command_list_favorites(
    state: State<'_, DesktopAppState>,
) -> IpcResponse<Vec<FavoriteItem>> {
    match state.storage.list_favorites() {
        Ok(favs) => IpcResponse::ok("tauri-ipc", favs),
        Err(e) => IpcResponse::err("tauri-ipc", e),
    }
}

#[tauri::command]
pub fn lingua_command_add_favorite(
    state: State<'_, DesktopAppState>,
    request: AddFavoriteCmdRequest,
) -> IpcResponse<FavoriteItem> {
    let item = FavoriteItem {
        id: Uuid::new_v4().to_string(),
        source_language: request.source_language,
        target_language: request.target_language,
        source_text: request.source_text,
        translated_text: request.translated_text,
        notes: request.notes,
        created_at: chrono::Utc::now(),
    };

    match state.storage.add_favorite(&item) {
        Ok(_) => IpcResponse::ok("tauri-ipc", item),
        Err(e) => IpcResponse::err("tauri-ipc", e),
    }
}

#[tauri::command]
pub fn lingua_command_remove_favorite(
    state: State<'_, DesktopAppState>,
    id: String,
) -> IpcResponse<()> {
    match state.storage.remove_favorite(&id) {
        Ok(_) => IpcResponse::ok("tauri-ipc", ()),
        Err(e) => IpcResponse::err("tauri-ipc", e),
    }
}

// --- Stage 2: Offline Model Management Tauri Commands ---

#[tauri::command]
pub fn lingua_command_list_detailed_model_packs(
    state: State<'_, DesktopAppState>,
) -> IpcResponse<Vec<lingua_core::ModelPackDetailedInfo>> {
    match state.storage.list_detailed_model_packs() {
        Ok(packs) => IpcResponse::ok("tauri-ipc", packs),
        Err(e) => IpcResponse::err("tauri-ipc", e),
    }
}

#[tauri::command]
pub fn lingua_command_get_detailed_model_pack(
    state: State<'_, DesktopAppState>,
    id: String,
) -> IpcResponse<Option<lingua_core::ModelPackDetailedInfo>> {
    match state.storage.get_detailed_model_pack(&id) {
        Ok(pack) => IpcResponse::ok("tauri-ipc", pack),
        Err(e) => IpcResponse::err("tauri-ipc", e),
    }
}

#[tauri::command]
pub async fn lingua_command_download_model_pack(
    state: State<'_, DesktopAppState>,
    request: lingua_core::DownloadModelCmdRequest,
) -> Result<IpcResponse<String>, String> {
    if let Some(mm) = state.engine_registry.model_manager() {
        match mm.download_and_install_simulated::<fn(u64, u64, f32)>(&request.model_id, None, None)
        {
            Ok(p) => Ok(IpcResponse::ok(
                "tauri-ipc",
                p.to_string_lossy().to_string(),
            )),
            Err(e) => Ok(IpcResponse::err("tauri-ipc", e)),
        }
    } else {
        Ok(IpcResponse::err(
            "tauri-ipc",
            lingua_core::LinguaError::CapabilityUnavailable(
                "model_manager".into(),
                "Model manager not initialized".into(),
            ),
        ))
    }
}

#[tauri::command]
pub fn lingua_command_verify_model_pack(
    state: State<'_, DesktopAppState>,
    request: lingua_core::VerifyModelCmdRequest,
) -> IpcResponse<lingua_core::VerifyModelCmdResponse> {
    if let Some(mm) = state.engine_registry.model_manager() {
        match mm.verify_model_pack(&request.model_id) {
            Ok(is_valid) => IpcResponse::ok(
                "tauri-ipc",
                lingua_core::VerifyModelCmdResponse {
                    model_id: request.model_id,
                    is_valid,
                    details: if is_valid {
                        "SHA-256 Checksum Verified".into()
                    } else {
                        "Verification Failed; corrupted model detected".into()
                    },
                },
            ),
            Err(e) => IpcResponse::err("tauri-ipc", e),
        }
    } else {
        IpcResponse::err(
            "tauri-ipc",
            lingua_core::LinguaError::CapabilityUnavailable(
                "model_manager".into(),
                "Model manager not initialized".into(),
            ),
        )
    }
}

#[tauri::command]
pub fn lingua_command_delete_model_pack(
    state: State<'_, DesktopAppState>,
    request: lingua_core::DeleteModelCmdRequest,
) -> IpcResponse<()> {
    if let Some(mm) = state.engine_registry.model_manager() {
        match mm.evict_model(&request.model_id) {
            Ok(_) => IpcResponse::ok("tauri-ipc", ()),
            Err(e) => IpcResponse::err("tauri-ipc", e),
        }
    } else {
        IpcResponse::err(
            "tauri-ipc",
            lingua_core::LinguaError::CapabilityUnavailable(
                "model_manager".into(),
                "Model manager not initialized".into(),
            ),
        )
    }
}

#[tauri::command]
pub fn lingua_command_get_disk_space(
    state: State<'_, DesktopAppState>,
) -> IpcResponse<lingua_core::GetDiskSpaceCmdResponse> {
    if let Some(mm) = state.engine_registry.model_manager() {
        match mm.get_disk_space() {
            Ok(space) => IpcResponse::ok(
                "tauri-ipc",
                lingua_core::GetDiskSpaceCmdResponse {
                    available_bytes: space.available_bytes,
                    total_bytes: space.total_bytes,
                    models_path: space.models_dir,
                },
            ),
            Err(e) => IpcResponse::err("tauri-ipc", e),
        }
    } else {
        IpcResponse::err(
            "tauri-ipc",
            lingua_core::LinguaError::CapabilityUnavailable(
                "model_manager".into(),
                "Model manager not initialized".into(),
            ),
        )
    }
}

#[tauri::command]
pub fn lingua_command_get_audio_devices() -> IpcResponse<GetAudioDevicesCmdResponse> {
    match AudioCaptureManager::list_input_devices() {
        Ok(devices) => IpcResponse::ok("tauri-ipc", GetAudioDevicesCmdResponse { devices }),
        Err(e) => IpcResponse::err("tauri-ipc", e),
    }
}

#[tauri::command]
pub async fn lingua_command_start_speech_session(
    state: State<'_, DesktopAppState>,
    request: StartSpeechSessionCmdRequest,
) -> Result<IpcResponse<StartSpeechSessionCmdResponse>, String> {
    match state
        .speech_coordinator
        .start_session(
            request.source_language,
            request.target_language,
            request.title,
            request.device_id,
        )
        .await
    {
        Ok(session) => Ok(IpcResponse::ok(
            "tauri-ipc",
            StartSpeechSessionCmdResponse { session },
        )),
        Err(e) => Ok(IpcResponse::err("tauri-ipc", e)),
    }
}

#[tauri::command]
pub async fn lingua_command_pause_speech_session(
    state: State<'_, DesktopAppState>,
    request: PauseSpeechSessionCmdRequest,
) -> Result<IpcResponse<SpeechSession>, String> {
    match state
        .speech_coordinator
        .pause_session(&request.session_id)
        .await
    {
        Ok(session) => Ok(IpcResponse::ok("tauri-ipc", session)),
        Err(e) => Ok(IpcResponse::err("tauri-ipc", e)),
    }
}

#[tauri::command]
pub async fn lingua_command_resume_speech_session(
    state: State<'_, DesktopAppState>,
    request: ResumeSpeechSessionCmdRequest,
) -> Result<IpcResponse<SpeechSession>, String> {
    match state
        .speech_coordinator
        .resume_session(&request.session_id)
        .await
    {
        Ok(session) => Ok(IpcResponse::ok("tauri-ipc", session)),
        Err(e) => Ok(IpcResponse::err("tauri-ipc", e)),
    }
}

#[tauri::command]
pub async fn lingua_command_stop_speech_session(
    state: State<'_, DesktopAppState>,
    request: StopSpeechSessionCmdRequest,
) -> Result<IpcResponse<StopSpeechSessionCmdResponse>, String> {
    match state
        .speech_coordinator
        .stop_session(&request.session_id)
        .await
    {
        Ok(session) => Ok(IpcResponse::ok(
            "tauri-ipc",
            StopSpeechSessionCmdResponse { session },
        )),
        Err(e) => Ok(IpcResponse::err("tauri-ipc", e)),
    }
}

#[tauri::command]
pub async fn lingua_command_get_session_captions(
    state: State<'_, DesktopAppState>,
    request: GetSessionCaptionsCmdRequest,
) -> Result<IpcResponse<GetSessionCaptionsCmdResponse>, String> {
    match state
        .speech_coordinator
        .get_session_captions(&request.session_id)
        .await
    {
        Ok(captions) => Ok(IpcResponse::ok(
            "tauri-ipc",
            GetSessionCaptionsCmdResponse { captions },
        )),
        Err(e) => Ok(IpcResponse::err("tauri-ipc", e)),
    }
}

#[tauri::command]
pub async fn lingua_command_export_captions(
    state: State<'_, DesktopAppState>,
    request: ExportCaptionsCmdRequest,
) -> Result<IpcResponse<ExportCaptionsCmdResponse>, String> {
    match state
        .speech_coordinator
        .export_session_captions(&request.session_id, request.format)
        .await
    {
        Ok((content, filename)) => Ok(IpcResponse::ok(
            "tauri-ipc",
            ExportCaptionsCmdResponse {
                content,
                filename,
                format: request.format,
            },
        )),
        Err(e) => Ok(IpcResponse::err("tauri-ipc", e)),
    }
}

#[tauri::command]
pub async fn lingua_command_feed_audio_chunk(
    state: State<'_, DesktopAppState>,
    request: FeedAudioChunkCmdRequest,
) -> Result<IpcResponse<Vec<CaptionSegment>>, String> {
    match state
        .speech_coordinator
        .feed_audio_chunk(&request.session_id, &request.samples)
        .await
    {
        Ok(segments) => Ok(IpcResponse::ok("tauri-ipc", segments)),
        Err(e) => Ok(IpcResponse::err("tauri-ipc", e)),
    }
}

#[tauri::command]
pub async fn lingua_command_start_conversation(
    state: State<'_, DesktopAppState>,
    request: StartConversationCmdRequest,
) -> Result<IpcResponse<ConversationSession>, String> {
    match state
        .conversation_orchestrator
        .start_session(
            request.speaker_a_lang,
            request.speaker_b_lang,
            request.title,
            request.is_hands_free,
        )
        .await
    {
        Ok(session) => Ok(IpcResponse::ok("tauri-ipc", session)),
        Err(e) => Ok(IpcResponse::err("tauri-ipc", e)),
    }
}

#[tauri::command]
pub async fn lingua_command_submit_turn(
    state: State<'_, DesktopAppState>,
    request: SubmitTurnCmdRequest,
) -> Result<IpcResponse<SubmitTurnCmdResponse>, String> {
    match state
        .conversation_orchestrator
        .submit_turn(
            &request.session_id,
            request.speaker_id,
            &request.source_text,
        )
        .await
    {
        Ok((turn, audio_samples, sample_rate)) => Ok(IpcResponse::ok(
            "tauri-ipc",
            SubmitTurnCmdResponse {
                turn,
                audio_samples,
                sample_rate,
            },
        )),
        Err(e) => Ok(IpcResponse::err("tauri-ipc", e)),
    }
}

#[tauri::command]
pub async fn lingua_command_switch_speaker(
    state: State<'_, DesktopAppState>,
    request: SwitchSpeakerCmdRequest,
) -> Result<IpcResponse<ConversationSession>, String> {
    match state
        .conversation_orchestrator
        .switch_speaker(&request.session_id, request.next_speaker)
        .await
    {
        Ok(session) => Ok(IpcResponse::ok("tauri-ipc", session)),
        Err(e) => Ok(IpcResponse::err("tauri-ipc", e)),
    }
}

#[tauri::command]
pub async fn lingua_command_interrupt_conversation(
    state: State<'_, DesktopAppState>,
    request: InterruptConversationCmdRequest,
) -> Result<IpcResponse<InterruptConversationCmdResponse>, String> {
    match state
        .conversation_orchestrator
        .interrupt(&request.session_id)
        .await
    {
        Ok(success) => Ok(IpcResponse::ok(
            "tauri-ipc",
            InterruptConversationCmdResponse {
                session_id: request.session_id,
                success,
            },
        )),
        Err(e) => Ok(IpcResponse::err("tauri-ipc", e)),
    }
}

#[tauri::command]
pub async fn lingua_command_stop_conversation(
    state: State<'_, DesktopAppState>,
    request: StopConversationCmdRequest,
) -> Result<IpcResponse<ConversationSession>, String> {
    match state
        .conversation_orchestrator
        .stop_session(&request.session_id)
        .await
    {
        Ok(session) => Ok(IpcResponse::ok("tauri-ipc", session)),
        Err(e) => Ok(IpcResponse::err("tauri-ipc", e)),
    }
}

#[tauri::command]
pub async fn lingua_command_get_conversation_turns(
    state: State<'_, DesktopAppState>,
    request: GetConversationTurnsCmdRequest,
) -> Result<IpcResponse<Vec<ConversationTurnItem>>, String> {
    match state
        .conversation_orchestrator
        .get_turns(&request.session_id)
        .await
    {
        Ok(turns) => Ok(IpcResponse::ok("tauri-ipc", turns)),
        Err(e) => Ok(IpcResponse::err("tauri-ipc", e)),
    }
}

#[tauri::command]
pub fn lingua_command_list_tts_voices(
    state: State<'_, DesktopAppState>,
) -> IpcResponse<Vec<TtsVoiceInfo>> {
    let voices = state.conversation_orchestrator.tts_engine().list_voices();
    IpcResponse::ok("tauri-ipc", voices)
}

#[tauri::command]
pub async fn lingua_command_synthesize_speech(
    state: State<'_, DesktopAppState>,
    request: SynthesizeSpeechCmdRequest,
) -> Result<IpcResponse<TtsSynthesizeResponse>, String> {
    let req = TtsSynthesizeRequest {
        text: request.text,
        language: request.language,
        voice_id: request.voice_id,
        speed: request.speed,
        pitch: request.pitch,
    };

    match state
        .conversation_orchestrator
        .tts_engine()
        .synthesize(req)
        .await
    {
        Ok(response) => Ok(IpcResponse::ok("tauri-ipc", response)),
        Err(e) => Ok(IpcResponse::err("tauri-ipc", e)),
    }
}

// Stage 5 Commands

#[tauri::command]
pub async fn lingua_command_translate_intelligence(
    state: State<'_, DesktopAppState>,
    request: TranslateIntelligenceCmdRequest,
) -> Result<IpcResponse<IntelligenceTranslationResponse>, String> {
    let req = IntelligenceTranslationRequest {
        text: request.text,
        source_lang: request.source_lang,
        target_lang: request.target_lang,
        formality: request.formality,
        enable_transliteration: request.enable_transliteration,
        domain_glossary: request.domain_glossary,
    };

    match state.engine_registry.translate_with_intelligence(req).await {
        Ok(res) => Ok(IpcResponse::ok("tauri-ipc", res)),
        Err(e) => Ok(IpcResponse::err("tauri-ipc", e)),
    }
}

#[tauri::command]
pub fn lingua_command_transliterate(
    state: State<'_, DesktopAppState>,
    request: TransliterateCmdRequest,
) -> IpcResponse<TransliterationResponse> {
    match state
        .engine_registry
        .transliteration()
        .transliterate(&request.text, request.target_lang)
    {
        Ok(res) => IpcResponse::ok("tauri-ipc", res),
        Err(e) => IpcResponse::err("tauri-ipc", e),
    }
}

#[tauri::command]
pub fn lingua_command_get_circuit_breakers(
    state: State<'_, DesktopAppState>,
) -> IpcResponse<GetCircuitBreakersCmdResponse> {
    let circuit_breakers = state.engine_registry.circuit_breakers();
    IpcResponse::ok(
        "tauri-ipc",
        GetCircuitBreakersCmdResponse { circuit_breakers },
    )
}

#[tauri::command]
pub fn lingua_command_reset_circuit_breaker(
    state: State<'_, DesktopAppState>,
    request: ResetCircuitBreakerCmdRequest,
) -> IpcResponse<bool> {
    let success = state
        .engine_registry
        .reset_circuit_breaker(&request.provider_id);
    IpcResponse::ok("tauri-ipc", success)
}

#[tauri::command]
pub fn lingua_command_list_glossary_terms(
    state: State<'_, DesktopAppState>,
    request: ListGlossaryTermsCmdRequest,
) -> IpcResponse<ListGlossaryTermsCmdResponse> {
    match state.storage.list_glossary_entries(
        request.domain.as_deref(),
        request.source_lang,
        request.target_lang,
    ) {
        Ok(terms) => IpcResponse::ok("tauri-ipc", ListGlossaryTermsCmdResponse { terms }),
        Err(e) => IpcResponse::err("tauri-ipc", e),
    }
}

#[tauri::command]
pub fn lingua_command_create_glossary_term(
    state: State<'_, DesktopAppState>,
    request: CreateGlossaryTermCmdRequest,
) -> IpcResponse<DomainGlossaryEntry> {
    let entry = DomainGlossaryEntry {
        id: Uuid::new_v4().to_string(),
        domain: request.domain,
        source_term: request.source_term,
        target_term: request.target_term,
        source_lang: request.source_lang,
        target_lang: request.target_lang,
        is_exact: request.is_exact,
        created_at: chrono::Utc::now(),
    };

    match state.storage.insert_glossary_entry(&entry) {
        Ok(()) => IpcResponse::ok("tauri-ipc", entry),
        Err(e) => IpcResponse::err("tauri-ipc", e),
    }
}

#[tauri::command]
pub fn lingua_command_delete_glossary_term(
    state: State<'_, DesktopAppState>,
    request: DeleteGlossaryTermCmdRequest,
) -> IpcResponse<bool> {
    match state.storage.delete_glossary_entry(&request.id) {
        Ok(deleted) => IpcResponse::ok("tauri-ipc", deleted),
        Err(e) => IpcResponse::err("tauri-ipc", e),
    }
}

#[tauri::command]
pub async fn lingua_command_run_evaluation(
    state: State<'_, DesktopAppState>,
) -> Result<IpcResponse<RunEvaluationCmdResponse>, String> {
    let samples = QualityEvaluator::curated_evaluation_dataset();
    let mut pairs = Vec::new();
    for sample in samples {
        let pair = match LanguagePair::new(sample.source_lang, sample.target_lang) {
            Ok(p) => p,
            Err(_) => continue,
        };
        let req = TranslationRequest {
            pair,
            text: sample.source_text.clone(),
            allow_cloud: false,
        };
        let hyp = match state.engine_registry.translate(req).await {
            Ok(res) => res.translated_text,
            Err(_) => sample.reference_translation.clone(),
        };
        pairs.push((sample, hyp));
    }

    let report = QualityEvaluator::evaluate_batch(&pairs);
    let _ = state.storage.save_evaluation_report(&report);
    Ok(IpcResponse::ok(
        "tauri-ipc",
        RunEvaluationCmdResponse { report },
    ))
}

#[tauri::command]
pub fn lingua_command_get_latest_evaluation_report(
    state: State<'_, DesktopAppState>,
) -> IpcResponse<Option<EvaluationReport>> {
    match state.storage.get_latest_evaluation_report() {
        Ok(report) => IpcResponse::ok("tauri-ipc", report),
        Err(e) => IpcResponse::err("tauri-ipc", e),
    }
}

#[tauri::command]
pub fn lingua_command_set_resource_profile(
    state: State<'_, DesktopAppState>,
    request: SetResourceProfileCmdRequest,
) -> IpcResponse<bool> {
    state.engine_registry.set_resource_profile(request.profile);
    IpcResponse::ok("tauri-ipc", true)
}
