import type {
  AppInfo,
  CapabilityMatrix,
  CircuitBreakerStatus,
  CreateGlossaryTermCmdRequest,
  DatabaseHealthInfo,
  DomainGlossaryEntry,
  EvaluationReport,
  IntelligenceTranslationRequest,
  IntelligenceTranslationResponse,
  IpcResponse,
  LanguageCode,
  ModelPackInfo,
  ResourceProfile,
  TransliterationResponse,
  UserSettings,
  MediaSession,
  MediaSubtitleSegment,
  MediaSubtitleExportFormat,
  OverlaySettings,
  SystemAudioPermission,
  OcrResult,
  DocumentFormat,
  DocumentTranslationJob,
  DocumentBlock,
  GlobalShortcutConfig,
  LearningDeck,
  LearningCard,
  ReviewResult,
  LearnerProgress,
  PronunciationAssessmentRequest,
  PronunciationAssessmentResult,
  ListeningChallenge,
  GrammarGuideTopic,
  DataPrivacySettings,
  ExportUserDataArchive,
  PurgeUserDataResult,
  PlatformReleaseStatus,
  UserAccount,
  AccountTier,
  DeviceRegistration,
  DeviceType,
  ApiKeyRecord,
  Organization,
  OrgMember,
  OrgGlossary,
  OrgGlossaryTerm,
  UsageSummary,
  BillingInvoiceRecord,
  AuditEvent,
  WorkerMetrics,
} from './types';

// Check if running in a Tauri native environment
const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

async function invokeTauri<T>(cmd: string, args: Record<string, unknown> = {}): Promise<T> {
  const { invoke } = await import('@tauri-apps/api/core');
  const res = (await invoke(cmd, args)) as IpcResponse<T>;
  if (!res.success || !res.data) {
    throw new Error(res.error?.message || `Failed IPC command: ${cmd}`);
  }
  return res.data;
}

const API_BASE = typeof window !== 'undefined' && window.location.port === '5173'
  ? 'http://127.0.0.1:8080/api/v1'
  : '/api/v1';

export async function getAppInfo(): Promise<AppInfo> {
  if (isTauri) {
    return invokeTauri<AppInfo>('lingua_command_get_app_info');
  }
  const res = await fetch(`${API_BASE}/app-info`);
  if (!res.ok) throw new Error('Failed to fetch app info');
  return res.json();
}

export async function getCapabilities(): Promise<CapabilityMatrix> {
  if (isTauri) {
    return invokeTauri<CapabilityMatrix>('lingua_command_get_capabilities');
  }
  const res = await fetch(`${API_BASE}/capabilities`);
  if (!res.ok) throw new Error('Failed to fetch capabilities');
  return res.json();
}

export async function getUserSettings(): Promise<UserSettings> {
  if (isTauri) {
    return invokeTauri<UserSettings>('lingua_command_get_user_settings');
  }
  const res = await fetch(`${API_BASE}/settings`);
  if (!res.ok) throw new Error('Failed to fetch settings');
  return res.json();
}

export async function saveUserSettings(settings: UserSettings): Promise<UserSettings> {
  if (isTauri) {
    return invokeTauri<UserSettings>('lingua_command_save_user_settings', { settings });
  }
  const res = await fetch(`${API_BASE}/settings`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(settings),
  });
  if (!res.ok) throw new Error('Failed to save settings');
  return settings;
}

export async function listModelPacks(): Promise<ModelPackInfo[]> {
  if (isTauri) {
    return invokeTauri<ModelPackInfo[]>('lingua_command_list_model_packs');
  }
  const res = await fetch(`${API_BASE}/model-packs`);
  if (!res.ok) throw new Error('Failed to fetch model packs');
  return res.json();
}

export async function checkDatabaseHealth(): Promise<DatabaseHealthInfo> {
  if (isTauri) {
    return invokeTauri<DatabaseHealthInfo>('lingua_command_check_database_health');
  }
  const res = await fetch(`${API_BASE}/health`);
  if (!res.ok) throw new Error('Failed to check database health');
  return res.json();
}

export async function translateText(req: {
  source_language: string;
  target_language: string;
  text: string;
  allow_cloud: boolean;
}): Promise<import('./types').TranslateCmdResponse> {
  if (isTauri) {
    return invokeTauri<import('./types').TranslateCmdResponse>('lingua_command_translate', { request: req });
  }
  const res = await fetch(`${API_BASE}/translate`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(req),
  });
  if (!res.ok) {
    const errText = await res.text();
    throw new Error(errText || 'Translation failed');
  }
  return res.json();
}

export async function searchDictionary(
  query: string,
  language?: string | null,
  limit: number = 20,
  partOfSpeech?: string | null
): Promise<import('./types').DictionarySearchCmdResponse> {
  if (isTauri) {
    return invokeTauri<import('./types').DictionarySearchCmdResponse>('lingua_command_dictionary_search', {
      request: { query, language, limit, part_of_speech: partOfSpeech },
    });
  }
  const params = new URLSearchParams({ q: query, limit: limit.toString() });
  if (language) params.append('lang', language);
  if (partOfSpeech && partOfSpeech !== 'all') params.append('pos', partOfSpeech);
  const res = await fetch(`${API_BASE}/dictionary/search?${params}`);
  if (!res.ok) throw new Error('Dictionary search failed');
  return res.json();
}

export async function getDictionaryWord(id: string): Promise<import('./types').DictionaryEntry | null> {
  if (isTauri) {
    return invokeTauri<import('./types').DictionaryEntry | null>('lingua_command_get_dictionary_word', { id });
  }
  const res = await fetch(`${API_BASE}/dictionary/word/${encodeURIComponent(id)}`);
  if (!res.ok) return null;
  return res.json();
}

export async function detectLanguage(text: string): Promise<string | null> {
  if (isTauri) {
    const res = await invokeTauri<{ detected: string | null }>('lingua_command_detect_language', { request: { text } });
    return res.detected;
  }
  const res = await fetch(`${API_BASE}/detect-language`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ text }),
  });
  if (!res.ok) return null;
  const data = await res.json();
  return data.detected;
}

export async function listTranslationHistory(limit: number = 50): Promise<import('./types').TranslationRecord[]> {
  if (isTauri) {
    return invokeTauri<import('./types').TranslationRecord[]>('lingua_command_list_translation_history', { limit });
  }
  const res = await fetch(`${API_BASE}/history?limit=${limit}`);
  if (!res.ok) throw new Error('Failed to load history');
  return res.json();
}

export async function clearTranslationHistory(): Promise<void> {
  if (isTauri) {
    return invokeTauri<void>('lingua_command_clear_translation_history');
  }
  const res = await fetch(`${API_BASE}/history`, { method: 'DELETE' });
  if (!res.ok) throw new Error('Failed to clear history');
}

export async function listFavorites(): Promise<import('./types').FavoriteItem[]> {
  if (isTauri) {
    return invokeTauri<import('./types').FavoriteItem[]>('lingua_command_list_favorites');
  }
  const res = await fetch(`${API_BASE}/favorites`);
  if (!res.ok) throw new Error('Failed to load favorites');
  return res.json();
}

export async function addFavorite(req: import('./types').AddFavoriteCmdRequest): Promise<import('./types').FavoriteItem> {
  if (isTauri) {
    return invokeTauri<import('./types').FavoriteItem>('lingua_command_add_favorite', { request: req });
  }
  const res = await fetch(`${API_BASE}/favorites`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(req),
  });
  if (!res.ok) throw new Error('Failed to add favorite');
  return res.json();
}

export async function removeFavorite(id: string): Promise<void> {
  if (isTauri) {
    return invokeTauri<void>('lingua_command_remove_favorite', { id });
  }
  const res = await fetch(`${API_BASE}/favorites/${encodeURIComponent(id)}`, { method: 'DELETE' });
  if (!res.ok) throw new Error('Failed to remove favorite');
}

export async function listDetailedModelPacks(): Promise<import('./types').ModelPackDetailedInfo[]> {
  if (isTauri) {
    return invokeTauri<import('./types').ModelPackDetailedInfo[]>('lingua_command_list_detailed_model_packs');
  }
  const res = await fetch(`${API_BASE}/models`);
  if (!res.ok) throw new Error('Failed to load detailed model packs');
  return res.json();
}

export async function getDetailedModelPack(id: string): Promise<import('./types').ModelPackDetailedInfo | null> {
  if (isTauri) {
    return invokeTauri<import('./types').ModelPackDetailedInfo | null>('lingua_command_get_detailed_model_pack', { id });
  }
  const res = await fetch(`${API_BASE}/models/${encodeURIComponent(id)}`);
  if (!res.ok) throw new Error('Failed to get model pack');
  return res.json();
}

export async function downloadModelPack(modelId: string): Promise<string> {
  if (isTauri) {
    return invokeTauri<string>('lingua_command_download_model_pack', { request: { model_id: modelId } });
  }
  const res = await fetch(`${API_BASE}/models/download/${encodeURIComponent(modelId)}`, { method: 'POST' });
  if (!res.ok) {
    const errText = await res.text();
    throw new Error(errText || 'Failed to download model pack');
  }
  return res.json();
}

export async function verifyModelPack(modelId: string): Promise<import('./types').VerifyModelCmdResponse> {
  if (isTauri) {
    return invokeTauri<import('./types').VerifyModelCmdResponse>('lingua_command_verify_model_pack', { request: { model_id: modelId } });
  }
  const res = await fetch(`${API_BASE}/models/verify/${encodeURIComponent(modelId)}`, { method: 'POST' });
  if (!res.ok) {
    const errText = await res.text();
    throw new Error(errText || 'Failed to verify model pack');
  }
  return res.json();
}

export async function deleteModelPack(modelId: string): Promise<void> {
  if (isTauri) {
    return invokeTauri<void>('lingua_command_delete_model_pack', { request: { model_id: modelId } });
  }
  const res = await fetch(`${API_BASE}/models/${encodeURIComponent(modelId)}`, { method: 'DELETE' });
  if (!res.ok) {
    const errText = await res.text();
    throw new Error(errText || 'Failed to delete model pack');
  }
}

export async function getDiskSpace(): Promise<import('./types').GetDiskSpaceCmdResponse> {
  if (isTauri) {
    return invokeTauri<import('./types').GetDiskSpaceCmdResponse>('lingua_command_get_disk_space');
  }
  const res = await fetch(`${API_BASE}/models/disk-space`);
  if (!res.ok) throw new Error('Failed to get disk space');
  return res.json();
}

export async function getAudioDevices(): Promise<import('./types').AudioDeviceItem[]> {
  if (isTauri) {
    const resp = await invokeTauri<import('./types').GetAudioDevicesCmdResponse>('lingua_command_get_audio_devices');
    return resp.devices;
  }
  const res = await fetch(`${API_BASE}/speech/devices`);
  if (!res.ok) throw new Error('Failed to fetch audio devices');
  const data: import('./types').GetAudioDevicesCmdResponse = await res.json();
  return data.devices;
}

export async function startSpeechSession(req: import('./types').StartSpeechSessionCmdRequest): Promise<import('./types').SpeechSession> {
  if (isTauri) {
    const resp = await invokeTauri<import('./types').StartSpeechSessionCmdResponse>('lingua_command_start_speech_session', { request: req });
    return resp.session;
  }
  const res = await fetch(`${API_BASE}/speech/session/start`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(req),
  });
  if (!res.ok) {
    const errText = await res.text();
    throw new Error(errText || 'Failed to start speech session');
  }
  const data: import('./types').StartSpeechSessionCmdResponse = await res.json();
  return data.session;
}

export async function pauseSpeechSession(sessionId: string): Promise<import('./types').SpeechSession> {
  if (isTauri) {
    return invokeTauri<import('./types').SpeechSession>('lingua_command_pause_speech_session', { request: { session_id: sessionId } });
  }
  const res = await fetch(`${API_BASE}/speech/session/pause`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ session_id: sessionId }),
  });
  if (!res.ok) throw new Error('Failed to pause speech session');
  return res.json();
}

export async function resumeSpeechSession(sessionId: string): Promise<import('./types').SpeechSession> {
  if (isTauri) {
    return invokeTauri<import('./types').SpeechSession>('lingua_command_resume_speech_session', { request: { session_id: sessionId } });
  }
  const res = await fetch(`${API_BASE}/speech/session/resume`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ session_id: sessionId }),
  });
  if (!res.ok) throw new Error('Failed to resume speech session');
  return res.json();
}

export async function stopSpeechSession(sessionId: string): Promise<import('./types').SpeechSession> {
  if (isTauri) {
    const resp = await invokeTauri<import('./types').StopSpeechSessionCmdResponse>('lingua_command_stop_speech_session', { request: { session_id: sessionId } });
    return resp.session;
  }
  const res = await fetch(`${API_BASE}/speech/session/stop`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ session_id: sessionId }),
  });
  if (!res.ok) throw new Error('Failed to stop speech session');
  const data: import('./types').StopSpeechSessionCmdResponse = await res.json();
  return data.session;
}

export async function getSessionCaptions(sessionId: string): Promise<import('./types').CaptionSegment[]> {
  if (isTauri) {
    const resp = await invokeTauri<import('./types').GetSessionCaptionsCmdResponse>('lingua_command_get_session_captions', { request: { session_id: sessionId } });
    return resp.captions;
  }
  const res = await fetch(`${API_BASE}/speech/session/${encodeURIComponent(sessionId)}/captions`);
  if (!res.ok) throw new Error('Failed to fetch session captions');
  const data: import('./types').GetSessionCaptionsCmdResponse = await res.json();
  return data.captions;
}

export async function exportCaptions(sessionId: string, format: import('./types').CaptionFormat): Promise<import('./types').ExportCaptionsCmdResponse> {
  if (isTauri) {
    return invokeTauri<import('./types').ExportCaptionsCmdResponse>('lingua_command_export_captions', { request: { session_id: sessionId, format } });
  }
  const res = await fetch(`${API_BASE}/speech/session/${encodeURIComponent(sessionId)}/export?format=${format}`);
  if (!res.ok) throw new Error('Failed to export captions');
  return res.json();
}

export async function feedAudioChunk(sessionId: string, samples: number[]): Promise<import('./types').CaptionSegment[]> {
  if (isTauri) {
    return invokeTauri<import('./types').CaptionSegment[]>('lingua_command_feed_audio_chunk', { request: { session_id: sessionId, samples } });
  }
  const res = await fetch(`${API_BASE}/speech/feed-chunk`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ session_id: sessionId, samples }),
  });
  if (!res.ok) throw new Error('Failed to feed audio chunk');
  return res.json();
}

// Stage 4: Real-Time Voice Conversation & TTS IPC Functions

export async function startConversation(
  speakerALang: import('./types').LanguageCode,
  speakerBLang: import('./types').LanguageCode,
  title?: string | null,
  isHandsFree = false
): Promise<import('./types').ConversationSession> {
  if (isTauri) {
    return invokeTauri<import('./types').ConversationSession>('lingua_command_start_conversation', {
      request: {
        speaker_a_lang: speakerALang,
        speaker_b_lang: speakerBLang,
        title,
        is_hands_free: isHandsFree,
      },
    });
  }
  const res = await fetch(`${API_BASE}/conversation/start`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      speaker_a_lang: speakerALang,
      speaker_b_lang: speakerBLang,
      title,
      is_hands_free: isHandsFree,
    }),
  });
  if (!res.ok) throw new Error('Failed to start conversation');
  const data = await res.json();
  return data.session;
}

export async function submitConversationTurn(
  sessionId: string,
  speakerId: import('./types').SpeakerId,
  sourceText: string
): Promise<import('./types').SubmitTurnCmdResponse> {
  if (isTauri) {
    return invokeTauri<import('./types').SubmitTurnCmdResponse>('lingua_command_submit_turn', {
      request: {
        session_id: sessionId,
        speaker_id: speakerId,
        source_text: sourceText,
      },
    });
  }
  const res = await fetch(`${API_BASE}/conversation/turn`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      session_id: sessionId,
      speaker_id: speakerId,
      source_text: sourceText,
    }),
  });
  if (!res.ok) throw new Error('Failed to submit conversation turn');
  return res.json();
}

export async function switchConversationSpeaker(
  sessionId: string,
  nextSpeaker?: import('./types').SpeakerId | null
): Promise<import('./types').ConversationSession> {
  if (isTauri) {
    return invokeTauri<import('./types').ConversationSession>('lingua_command_switch_speaker', {
      request: { session_id: sessionId, next_speaker: nextSpeaker },
    });
  }
  const res = await fetch(`${API_BASE}/conversation/switch-speaker`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ session_id: sessionId, next_speaker: nextSpeaker }),
  });
  if (!res.ok) throw new Error('Failed to switch speaker');
  const data = await res.json();
  return data.session;
}

export async function interruptConversation(sessionId: string): Promise<boolean> {
  if (isTauri) {
    const res = await invokeTauri<{ session_id: string; success: boolean }>('lingua_command_interrupt_conversation', {
      request: { session_id: sessionId },
    });
    return res.success;
  }
  const res = await fetch(`${API_BASE}/conversation/interrupt`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ session_id: sessionId }),
  });
  if (!res.ok) throw new Error('Failed to interrupt conversation');
  const data = await res.json();
  return data.success;
}

export async function stopConversation(sessionId: string): Promise<import('./types').ConversationSession> {
  if (isTauri) {
    return invokeTauri<import('./types').ConversationSession>('lingua_command_stop_conversation', {
      request: { session_id: sessionId },
    });
  }
  const res = await fetch(`${API_BASE}/conversation/stop`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ session_id: sessionId }),
  });
  if (!res.ok) throw new Error('Failed to stop conversation');
  const data = await res.json();
  return data.session;
}

export async function getConversationTurns(sessionId: string): Promise<import('./types').ConversationTurnItem[]> {
  if (isTauri) {
    return invokeTauri<import('./types').ConversationTurnItem[]>('lingua_command_get_conversation_turns', {
      request: { session_id: sessionId },
    });
  }
  const res = await fetch(`${API_BASE}/conversation/${encodeURIComponent(sessionId)}/turns`);
  if (!res.ok) throw new Error('Failed to fetch conversation turns');
  const data = await res.json();
  return data.turns;
}

export async function listTtsVoices(): Promise<import('./types').TtsVoiceInfo[]> {
  if (isTauri) {
    return invokeTauri<import('./types').TtsVoiceInfo[]>('lingua_command_list_tts_voices');
  }
  const res = await fetch(`${API_BASE}/tts/voices`);
  if (!res.ok) throw new Error('Failed to fetch TTS voices');
  const data = await res.json();
  return data.voices;
}

export async function synthesizeSpeech(
  req: import('./types').TtsSynthesizeRequest
): Promise<import('./types').TtsSynthesizeResponse> {
  if (isTauri) {
    return invokeTauri<import('./types').TtsSynthesizeResponse>('lingua_command_synthesize_speech', {
      request: req,
    });
  }
  const res = await fetch(`${API_BASE}/tts/synthesize`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(req),
  });
  if (!res.ok) throw new Error('Failed to synthesize speech');
  const data = await res.json();
  return data.response;
}

// ==========================================
// Stage 5: Translation Intelligence & Optimization
// ==========================================

export async function translateWithIntelligence(
  req: IntelligenceTranslationRequest
): Promise<IntelligenceTranslationResponse> {
  if (isTauri) {
    return invokeTauri<IntelligenceTranslationResponse>('lingua_command_translate_intelligence', {
      request: req,
    });
  }
  const res = await fetch(`${API_BASE}/intelligence/translate`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(req),
  });
  if (!res.ok) {
    const err = await res.text();
    throw new Error(err || 'Failed to translate with intelligence');
  }
  return res.json();
}

export async function transliterateText(
  text: string,
  target_lang: LanguageCode
): Promise<TransliterationResponse> {
  if (isTauri) {
    return invokeTauri<TransliterationResponse>('lingua_command_transliterate', {
      request: { text, target_lang },
    });
  }
  const res = await fetch(`${API_BASE}/intelligence/transliterate`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ text, target_lang }),
  });
  if (!res.ok) {
    const err = await res.text();
    throw new Error(err || 'Failed to transliterate text');
  }
  return res.json();
}

export async function getCircuitBreakers(): Promise<CircuitBreakerStatus[]> {
  if (isTauri) {
    const resp = await invokeTauri<{ circuit_breakers: CircuitBreakerStatus[] }>('lingua_command_get_circuit_breakers');
    return resp.circuit_breakers;
  }
  const res = await fetch(`${API_BASE}/intelligence/circuit-breakers`);
  if (!res.ok) throw new Error('Failed to fetch circuit breakers');
  const data = await res.json();
  return data.circuit_breakers;
}

export async function resetCircuitBreaker(providerId: string): Promise<boolean> {
  if (isTauri) {
    return invokeTauri<boolean>('lingua_command_reset_circuit_breaker', {
      request: { provider_id: providerId },
    });
  }
  const res = await fetch(`${API_BASE}/intelligence/circuit-breakers/${encodeURIComponent(providerId)}/reset`, {
    method: 'POST',
  });
  return res.ok;
}

export async function listGlossaryTerms(
  domain?: string,
  sourceLang?: LanguageCode,
  targetLang?: LanguageCode
): Promise<DomainGlossaryEntry[]> {
  if (isTauri) {
    const resp = await invokeTauri<{ terms: DomainGlossaryEntry[] }>('lingua_command_list_glossary_terms', {
      request: {
        domain: domain || null,
        source_lang: sourceLang || null,
        target_lang: targetLang || null,
      },
    });
    return resp.terms;
  }
  const params = new URLSearchParams();
  if (domain) params.set('domain', domain);
  if (sourceLang) params.set('source_lang', sourceLang);
  if (targetLang) params.set('target_lang', targetLang);

  const res = await fetch(`${API_BASE}/glossary?${params.toString()}`);
  if (!res.ok) throw new Error('Failed to fetch glossary terms');
  const data = await res.json();
  return data.terms;
}

export async function createGlossaryTerm(
  req: CreateGlossaryTermCmdRequest
): Promise<DomainGlossaryEntry> {
  if (isTauri) {
    return invokeTauri<DomainGlossaryEntry>('lingua_command_create_glossary_term', {
      request: req,
    });
  }
  const res = await fetch(`${API_BASE}/glossary`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(req),
  });
  if (!res.ok) throw new Error('Failed to create glossary term');
  return res.json();
}

export async function deleteGlossaryTerm(id: string): Promise<boolean> {
  if (isTauri) {
    return invokeTauri<boolean>('lingua_command_delete_glossary_term', {
      request: { id },
    });
  }
  const res = await fetch(`${API_BASE}/glossary/${encodeURIComponent(id)}`, {
    method: 'DELETE',
  });
  return res.ok;
}

export async function runEvaluation(): Promise<EvaluationReport> {
  if (isTauri) {
    const resp = await invokeTauri<{ report: EvaluationReport }>('lingua_command_run_evaluation');
    return resp.report;
  }
  const res = await fetch(`${API_BASE}/intelligence/evaluate`, {
    method: 'POST',
  });
  if (!res.ok) throw new Error('Failed to run quality evaluation');
  const data = await res.json();
  return data.report;
}

export async function getLatestEvaluationReport(): Promise<EvaluationReport | null> {
  if (isTauri) {
    return invokeTauri<EvaluationReport | null>('lingua_command_get_latest_evaluation_report');
  }
  const res = await fetch(`${API_BASE}/intelligence/evaluation-report`);
  if (!res.ok) throw new Error('Failed to fetch latest evaluation report');
  return res.json();
}

export async function setResourceProfile(profile: ResourceProfile): Promise<boolean> {
  if (isTauri) {
    return invokeTauri<boolean>('lingua_command_set_resource_profile', {
      request: { profile },
    });
  }
  const res = await fetch(`${API_BASE}/intelligence/resource-profile`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ profile }),
  });
  return res.ok;
}

// Stage 6 IPC Wrappers: Media, Subtitles, OCR, Document & Overlay

export async function startMediaTranscription(req: {
  file_name: string;
  file_bytes: number[];
  source_lang: LanguageCode;
  target_lang: LanguageCode;
  title?: string;
}): Promise<{ session: MediaSession; segments: MediaSubtitleSegment[] }> {
  if (isTauri) {
    return invokeTauri<{ session: MediaSession; segments: MediaSubtitleSegment[] }>(
      'lingua_command_start_media_transcription',
      { request: req }
    );
  }
  const res = await fetch(`${API_BASE}/media/transcribe`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(req),
  });
  if (!res.ok) throw new Error('Failed to transcribe media file');
  return res.json();
}

export async function listMediaSegments(sessionId: string): Promise<MediaSubtitleSegment[]> {
  if (isTauri) {
    const res = await invokeTauri<{ segments: MediaSubtitleSegment[] }>(
      'lingua_command_list_media_segments',
      { request: { session_id: sessionId } }
    );
    return res.segments;
  }
  const res = await fetch(`${API_BASE}/media/${sessionId}/segments`);
  if (!res.ok) throw new Error('Failed to list media segments');
  const data = await res.json();
  return data.segments;
}

export async function updateMediaSegment(segment: {
  segment_id: string;
  source_text: string;
  translated_text: string;
  start_ms: number;
  end_ms: number;
}): Promise<boolean> {
  if (isTauri) {
    return invokeTauri<boolean>('lingua_command_update_media_segment', { request: segment });
  }
  const res = await fetch(`${API_BASE}/media/segments/${segment.segment_id}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(segment),
  });
  return res.ok;
}

export async function exportSubtitles(
  sessionId: string,
  format: MediaSubtitleExportFormat,
  targetOnly: boolean = false
): Promise<{ content: string; file_name: string }> {
  if (isTauri) {
    return invokeTauri<{ content: string; file_name: string }>(
      'lingua_command_export_subtitles',
      { request: { session_id: sessionId, format, target_only: targetOnly } }
    );
  }
  const res = await fetch(`${API_BASE}/media/${sessionId}/export?format=${format}&target_only=${targetOnly}`);
  if (!res.ok) throw new Error('Failed to export subtitles');
  return res.json();
}

export async function performOcr(req: {
  image_bytes: number[];
  mime_type: string;
  source_lang?: LanguageCode | null;
  target_lang?: LanguageCode | null;
}): Promise<OcrResult> {
  if (isTauri) {
    const res = await invokeTauri<{ result: OcrResult }>('lingua_command_perform_ocr', { request: req });
    return res.result;
  }
  const res = await fetch(`${API_BASE}/ocr/scan`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(req),
  });
  if (!res.ok) throw new Error('Failed to perform OCR scan');
  const data = await res.json();
  return data.result;
}

export async function translateDocument(req: {
  file_name: string;
  content: number[];
  format: DocumentFormat;
  source_lang: LanguageCode;
  target_lang: LanguageCode;
}): Promise<{ job: DocumentTranslationJob; blocks: DocumentBlock[]; exported_content: string }> {
  if (isTauri) {
    return invokeTauri<{ job: DocumentTranslationJob; blocks: DocumentBlock[]; exported_content: string }>(
      'lingua_command_translate_document',
      { request: req }
    );
  }
  const res = await fetch(`${API_BASE}/document/translate`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(req),
  });
  if (!res.ok) throw new Error('Failed to translate document');
  return res.json();
}

export async function getOverlaySettings(): Promise<OverlaySettings> {
  if (isTauri) {
    const res = await invokeTauri<{ settings: OverlaySettings }>('lingua_command_get_overlay_settings');
    return res.settings;
  }
  const res = await fetch(`${API_BASE}/overlay/settings`);
  if (!res.ok) throw new Error('Failed to fetch overlay settings');
  const data = await res.json();
  return data.settings;
}

export async function saveOverlaySettings(settings: OverlaySettings): Promise<boolean> {
  if (isTauri) {
    return invokeTauri<boolean>('lingua_command_save_overlay_settings', { request: { settings } });
  }
  const res = await fetch(`${API_BASE}/overlay/settings`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ settings }),
  });
  return res.ok;
}

export async function getOverlayPermission(): Promise<SystemAudioPermission> {
  if (isTauri) {
    return invokeTauri<SystemAudioPermission>('lingua_command_get_overlay_permission');
  }
  const res = await fetch(`${API_BASE}/overlay/permission`);
  if (!res.ok) return 'granted';
  return res.json();
}

export async function getGlobalShortcuts(): Promise<GlobalShortcutConfig> {
  if (isTauri) {
    const res = await invokeTauri<{ config: GlobalShortcutConfig }>('lingua_command_get_global_shortcuts');
    return res.config;
  }
  return {
    quick_translate: 'CommandOrControl+Shift+L',
    overlay_toggle: 'CommandOrControl+Shift+C',
    enabled: true,
  };
}

export async function saveGlobalShortcuts(config: GlobalShortcutConfig): Promise<boolean> {
  if (isTauri) {
    return invokeTauri<boolean>('lingua_command_save_global_shortcuts', { request: { config } });
  }
  return true;
}

// ============================================================================
// Stage 7: Learning & Production Certification IPC Handlers
// ============================================================================

export async function listLearningDecks(): Promise<LearningDeck[]> {
  if (isTauri) {
    const res = await invokeTauri<{ decks: LearningDeck[] }>('lingua_command_list_learning_decks');
    return res.decks;
  }
  const res = await fetch(`${API_BASE}/learning/decks`);
  if (!res.ok) throw new Error('Failed to fetch learning decks');
  const data = await res.json();
  return data.decks;
}

export async function getDeckCards(deckId: string): Promise<LearningCard[]> {
  if (isTauri) {
    const res = await invokeTauri<{ cards: LearningCard[] }>('lingua_command_get_deck_cards', {
      request: { deck_id: deckId },
    });
    return res.cards;
  }
  const res = await fetch(`${API_BASE}/learning/deck/${deckId}/cards`);
  if (!res.ok) throw new Error('Failed to fetch deck cards');
  const data = await res.json();
  return data.cards;
}

export async function reviewCard(cardId: string, rating: number): Promise<ReviewResult> {
  if (isTauri) {
    const res = await invokeTauri<{ result: ReviewResult }>('lingua_command_review_card', {
      request: { card_id: cardId, rating },
    });
    return res.result;
  }
  const res = await fetch(`${API_BASE}/learning/card/${cardId}/review`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ rating }),
  });
  if (!res.ok) throw new Error('Failed to log review');
  const data = await res.json();
  return data.result;
}

export async function getLearnerProgress(): Promise<LearnerProgress> {
  if (isTauri) {
    const res = await invokeTauri<{ progress: LearnerProgress }>('lingua_command_get_learner_progress');
    return res.progress;
  }
  const res = await fetch(`${API_BASE}/learning/progress`);
  if (!res.ok) throw new Error('Failed to fetch learner progress');
  const data = await res.json();
  return data.progress;
}

export async function assessPronunciation(
  req: PronunciationAssessmentRequest
): Promise<PronunciationAssessmentResult> {
  if (isTauri) {
    const res = await invokeTauri<{ result: PronunciationAssessmentResult }>(
      'lingua_command_assess_pronunciation',
      { request: { request: req } }
    );
    return res.result;
  }
  const res = await fetch(`${API_BASE}/learning/pronunciation/assess`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ request: req }),
  });
  if (!res.ok) throw new Error('Failed to assess pronunciation');
  const data = await res.json();
  return data.result;
}

export async function getListeningChallenges(
  lang: import('./types').LanguageCode,
  count = 3
): Promise<ListeningChallenge[]> {
  if (isTauri) {
    const res = await invokeTauri<{ challenges: ListeningChallenge[] }>(
      'lingua_command_get_listening_challenges',
      { request: { language: lang, count } }
    );
    return res.challenges;
  }
  const res = await fetch(`${API_BASE}/learning/listening/challenges?language=${encodeURIComponent(lang)}&count=${count}`);
  if (!res.ok) throw new Error('Failed to fetch listening challenges');
  const data = await res.json();
  return data.challenges;
}

export async function getGrammarTopics(
  lang: import('./types').LanguageCode
): Promise<GrammarGuideTopic[]> {
  if (isTauri) {
    const res = await invokeTauri<{ topics: GrammarGuideTopic[] }>(
      'lingua_command_get_grammar_topics',
      { request: { language: lang } }
    );
    return res.topics;
  }
  const res = await fetch(`${API_BASE}/learning/grammar/topics?language=${encodeURIComponent(lang)}`);
  if (!res.ok) throw new Error('Failed to fetch grammar topics');
  const data = await res.json();
  return data.topics;
}

export async function getPrivacySettings(): Promise<DataPrivacySettings> {
  if (isTauri) {
    const res = await invokeTauri<{ settings: DataPrivacySettings }>(
      'lingua_command_get_privacy_settings'
    );
    return res.settings;
  }
  const res = await fetch(`${API_BASE}/privacy/settings`);
  if (!res.ok) throw new Error('Failed to fetch privacy settings');
  const data = await res.json();
  return data.settings;
}

export async function savePrivacySettings(settings: DataPrivacySettings): Promise<boolean> {
  if (isTauri) {
    return invokeTauri<boolean>('lingua_command_save_privacy_settings', {
      request: { settings },
    });
  }
  const res = await fetch(`${API_BASE}/privacy/settings`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ settings }),
  });
  return res.ok;
}

export async function exportUserData(): Promise<ExportUserDataArchive> {
  if (isTauri) {
    const res = await invokeTauri<{ archive: ExportUserDataArchive }>(
      'lingua_command_export_user_data'
    );
    return res.archive;
  }
  const res = await fetch(`${API_BASE}/privacy/export`, { method: 'POST' });
  if (!res.ok) throw new Error('Failed to export user data');
  const data = await res.json();
  return data.archive;
}

export async function purgeUserData(confirmPhrase: string): Promise<PurgeUserDataResult> {
  if (isTauri) {
    const res = await invokeTauri<{ result: PurgeUserDataResult }>(
      'lingua_command_purge_user_data',
      { request: { confirm_phrase: confirmPhrase } }
    );
    return res.result;
  }
  const res = await fetch(`${API_BASE}/privacy/purge`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ confirm_phrase: confirmPhrase }),
  });
  if (!res.ok) throw new Error('Failed to purge user data');
  const data = await res.json();
  return data.result;
}

export async function getPlatformReleaseStatus(): Promise<PlatformReleaseStatus[]> {
  if (isTauri) {
    const res = await invokeTauri<{ status: PlatformReleaseStatus[] }>(
      'lingua_command_get_platform_release_status'
    );
    return res.status;
  }
  const res = await fetch(`${API_BASE}/platform/release-status`);
  if (!res.ok) throw new Error('Failed to fetch platform release status');
  const data = await res.json();
  return data.status;
}

// =========================================================================
// Stage 8 IPC Helpers: Commercial & Enterprise Platform
// =========================================================================

export async function getEnterpriseAccount(accountId?: string): Promise<UserAccount> {
  if (isTauri) {
    const res = await invokeTauri<{ account: UserAccount }>(
      'lingua_command_get_enterprise_account',
      { request: accountId ? { account_id: accountId } : null }
    );
    return res.account;
  }
  const url = accountId
    ? `${API_BASE}/enterprise/account?account_id=${encodeURIComponent(accountId)}`
    : `${API_BASE}/enterprise/account`;
  const res = await fetch(url);
  if (!res.ok) throw new Error('Failed to fetch enterprise account');
  const data = await res.json();
  return data.account;
}

export async function updateAccountTier(accountId: string, tier: AccountTier): Promise<boolean> {
  if (isTauri) {
    return invokeTauri<boolean>('lingua_command_update_account_tier', {
      accountId,
      tier,
    });
  }
  const res = await fetch(`${API_BASE}/enterprise/account/tier`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ account_id: accountId, tier }),
  });
  return res.ok;
}

export async function registerDevice(
  accountId: string,
  deviceName: string,
  deviceType: DeviceType,
  publicKeyFingerprint: string
): Promise<DeviceRegistration> {
  if (isTauri) {
    const res = await invokeTauri<{ device: DeviceRegistration }>(
      'lingua_command_register_device',
      {
        request: {
          account_id: accountId,
          device_name: deviceName,
          device_type: deviceType,
          public_key_fingerprint: publicKeyFingerprint,
        },
      }
    );
    return res.device;
  }
  const res = await fetch(`${API_BASE}/enterprise/devices`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      account_id: accountId,
      device_name: deviceName,
      device_type: deviceType,
      public_key_fingerprint: publicKeyFingerprint,
    }),
  });
  if (!res.ok) throw new Error('Failed to register device');
  const data = await res.json();
  return data.device;
}

export async function listDevices(accountId?: string): Promise<DeviceRegistration[]> {
  if (isTauri) {
    const res = await invokeTauri<{ devices: DeviceRegistration[] }>(
      'lingua_command_list_devices',
      { accountId }
    );
    return res.devices;
  }
  const res = await fetch(`${API_BASE}/enterprise/devices`);
  if (!res.ok) throw new Error('Failed to list devices');
  const data = await res.json();
  return data.devices;
}

export async function createApiKey(
  userId: string,
  name: string,
  orgId?: string | null
): Promise<{ record: ApiKeyRecord; rawKey: string }> {
  if (isTauri) {
    const res = await invokeTauri<{ api_key_record: ApiKeyRecord; raw_key: string }>(
      'lingua_command_create_api_key',
      {
        request: { user_id: userId, org_id: orgId || null, name },
      }
    );
    return { record: res.api_key_record, rawKey: res.raw_key };
  }
  const res = await fetch(`${API_BASE}/enterprise/keys`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ user_id: userId, org_id: orgId || null, name }),
  });
  if (!res.ok) throw new Error('Failed to create API key');
  const data = await res.json();
  return { record: data.api_key_record, rawKey: data.raw_key };
}

export async function listApiKeys(userId?: string): Promise<ApiKeyRecord[]> {
  if (isTauri) {
    const res = await invokeTauri<{ api_keys: ApiKeyRecord[] }>(
      'lingua_command_list_api_keys',
      { userId }
    );
    return res.api_keys;
  }
  const res = await fetch(`${API_BASE}/enterprise/keys`);
  if (!res.ok) throw new Error('Failed to list API keys');
  const data = await res.json();
  return data.api_keys;
}

export async function revokeApiKey(keyId: string): Promise<boolean> {
  if (isTauri) {
    return invokeTauri<boolean>('lingua_command_revoke_api_key', { keyId });
  }
  return true;
}

export async function getUsageSummary(accountId?: string): Promise<UsageSummary> {
  if (isTauri) {
    const res = await invokeTauri<{ summary: UsageSummary }>(
      'lingua_command_get_usage_summary',
      { accountId }
    );
    return res.summary;
  }
  const res = await fetch(`${API_BASE}/enterprise/usage`);
  if (!res.ok) throw new Error('Failed to fetch usage summary');
  const data = await res.json();
  return data.summary;
}

export async function listInvoices(accountId?: string): Promise<BillingInvoiceRecord[]> {
  if (isTauri) {
    const res = await invokeTauri<{ invoices: BillingInvoiceRecord[] }>(
      'lingua_command_list_invoices',
      { accountId }
    );
    return res.invoices;
  }
  const res = await fetch(`${API_BASE}/enterprise/invoices`);
  if (!res.ok) throw new Error('Failed to list invoices');
  const data = await res.json();
  return data.invoices;
}

export async function createOrganization(
  name: string,
  slug: string,
  ownerId: string
): Promise<Organization> {
  if (isTauri) {
    const res = await invokeTauri<{ organization: Organization }>(
      'lingua_command_create_organization',
      { request: { name, slug, owner_id: ownerId } }
    );
    return res.organization;
  }
  const res = await fetch(`${API_BASE}/enterprise/orgs`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ name, slug, owner_id: ownerId }),
  });
  if (!res.ok) throw new Error('Failed to create organization');
  const data = await res.json();
  return data.organization;
}

export async function listOrgMembers(orgId: string): Promise<OrgMember[]> {
  if (isTauri) {
    const res = await invokeTauri<{ members: OrgMember[] }>(
      'lingua_command_list_org_members',
      { orgId }
    );
    return res.members;
  }
  const res = await fetch(`${API_BASE}/enterprise/orgs/${encodeURIComponent(orgId)}/members`);
  if (!res.ok) throw new Error('Failed to list organization members');
  const data = await res.json();
  return data.members;
}

export async function addOrgMember(
  orgId: string,
  userId: string,
  email: string,
  displayName: string,
  role: string
): Promise<boolean> {
  if (isTauri) {
    return invokeTauri<boolean>('lingua_command_add_org_member', {
      orgId,
      userId,
      email,
      displayName,
      role,
    });
  }
  const res = await fetch(`${API_BASE}/enterprise/orgs/${encodeURIComponent(orgId)}/members`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ user_id: userId, email, display_name: displayName, role }),
  });
  return res.ok;
}

export async function listOrgGlossaries(orgId: string): Promise<OrgGlossary[]> {
  if (isTauri) {
    const res = await invokeTauri<{ glossaries: OrgGlossary[] }>(
      'lingua_command_list_org_glossaries',
      { orgId }
    );
    return res.glossaries;
  }
  const res = await fetch(`${API_BASE}/enterprise/orgs/${encodeURIComponent(orgId)}/glossaries`);
  if (!res.ok) throw new Error('Failed to list organization glossaries');
  const data = await res.json();
  return data.glossaries;
}

export async function createOrgGlossary(
  orgId: string,
  name: string,
  description: string,
  sourceLang: LanguageCode,
  targetLang: LanguageCode
): Promise<boolean> {
  if (isTauri) {
    return invokeTauri<boolean>('lingua_command_create_org_glossary', {
      request: {
        org_id: orgId,
        name,
        description,
        source_lang: sourceLang,
        target_lang: targetLang,
      },
    });
  }
  const res = await fetch(`${API_BASE}/enterprise/orgs/${encodeURIComponent(orgId)}/glossaries`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      org_id: orgId,
      name,
      description,
      source_lang: sourceLang,
      target_lang: targetLang,
    }),
  });
  return res.ok;
}

export async function addOrgGlossaryTerm(
  orgId: string,
  glossaryId: string,
  sourceTerm: string,
  targetTerm: string,
  notes?: string
): Promise<boolean> {
  if (isTauri) {
    return invokeTauri<boolean>('lingua_command_add_org_glossary_term', {
      glossaryId,
      sourceTerm,
      targetTerm,
      notes,
      caseSensitive: false,
    });
  }
  const res = await fetch(
    `${API_BASE}/enterprise/orgs/${encodeURIComponent(orgId)}/glossaries/${encodeURIComponent(glossaryId)}/terms`,
    {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        source_term: sourceTerm,
        target_term: targetTerm,
        notes,
        case_sensitive: false,
      }),
    }
  );
  return res.ok;
}

export async function listAuditEvents(orgId?: string, limit = 50): Promise<AuditEvent[]> {
  if (isTauri) {
    const res = await invokeTauri<{ events: AuditEvent[] }>(
      'lingua_command_list_audit_events',
      { orgId, limit }
    );
    return res.events;
  }
  const url = orgId
    ? `${API_BASE}/enterprise/audit?org_id=${encodeURIComponent(orgId)}&limit=${limit}`
    : `${API_BASE}/enterprise/audit?limit=${limit}`;
  const res = await fetch(url);
  if (!res.ok) throw new Error('Failed to list audit events');
  const data = await res.json();
  return data.events;
}

export async function getWorkerMetrics(): Promise<WorkerMetrics> {
  if (isTauri) {
    const res = await invokeTauri<{ metrics: WorkerMetrics }>(
      'lingua_command_get_worker_metrics'
    );
    return res.metrics;
  }
  const res = await fetch(`${API_BASE}/enterprise/workers/metrics`);
  if (!res.ok) throw new Error('Failed to fetch worker metrics');
  const data = await res.json();
  return data.metrics;
}

