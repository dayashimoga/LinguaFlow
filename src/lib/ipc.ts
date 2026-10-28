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
  limit: number = 20
): Promise<import('./types').DictionarySearchCmdResponse> {
  if (isTauri) {
    return invokeTauri<import('./types').DictionarySearchCmdResponse>('lingua_command_dictionary_search', {
      request: { query, language, limit },
    });
  }
  const params = new URLSearchParams({ q: query, limit: limit.toString() });
  if (language) params.append('lang', language);
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
