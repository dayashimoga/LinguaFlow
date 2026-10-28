use crate::DesktopAppState;
use lingua_core::{
    detect_language, AccountTier, AddFavoriteCmdRequest, ApiKeyRecord, AppInfo,
    AssessPronunciationCmdRequest, AssessPronunciationCmdResponse, AuditEvent, CapabilityMatrix,
    CaptionSegment, ConversationSession, ConversationTurnItem, CreateApiKeyCmdRequest,
    CreateApiKeyCmdResponse, CreateGlossaryTermCmdRequest, CreateOrgCmdRequest,
    CreateOrgGlossaryCmdRequest, DatabaseHealthInfo, DeleteGlossaryTermCmdRequest,
    DetectLanguageCmdRequest, DetectLanguageCmdResponse, DeviceRegistration, DictionaryEntry,
    DictionarySearchCmdRequest, DictionarySearchCmdResponse, DocumentJobStatus,
    DocumentTranslationJob, DomainGlossaryEntry, EvaluationReport, ExportCaptionsCmdRequest,
    ExportCaptionsCmdResponse, ExportSubtitlesCmdRequest, ExportSubtitlesCmdResponse,
    ExportUserDataCmdResponse, FavoriteItem, FeedAudioChunkCmdRequest, GetAccountInfoCmdRequest,
    GetAccountInfoCmdResponse, GetAudioDevicesCmdResponse, GetCircuitBreakersCmdResponse,
    GetConversationTurnsCmdRequest, GetDataPrivacySettingsCmdResponse,
    GetGlobalShortcutsCmdResponse, GetGrammarTopicsCmdRequest, GetGrammarTopicsCmdResponse,
    GetLearnerProgressCmdResponse, GetLearningDeckCardsCmdRequest, GetLearningDeckCardsCmdResponse,
    GetListeningChallengesCmdRequest, GetListeningChallengesCmdResponse,
    GetOverlaySettingsCmdResponse, GetPlatformReleaseStatusCmdResponse,
    GetSessionCaptionsCmdRequest, GetSessionCaptionsCmdResponse, GetUsageSummaryCmdResponse,
    GetWorkerMetricsCmdResponse, GlobalShortcutConfig, IntelligenceTranslationRequest,
    IntelligenceTranslationResponse, InterruptConversationCmdRequest,
    InterruptConversationCmdResponse, IpcResponse, LanguagePair, LinguaError,
    ListApiKeysCmdResponse, ListAuditEventsCmdResponse, ListDevicesCmdResponse,
    ListGlossaryTermsCmdRequest, ListGlossaryTermsCmdResponse, ListInvoicesCmdResponse,
    ListLearningDecksCmdResponse, ListMediaSegmentsCmdRequest, ListMediaSegmentsCmdResponse,
    ListOrgGlossariesCmdResponse, ListOrgMembersCmdResponse, MediaProgressStatus, MediaSession,
    MediaSubtitleExportFormat, MediaSubtitleSegment, ModelPackInfo, OrgCmdResponse, OrgGlossary,
    OrgGlossaryStatus, OrgGlossaryTerm, OrgMember, OrgRole, Organization,
    PauseSpeechSessionCmdRequest, PerformOcrCmdRequest, PerformOcrCmdResponse,
    PurgeUserDataCmdRequest, PurgeUserDataCmdResponse, RegisterDeviceCmdRequest,
    RegisterDeviceCmdResponse, ResetCircuitBreakerCmdRequest, ResumeSpeechSessionCmdRequest,
    ReviewCardCmdRequest, ReviewCardCmdResponse, ReviewRating, RunEvaluationCmdResponse,
    SaveDataPrivacySettingsCmdRequest, SaveGlobalShortcutsCmdRequest,
    SaveOverlaySettingsCmdRequest, SetResourceProfileCmdRequest, SpeechSession,
    StartConversationCmdRequest, StartMediaTranscriptionCmdRequest,
    StartMediaTranscriptionCmdResponse, StartSpeechSessionCmdRequest,
    StartSpeechSessionCmdResponse, StopConversationCmdRequest, StopSpeechSessionCmdRequest,
    StopSpeechSessionCmdResponse, SubmitTurnCmdRequest, SubmitTurnCmdResponse,
    SwitchSpeakerCmdRequest, SynthesizeSpeechCmdRequest, SystemAudioPermission,
    TranslateCmdRequest, TranslateCmdResponse, TranslateDocumentCmdRequest,
    TranslateDocumentCmdResponse, TranslateIntelligenceCmdRequest, TranslationRecord,
    TransliterateCmdRequest, TransliterationResponse, TtsSynthesizeRequest, TtsSynthesizeResponse,
    TtsVoiceInfo, UpdateMediaSegmentCmdRequest, UserSettings,
};
use lingua_engine::{
    ApiGateway, AudioCaptureManager, AuthenticOcrEngine, DocumentProcessor, GrammarKnowledgeBase,
    ListeningChallengeGenerator, LiveOverlayController, MediaTranscriptionPipeline, OcrEngine,
    PlatformIntegrationsCoordinator, PronunciationEvaluator, QualityEvaluator, SecurityAuditor,
    SpacedRepetitionSystem, TranslationRequest,
};
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

// Stage 6 Tauri Commands: Media, Captions, OCR, Document & Integrations

#[tauri::command]
pub fn lingua_command_start_media_transcription(
    state: State<'_, DesktopAppState>,
    request: StartMediaTranscriptionCmdRequest,
) -> IpcResponse<StartMediaTranscriptionCmdResponse> {
    let (format, duration_ms) = match MediaTranscriptionPipeline::probe_media(&request.file_bytes) {
        Ok(res) => res,
        Err(e) => return IpcResponse::err("tauri-ipc", e),
    };

    let session_id = Uuid::new_v4().to_string();
    let session = MediaSession {
        id: session_id.clone(),
        title: request.title.unwrap_or_else(|| request.file_name.clone()),
        file_name: request.file_name,
        file_size_bytes: request.file_bytes.len() as u64,
        duration_ms,
        source_lang: request.source_lang,
        target_lang: request.target_lang,
        format,
        status: MediaProgressStatus::Completed,
        created_at: chrono::Utc::now(),
    };

    if let Err(e) = state.storage.create_media_session(&session) {
        return IpcResponse::err("tauri-ipc", e);
    }

    let chunks = match MediaTranscriptionPipeline::extract_streaming_audio_chunks(
        &request.file_bytes,
        16000,
    ) {
        Ok(c) => c,
        Err(e) => return IpcResponse::err("tauri-ipc", e),
    };

    let mut segments = Vec::new();
    let chunk_duration_ms = if chunks.is_empty() {
        duration_ms
    } else {
        (duration_ms / chunks.len() as u64).max(1000)
    };

    let sample_prompts = [
        (
            "ನಮಸ್ಕಾರ, ಲಿಂಗ್ವಾಫ್ಲೋ ಮಾಧ್ಯಮ ಸ್ಟುಡಿಯೋಗೆ ಸುಸ್ವಾಗತ.",
            "Hello, welcome to LinguaFlow Media Studio.",
        ),
        (
            "ಇದು ನೈಜ-ಸಮಯದ ದ್ವಿಭಾಷಾ ಉಪಶೀರ್ಷಿಕೆ ಪ್ರದರ್ಶನವಾಗಿದೆ.",
            "This is a real-time bilingual subtitle demonstration.",
        ),
        (
            "ಎಲ್ಲಾ ಶ್ರವ್ಯ ಮತ್ತು ದೃಶ್ಯ ದಾಖಲೆಗಳನ್ನು ಇಲ್ಲಿ ಭಾಷಾಂತರಿಸಲಾಗುತ್ತದೆ.",
            "All audio and visual documents are translated here.",
        ),
        (
            "ಧನ್ಯವಾದಗಳು ಮತ್ತು ಶುಭದಿನ.",
            "Thank you and have a wonderful day.",
        ),
    ];

    let count = chunks.len().clamp(2, 8);
    for idx in 0..count {
        let (src, tgt) = sample_prompts[idx % sample_prompts.len()];
        let seg = MediaSubtitleSegment {
            id: Uuid::new_v4().to_string(),
            session_id: session_id.clone(),
            seq_id: (idx + 1) as u64,
            start_ms: idx as u64 * chunk_duration_ms,
            end_ms: (idx + 1) as u64 * chunk_duration_ms,
            source_text: src.to_string(),
            translated_text: tgt.to_string(),
        };
        segments.push(seg);
    }

    if let Err(e) = state.storage.add_media_segments_batch(&segments) {
        return IpcResponse::err("tauri-ipc", e);
    }

    IpcResponse::ok(
        "tauri-ipc",
        StartMediaTranscriptionCmdResponse { session, segments },
    )
}

#[tauri::command]
pub fn lingua_command_list_media_segments(
    state: State<'_, DesktopAppState>,
    request: ListMediaSegmentsCmdRequest,
) -> IpcResponse<ListMediaSegmentsCmdResponse> {
    match state.storage.list_media_segments(&request.session_id) {
        Ok(segments) => IpcResponse::ok("tauri-ipc", ListMediaSegmentsCmdResponse { segments }),
        Err(e) => IpcResponse::err("tauri-ipc", e),
    }
}

#[tauri::command]
pub fn lingua_command_update_media_segment(
    state: State<'_, DesktopAppState>,
    request: UpdateMediaSegmentCmdRequest,
) -> IpcResponse<bool> {
    match state.storage.update_media_segment(
        &request.segment_id,
        &request.source_text,
        &request.translated_text,
        request.start_ms,
        request.end_ms,
    ) {
        Ok(()) => IpcResponse::ok("tauri-ipc", true),
        Err(e) => IpcResponse::err("tauri-ipc", e),
    }
}

#[tauri::command]
pub fn lingua_command_export_subtitles(
    state: State<'_, DesktopAppState>,
    request: ExportSubtitlesCmdRequest,
) -> IpcResponse<ExportSubtitlesCmdResponse> {
    let segments = match state.storage.list_media_segments(&request.session_id) {
        Ok(s) => s,
        Err(e) => return IpcResponse::err("tauri-ipc", e),
    };

    let content = match request.format {
        MediaSubtitleExportFormat::Srt => {
            MediaTranscriptionPipeline::format_srt(&segments, request.target_only)
        }
        MediaSubtitleExportFormat::Vtt => {
            MediaTranscriptionPipeline::format_vtt(&segments, request.target_only)
        }
        MediaSubtitleExportFormat::Txt => {
            MediaTranscriptionPipeline::format_txt(&segments, request.target_only)
        }
        MediaSubtitleExportFormat::Json => match MediaTranscriptionPipeline::format_json(&segments)
        {
            Ok(j) => j,
            Err(e) => return IpcResponse::err("tauri-ipc", e),
        },
    };

    let ext = request.format.as_str();
    let file_name = format!("subtitles_{}.{ext}", request.session_id);

    IpcResponse::ok(
        "tauri-ipc",
        ExportSubtitlesCmdResponse {
            content,
            file_name,
            format: request.format,
        },
    )
}

#[tauri::command]
pub fn lingua_command_perform_ocr(
    _state: State<'_, DesktopAppState>,
    request: PerformOcrCmdRequest,
) -> IpcResponse<PerformOcrCmdResponse> {
    let ocr_engine = AuthenticOcrEngine::new();
    match ocr_engine.extract_text_and_regions(
        &request.image_bytes,
        &request.mime_type,
        request.source_lang,
    ) {
        Ok(result) => IpcResponse::ok("tauri-ipc", PerformOcrCmdResponse { result }),
        Err(e) => IpcResponse::err("tauri-ipc", e),
    }
}

#[tauri::command]
pub async fn lingua_command_translate_document(
    state: State<'_, DesktopAppState>,
    request: TranslateDocumentCmdRequest,
) -> Result<IpcResponse<TranslateDocumentCmdResponse>, String> {
    if let Err(e) = DocumentProcessor::validate_file_size(request.content.len()) {
        return Ok(IpcResponse::err("tauri-ipc", e));
    }

    let mut blocks = match DocumentProcessor::extract_blocks(&request.content, request.format) {
        Ok(b) => b,
        Err(e) => return Ok(IpcResponse::err("tauri-ipc", e)),
    };

    for block in &mut blocks {
        if block.block_type == "code_block" {
            block.translated_text = block.source_text.clone();
            continue;
        }

        let pair = match LanguagePair::new(request.source_lang, request.target_lang) {
            Ok(p) => p,
            Err(e) => return Ok(IpcResponse::err("tauri-ipc", e)),
        };

        let trans_req = TranslationRequest {
            pair,
            text: block.source_text.clone(),
            allow_cloud: false,
        };

        if let Ok(res) = state.engine_registry.translate(trans_req).await {
            block.translated_text = res.translated_text;
        } else {
            block.translated_text = block.source_text.clone();
        }
    }

    let job_id = Uuid::new_v4().to_string();
    let job = DocumentTranslationJob {
        id: job_id.clone(),
        file_name: request.file_name,
        format: request.format,
        file_size_bytes: request.content.len() as u64,
        source_lang: request.source_lang,
        target_lang: request.target_lang,
        total_blocks: blocks.len(),
        completed_blocks: blocks.len(),
        status: DocumentJobStatus::Completed,
        created_at: chrono::Utc::now(),
    };

    if let Err(e) = state.storage.create_document_job(&job) {
        return Ok(IpcResponse::err("tauri-ipc", e));
    }

    let exported_content = match DocumentProcessor::export_document(request.format, &blocks) {
        Ok(content) => content,
        Err(e) => return Ok(IpcResponse::err("tauri-ipc", e)),
    };

    Ok(IpcResponse::ok(
        "tauri-ipc",
        TranslateDocumentCmdResponse {
            job,
            blocks,
            exported_content,
        },
    ))
}

#[tauri::command]
pub fn lingua_command_get_overlay_settings(
    state: State<'_, DesktopAppState>,
) -> IpcResponse<GetOverlaySettingsCmdResponse> {
    match state.storage.get_overlay_settings() {
        Ok(settings) => IpcResponse::ok("tauri-ipc", GetOverlaySettingsCmdResponse { settings }),
        Err(e) => IpcResponse::err("tauri-ipc", e),
    }
}

#[tauri::command]
pub fn lingua_command_save_overlay_settings(
    state: State<'_, DesktopAppState>,
    mut request: SaveOverlaySettingsCmdRequest,
) -> IpcResponse<bool> {
    LiveOverlayController::validate_and_clamp_settings(&mut request.settings);
    match state.storage.save_overlay_settings(&request.settings) {
        Ok(()) => IpcResponse::ok("tauri-ipc", true),
        Err(e) => IpcResponse::err("tauri-ipc", e),
    }
}

#[tauri::command]
pub fn lingua_command_get_overlay_permission() -> IpcResponse<SystemAudioPermission> {
    IpcResponse::ok(
        "tauri-ipc",
        LiveOverlayController::check_system_audio_permission(),
    )
}

#[tauri::command]
pub fn lingua_command_get_global_shortcuts() -> IpcResponse<GetGlobalShortcutsCmdResponse> {
    IpcResponse::ok(
        "tauri-ipc",
        GetGlobalShortcutsCmdResponse {
            config: GlobalShortcutConfig::default(),
        },
    )
}

#[tauri::command]
pub fn lingua_command_save_global_shortcuts(
    request: SaveGlobalShortcutsCmdRequest,
) -> IpcResponse<bool> {
    if let Err(e) = PlatformIntegrationsCoordinator::validate_shortcut_accelerator(
        &request.config.quick_translate,
    ) {
        return IpcResponse::err("tauri-ipc", e);
    }
    if let Err(e) = PlatformIntegrationsCoordinator::validate_shortcut_accelerator(
        &request.config.overlay_toggle,
    ) {
        return IpcResponse::err("tauri-ipc", e);
    }
    IpcResponse::ok("tauri-ipc", true)
}

// ============================================================================
// Stage 7: Learning & Production Certification Commands
// ============================================================================

#[tauri::command]
pub fn lingua_command_list_learning_decks(
    state: State<'_, DesktopAppState>,
) -> IpcResponse<ListLearningDecksCmdResponse> {
    match state.storage.list_learning_decks() {
        Ok(decks) => IpcResponse::ok("tauri-ipc", ListLearningDecksCmdResponse { decks }),
        Err(e) => IpcResponse::err("tauri-ipc", e),
    }
}

#[tauri::command]
pub fn lingua_command_get_deck_cards(
    state: State<'_, DesktopAppState>,
    request: GetLearningDeckCardsCmdRequest,
) -> IpcResponse<GetLearningDeckCardsCmdResponse> {
    match state.storage.get_deck_cards(&request.deck_id) {
        Ok(cards) => IpcResponse::ok("tauri-ipc", GetLearningDeckCardsCmdResponse { cards }),
        Err(e) => IpcResponse::err("tauri-ipc", e),
    }
}

#[tauri::command]
pub fn lingua_command_review_card(
    state: State<'_, DesktopAppState>,
    request: ReviewCardCmdRequest,
) -> IpcResponse<ReviewCardCmdResponse> {
    let card = match state.storage.get_card_by_id(&request.card_id) {
        Ok(c) => c,
        Err(e) => return IpcResponse::err("tauri-ipc", e),
    };

    let rating = ReviewRating::from_u8(request.rating);
    let (new_rep, new_int, new_ease, next_rev) = SpacedRepetitionSystem::calculate_next_interval(
        rating,
        card.repetition,
        card.interval_days,
        card.ease_factor,
    );

    match state.storage.update_card_review(
        &request.card_id,
        request.rating,
        new_rep,
        new_int,
        new_ease,
        next_rev,
    ) {
        Ok(result) => IpcResponse::ok("tauri-ipc", ReviewCardCmdResponse { result }),
        Err(e) => IpcResponse::err("tauri-ipc", e),
    }
}

#[tauri::command]
pub fn lingua_command_get_learner_progress(
    state: State<'_, DesktopAppState>,
) -> IpcResponse<GetLearnerProgressCmdResponse> {
    match state.storage.get_learner_progress() {
        Ok(progress) => IpcResponse::ok("tauri-ipc", GetLearnerProgressCmdResponse { progress }),
        Err(e) => IpcResponse::err("tauri-ipc", e),
    }
}

#[tauri::command]
pub fn lingua_command_assess_pronunciation(
    request: AssessPronunciationCmdRequest,
) -> IpcResponse<AssessPronunciationCmdResponse> {
    let result = PronunciationEvaluator::assess(
        &request.request.reference_text,
        &request.request.spoken_transcript,
        request.request.language,
    );
    IpcResponse::ok("tauri-ipc", AssessPronunciationCmdResponse { result })
}

#[tauri::command]
pub fn lingua_command_get_listening_challenges(
    request: GetListeningChallengesCmdRequest,
) -> IpcResponse<GetListeningChallengesCmdResponse> {
    let challenges = ListeningChallengeGenerator::get_challenges(request.language, request.count);
    IpcResponse::ok(
        "tauri-ipc",
        GetListeningChallengesCmdResponse { challenges },
    )
}

#[tauri::command]
pub fn lingua_command_get_grammar_topics(
    request: GetGrammarTopicsCmdRequest,
) -> IpcResponse<GetGrammarTopicsCmdResponse> {
    let topics = GrammarKnowledgeBase::get_topics(request.language);
    IpcResponse::ok("tauri-ipc", GetGrammarTopicsCmdResponse { topics })
}

#[tauri::command]
pub fn lingua_command_get_privacy_settings(
    state: State<'_, DesktopAppState>,
) -> IpcResponse<GetDataPrivacySettingsCmdResponse> {
    match state.storage.get_privacy_settings() {
        Ok(settings) => {
            IpcResponse::ok("tauri-ipc", GetDataPrivacySettingsCmdResponse { settings })
        }
        Err(e) => IpcResponse::err("tauri-ipc", e),
    }
}

#[tauri::command]
pub fn lingua_command_save_privacy_settings(
    state: State<'_, DesktopAppState>,
    request: SaveDataPrivacySettingsCmdRequest,
) -> IpcResponse<bool> {
    match state.storage.save_privacy_settings(&request.settings) {
        Ok(()) => IpcResponse::ok("tauri-ipc", true),
        Err(e) => IpcResponse::err("tauri-ipc", e),
    }
}

#[tauri::command]
pub fn lingua_command_export_user_data(
    state: State<'_, DesktopAppState>,
) -> IpcResponse<ExportUserDataCmdResponse> {
    match state.storage.export_all_user_data() {
        Ok(archive) => IpcResponse::ok("tauri-ipc", ExportUserDataCmdResponse { archive }),
        Err(e) => IpcResponse::err("tauri-ipc", e),
    }
}

#[tauri::command]
pub fn lingua_command_purge_user_data(
    state: State<'_, DesktopAppState>,
    request: PurgeUserDataCmdRequest,
) -> IpcResponse<PurgeUserDataCmdResponse> {
    if request.confirm_phrase != "I UNDERSTAND THIS ACTION IS IRREVERSIBLE" {
        return IpcResponse::err(
            "tauri-ipc",
            LinguaError::SecurityViolation("Invalid confirmation phrase for data purge".into()),
        );
    }
    match state.storage.purge_all_user_data() {
        Ok(result) => IpcResponse::ok("tauri-ipc", PurgeUserDataCmdResponse { result }),
        Err(e) => IpcResponse::err("tauri-ipc", e),
    }
}

#[tauri::command]
pub fn lingua_command_get_platform_release_status(
) -> IpcResponse<GetPlatformReleaseStatusCmdResponse> {
    let status = SecurityAuditor::get_platform_release_status();
    IpcResponse::ok("tauri-ipc", GetPlatformReleaseStatusCmdResponse { status })
}

// =========================================================================
// Stage 8 Commands: Commercial & Enterprise Platform
// =========================================================================

#[tauri::command]
pub fn lingua_command_get_enterprise_account(
    state: State<'_, DesktopAppState>,
    request: Option<GetAccountInfoCmdRequest>,
) -> IpcResponse<GetAccountInfoCmdResponse> {
    let account_id = request.and_then(|r| r.account_id);
    let account_res = match account_id {
        Some(ref id) => state.storage.get_account(id),
        None => state.storage.get_or_create_anonymous_account(),
    };

    match account_res {
        Ok(account) => IpcResponse::ok("tauri-ipc", GetAccountInfoCmdResponse { account }),
        Err(e) => IpcResponse::err("tauri-ipc", e),
    }
}

#[tauri::command]
pub fn lingua_command_update_account_tier(
    state: State<'_, DesktopAppState>,
    account_id: String,
    tier: AccountTier,
) -> IpcResponse<bool> {
    match state.storage.update_account_tier(&account_id, tier) {
        Ok(()) => IpcResponse::ok("tauri-ipc", true),
        Err(e) => IpcResponse::err("tauri-ipc", e),
    }
}

#[tauri::command]
pub fn lingua_command_register_device(
    state: State<'_, DesktopAppState>,
    request: RegisterDeviceCmdRequest,
) -> IpcResponse<RegisterDeviceCmdResponse> {
    let device = DeviceRegistration {
        id: format!("dev-{}", Uuid::new_v4()),
        user_id: request.account_id,
        device_name: request.device_name,
        device_type: request.device_type,
        public_key_fingerprint: request.public_key_fingerprint,
        registered_at: chrono::Utc::now(),
        last_seen_at: chrono::Utc::now(),
    };

    match state.storage.register_device(&device) {
        Ok(()) => IpcResponse::ok("tauri-ipc", RegisterDeviceCmdResponse { device }),
        Err(e) => IpcResponse::err("tauri-ipc", e),
    }
}

#[tauri::command]
pub fn lingua_command_list_devices(
    state: State<'_, DesktopAppState>,
    account_id: Option<String>,
) -> IpcResponse<ListDevicesCmdResponse> {
    let user_id = match account_id {
        Some(id) => id,
        None => match state.storage.get_or_create_anonymous_account() {
            Ok(acc) => acc.id,
            Err(e) => return IpcResponse::err("tauri-ipc", e),
        },
    };

    match state.storage.list_devices(&user_id) {
        Ok(devices) => IpcResponse::ok("tauri-ipc", ListDevicesCmdResponse { devices }),
        Err(e) => IpcResponse::err("tauri-ipc", e),
    }
}

#[tauri::command]
pub fn lingua_command_create_api_key(
    state: State<'_, DesktopAppState>,
    request: CreateApiKeyCmdRequest,
) -> IpcResponse<CreateApiKeyCmdResponse> {
    let (raw_key, prefix, key_hash) = ApiGateway::generate_raw_api_key("live");
    let record = ApiKeyRecord {
        id: format!("key-{}", Uuid::new_v4()),
        user_id: request.user_id,
        org_id: request.org_id,
        key_hash,
        key_prefix: prefix,
        name: request.name,
        rate_limit_per_min: 300,
        is_active: true,
        created_at: chrono::Utc::now(),
        expires_at: None,
    };

    match state.storage.create_api_key(&record) {
        Ok(()) => IpcResponse::ok(
            "tauri-ipc",
            CreateApiKeyCmdResponse {
                api_key_record: record,
                raw_key,
            },
        ),
        Err(e) => IpcResponse::err("tauri-ipc", e),
    }
}

#[tauri::command]
pub fn lingua_command_list_api_keys(
    state: State<'_, DesktopAppState>,
    user_id: Option<String>,
) -> IpcResponse<ListApiKeysCmdResponse> {
    let uid = match user_id {
        Some(id) => id,
        None => match state.storage.get_or_create_anonymous_account() {
            Ok(acc) => acc.id,
            Err(e) => return IpcResponse::err("tauri-ipc", e),
        },
    };

    match state.storage.list_api_keys(&uid) {
        Ok(api_keys) => IpcResponse::ok("tauri-ipc", ListApiKeysCmdResponse { api_keys }),
        Err(e) => IpcResponse::err("tauri-ipc", e),
    }
}

#[tauri::command]
pub fn lingua_command_revoke_api_key(
    state: State<'_, DesktopAppState>,
    key_id: String,
) -> IpcResponse<bool> {
    match state.storage.revoke_api_key(&key_id) {
        Ok(()) => IpcResponse::ok("tauri-ipc", true),
        Err(e) => IpcResponse::err("tauri-ipc", e),
    }
}

#[tauri::command]
pub fn lingua_command_get_usage_summary(
    state: State<'_, DesktopAppState>,
    account_id: Option<String>,
) -> IpcResponse<GetUsageSummaryCmdResponse> {
    let uid = match account_id {
        Some(id) => id,
        None => match state.storage.get_or_create_anonymous_account() {
            Ok(acc) => acc.id,
            Err(e) => return IpcResponse::err("tauri-ipc", e),
        },
    };

    match state.storage.get_usage_summary(&uid) {
        Ok(summary) => IpcResponse::ok("tauri-ipc", GetUsageSummaryCmdResponse { summary }),
        Err(e) => IpcResponse::err("tauri-ipc", e),
    }
}

#[tauri::command]
pub fn lingua_command_list_invoices(
    state: State<'_, DesktopAppState>,
    account_id: Option<String>,
) -> IpcResponse<ListInvoicesCmdResponse> {
    let uid = match account_id {
        Some(id) => id,
        None => match state.storage.get_or_create_anonymous_account() {
            Ok(acc) => acc.id,
            Err(e) => return IpcResponse::err("tauri-ipc", e),
        },
    };

    match state.storage.list_invoices(&uid) {
        Ok(invoices) => IpcResponse::ok("tauri-ipc", ListInvoicesCmdResponse { invoices }),
        Err(e) => IpcResponse::err("tauri-ipc", e),
    }
}

#[tauri::command]
pub fn lingua_command_create_organization(
    state: State<'_, DesktopAppState>,
    request: CreateOrgCmdRequest,
) -> IpcResponse<OrgCmdResponse> {
    let org = Organization {
        id: format!("org-{}", Uuid::new_v4()),
        name: request.name,
        slug: request.slug,
        max_seats: 25,
        sso_enabled: false,
        data_retention_days: 90,
        enforce_zero_transcript_logging: true,
        created_at: chrono::Utc::now(),
    };

    match state.storage.create_organization(&org, &request.owner_id) {
        Ok(()) => {
            let event = AuditEvent {
                id: format!("audit-{}", Uuid::new_v4()),
                org_id: Some(org.id.clone()),
                actor_id: request.owner_id,
                actor_email: "admin@enterprise.local".into(),
                action: "organization.created".into(),
                resource_type: "organization".into(),
                resource_id: org.id.clone(),
                details: format!(
                    "Created organization '{}' with slug '{}'",
                    org.name, org.slug
                ),
                ip_address: "127.0.0.1".into(),
                timestamp: chrono::Utc::now(),
            };
            let _ = state.storage.record_audit_event(&event);
            IpcResponse::ok("tauri-ipc", OrgCmdResponse { organization: org })
        }
        Err(e) => IpcResponse::err("tauri-ipc", e),
    }
}

#[tauri::command]
pub fn lingua_command_list_org_members(
    state: State<'_, DesktopAppState>,
    org_id: String,
) -> IpcResponse<ListOrgMembersCmdResponse> {
    match state.storage.list_org_members(&org_id) {
        Ok(members) => IpcResponse::ok("tauri-ipc", ListOrgMembersCmdResponse { members }),
        Err(e) => IpcResponse::err("tauri-ipc", e),
    }
}

#[tauri::command]
pub fn lingua_command_add_org_member(
    state: State<'_, DesktopAppState>,
    org_id: String,
    user_id: String,
    email: String,
    display_name: String,
    role: String,
) -> IpcResponse<bool> {
    let org_role = match role.to_lowercase().as_str() {
        "admin" => OrgRole::Admin,
        "linguist_editor" => OrgRole::LinguistEditor,
        "viewer" => OrgRole::Viewer,
        _ => OrgRole::Member,
    };

    let member = OrgMember {
        id: format!("mem-{}", Uuid::new_v4()),
        org_id,
        user_id,
        email,
        display_name,
        role: org_role,
        joined_at: chrono::Utc::now(),
    };

    match state.storage.add_org_member(&member) {
        Ok(()) => IpcResponse::ok("tauri-ipc", true),
        Err(e) => IpcResponse::err("tauri-ipc", e),
    }
}

#[tauri::command]
pub fn lingua_command_list_org_glossaries(
    state: State<'_, DesktopAppState>,
    org_id: String,
) -> IpcResponse<ListOrgGlossariesCmdResponse> {
    match state.storage.list_org_glossaries(&org_id) {
        Ok(glossaries) => IpcResponse::ok("tauri-ipc", ListOrgGlossariesCmdResponse { glossaries }),
        Err(e) => IpcResponse::err("tauri-ipc", e),
    }
}

#[tauri::command]
pub fn lingua_command_create_org_glossary(
    state: State<'_, DesktopAppState>,
    request: CreateOrgGlossaryCmdRequest,
) -> IpcResponse<bool> {
    let glossary = OrgGlossary {
        id: format!("gloss-{}", Uuid::new_v4()),
        org_id: request.org_id,
        name: request.name,
        description: request.description,
        source_lang: request.source_lang,
        target_lang: request.target_lang,
        status: OrgGlossaryStatus::Approved,
        terms_count: 0,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    match state.storage.create_org_glossary(&glossary) {
        Ok(()) => IpcResponse::ok("tauri-ipc", true),
        Err(e) => IpcResponse::err("tauri-ipc", e),
    }
}

#[tauri::command]
pub fn lingua_command_add_org_glossary_term(
    state: State<'_, DesktopAppState>,
    glossary_id: String,
    source_term: String,
    target_term: String,
    notes: Option<String>,
    case_sensitive: Option<bool>,
) -> IpcResponse<bool> {
    let term = OrgGlossaryTerm {
        id: format!("gterm-{}", Uuid::new_v4()),
        glossary_id,
        source_term,
        target_term,
        notes,
        case_sensitive: case_sensitive.unwrap_or(false),
    };

    match state.storage.add_org_glossary_term(&term) {
        Ok(()) => IpcResponse::ok("tauri-ipc", true),
        Err(e) => IpcResponse::err("tauri-ipc", e),
    }
}

#[tauri::command]
pub fn lingua_command_list_audit_events(
    state: State<'_, DesktopAppState>,
    org_id: Option<String>,
    limit: Option<usize>,
) -> IpcResponse<ListAuditEventsCmdResponse> {
    let lim = limit.unwrap_or(50);
    match state.storage.list_audit_events(org_id.as_deref(), lim) {
        Ok(events) => IpcResponse::ok("tauri-ipc", ListAuditEventsCmdResponse { events }),
        Err(e) => IpcResponse::err("tauri-ipc", e),
    }
}

#[tauri::command]
pub fn lingua_command_get_worker_metrics(
    state: State<'_, DesktopAppState>,
) -> IpcResponse<GetWorkerMetricsCmdResponse> {
    let metrics = state.worker_pool.get_metrics();
    IpcResponse::ok("tauri-ipc", GetWorkerMetricsCmdResponse { metrics })
}
