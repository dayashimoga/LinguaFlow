export type LanguageCode = 'kn-IN' | 'ta-IN' | 'en-US' | 'hi-IN' | 'te-IN' | 'ml-IN';

export interface LanguageMeta {
  code: LanguageCode;
  short: string;
  name: string;
  native: string;
  script: string;
}

export const SUPPORTED_LANGUAGES: LanguageMeta[] = [
  { code: 'kn-IN', short: 'kn', name: 'Kannada', native: 'ಕನ್ನಡ', script: 'Kannada (Knda)' },
  { code: 'ta-IN', short: 'ta', name: 'Tamil', native: 'தமிழ்', script: 'Tamil (Taml)' },
  { code: 'en-US', short: 'en', name: 'English', native: 'English', script: 'Latin (Latn)' },
];

export type ThemeMode = 'system' | 'light' | 'dark';
export type EnginePreference = 'auto_hybrid' | 'offline_only' | 'online_only';

export interface AudioSettings {
  input_device_id: string | null;
  output_device_id: string | null;
  sample_rate: number;
  channels: number;
  vad_sensitivity: number;
  echo_cancellation: boolean;
  noise_suppression: boolean;
}

export interface UserSettings {
  theme: ThemeMode;
  source_language: LanguageCode;
  target_language: LanguageCode;
  font_scale: number;
  engine_preference: EnginePreference;
  offline_only: boolean;
  audio: AudioSettings;
  updated_at: string;
}

export type CapabilityState =
  | 'Active'
  | 'Ready'
  | 'NotInstalled'
  | 'UnsupportedHardware'
  | 'UnreachableProvider'
  | { ScheduledForStage: number };

export interface CapabilityMatrix {
  dictionary: CapabilityState;
  text_translation: CapabilityState;
  streaming_asr: CapabilityState;
  speech_synthesis: CapabilityState;
  offline_models: CapabilityState;
  live_conversation: CapabilityState;
  media_subtitles: CapabilityState;
}

export interface ModelPackInfo {
  id: string;
  name: string;
  source: LanguageCode;
  target: LanguageCode;
  size_bytes: number;
  version: string;
  sha256: string;
  is_installed: boolean;
  scheduled_stage: number;
}

export interface AppInfo {
  name: string;
  version: string;
  stage: number;
  stage_name: string;
  platform: string;
  arch: string;
  build_profile: string;
}

export interface DatabaseHealthInfo {
  status: string;
  path: string;
  schema_version: number;
  settings_persisted: boolean;
  error?: string | null;
}

export interface IpcResponse<T> {
  request_id: string;
  success: boolean;
  data: T | null;
  error: {
    code: string;
    message: string;
    details?: string;
    recoverable: boolean;
  } | null;
}

export interface DictionaryExample {
  source: string;
  target: string;
}

export interface DictionaryEntry {
  id: string;
  word: string;
  normalized_word: string;
  language: LanguageCode;
  part_of_speech: string;
  definition: string;
  alternate_meanings: string[];
  examples: DictionaryExample[];
  pronunciation_ipa?: string | null;
  transliteration?: string | null;
  synonyms: string[];
  antonyms: string[];
  provenance: string;
}

export interface TranslationRecord {
  id: string;
  source_language: LanguageCode;
  target_language: LanguageCode;
  source_text: string;
  translated_text: string;
  engine_used: string;
  latency_ms: number;
  created_at: string;
}

export interface FavoriteItem {
  id: string;
  source_language: LanguageCode;
  target_language: LanguageCode;
  source_text: string;
  translated_text: string;
  notes?: string | null;
  created_at: string;
}

export interface TranslateCmdRequest {
  source_language: LanguageCode;
  target_language: LanguageCode;
  text: string;
  allow_cloud: boolean;
}

export interface TranslateCmdResponse {
  source_language: LanguageCode;
  target_language: LanguageCode;
  original_text: string;
  translated_text: string;
  engine_id: string;
  is_offline: boolean;
  latency_ms: number;
}

export interface DictionarySearchCmdRequest {
  query: string;
  language?: LanguageCode | null;
  limit?: number;
}

export interface DictionarySearchCmdResponse {
  entries: DictionaryEntry[];
  query: string;
  total_matches: number;
}

export interface DetectLanguageCmdRequest {
  text: string;
}

export interface DetectLanguageCmdResponse {
  detected: LanguageCode | null;
}

export interface AddFavoriteCmdRequest {
  source_language: LanguageCode;
  target_language: LanguageCode;
  source_text: string;
  translated_text: string;
  notes?: string | null;
}

export type ModelInstallStatus =
  | { status: 'NotInstalled' }
  | { status: 'Downloading'; detail: { progress_pct: number; downloaded_bytes: number; total_bytes: number } }
  | { status: 'Installed'; detail: { installed_at: string; install_path: string } }
  | { status: 'Corrupted'; detail: { reason: string } };

export interface ModelPackDetailedInfo {
  id: string;
  name: string;
  source: LanguageCode;
  target: LanguageCode;
  size_bytes: number;
  version: string;
  sha256: string;
  state: ModelInstallStatus;
  architecture: string;
  storage_path?: string | null;
  scheduled_stage: number;
}

export interface DiskSpaceInfo {
  available_bytes: number;
  total_bytes: number;
  models_dir: string;
}

export interface VerifyModelCmdResponse {
  model_id: string;
  is_valid: boolean;
  details: string;
}

export interface GetDiskSpaceCmdResponse {
  available_bytes: number;
  total_bytes: number;
  models_path: string;
}

export interface AudioDeviceItem {
  id: string;
  name: string;
  is_default: boolean;
  channels: number;
  sample_rate: number;
}

export type VadState = 'Silence' | 'Speech';

export interface AsrTranscriptEvent {
  session_id: string;
  seq_id: number;
  text: string;
  is_final: boolean;
  confidence: number;
  start_time_ms: number;
  end_time_ms: number;
  language: LanguageCode;
}

export type SpeechSessionStatus = 'Active' | 'Paused' | 'Completed' | 'Failed';

export interface SpeechSession {
  id: string;
  title: string;
  source_lang: LanguageCode;
  target_lang: LanguageCode;
  status: SpeechSessionStatus;
  created_at: string;
  ended_at: string | null;
  duration_seconds: number;
}

export interface CaptionSegment {
  id: string;
  session_id: string;
  seq_id: number;
  source_lang: LanguageCode;
  target_lang: LanguageCode;
  source_text: string;
  translated_text: string;
  is_final: boolean;
  confidence: number;
  start_time_ms: number;
  end_time_ms: number;
  created_at: string;
}

export type CaptionFormat = 'vtt' | 'srt' | 'json';

export interface StartSpeechSessionCmdRequest {
  source_language: LanguageCode;
  target_language: LanguageCode;
  title?: string | null;
  device_id?: string | null;
}

export interface StartSpeechSessionCmdResponse {
  session: SpeechSession;
}

export interface PauseSpeechSessionCmdRequest {
  session_id: string;
}

export interface ResumeSpeechSessionCmdRequest {
  session_id: string;
}

export interface StopSpeechSessionCmdRequest {
  session_id: string;
}

export interface StopSpeechSessionCmdResponse {
  session: SpeechSession;
}

export interface GetAudioDevicesCmdResponse {
  devices: AudioDeviceItem[];
}

export interface GetSessionCaptionsCmdRequest {
  session_id: string;
}

export interface GetSessionCaptionsCmdResponse {
  captions: CaptionSegment[];
}

export interface ExportCaptionsCmdRequest {
  session_id: string;
  format: CaptionFormat;
}

export interface ExportCaptionsCmdResponse {
  content: string;
  filename: string;
  format: CaptionFormat;
}

export interface FeedAudioChunkCmdRequest {
  session_id: string;
  samples: number[];
}

export type SpeakerId = 'SpeakerA' | 'SpeakerB';

export type TurnState =
  | 'Idle'
  | { Listening: SpeakerId }
  | { Processing: SpeakerId }
  | { Speaking: SpeakerId };

export interface ConversationSession {
  id: string;
  title: string;
  speaker_a_lang: LanguageCode;
  speaker_b_lang: LanguageCode;
  active_speaker: SpeakerId;
  is_hands_free: boolean;
  status: string;
  created_at: string;
  ended_at: string | null;
  total_turns: number;
}

export interface ConversationTurnItem {
  id: string;
  session_id: string;
  turn_number: number;
  speaker_id: SpeakerId;
  source_lang: LanguageCode;
  target_lang: LanguageCode;
  source_text: string;
  translated_text: string;
  audio_duration_ms: number;
  is_interrupted: boolean;
  created_at: string;
}

export interface TtsVoiceInfo {
  id: string;
  name: string;
  language: LanguageCode;
  gender: string;
  is_offline: boolean;
  sample_rate: number;
}

export interface TtsSynthesizeRequest {
  text: string;
  language: LanguageCode;
  voice_id?: string | null;
  speed: number;
  pitch: number;
}

export interface TtsSynthesizeResponse {
  audio_samples: number[];
  sample_rate: number;
  channels: number;
  duration_ms: number;
  voice_id: string;
  language: LanguageCode;
}

export interface StartConversationCmdRequest {
  speaker_a_lang: LanguageCode;
  speaker_b_lang: LanguageCode;
  title?: string | null;
  is_hands_free: boolean;
}

export interface SubmitTurnCmdRequest {
  session_id: string;
  speaker_id: SpeakerId;
  source_text: string;
}

export interface SubmitTurnCmdResponse {
  turn: ConversationTurnItem;
  audio_samples: number[];
  sample_rate: number;
}

export interface SwitchSpeakerCmdRequest {
  session_id: string;
  next_speaker?: SpeakerId | null;
}

export interface InterruptConversationCmdRequest {
  session_id: string;
}

export interface StopConversationCmdRequest {
  session_id: string;
}
export interface GetConversationTurnsCmdRequest {
  session_id: string;
}

// ==========================================
// Stage 5: Translation Intelligence & Optimization
// ==========================================

export type FormalityMode = 'neutral' | 'formal' | 'colloquial';
export type CircuitState = 'closed' | 'open' | 'half_open';
export type ResourceProfile = 'power_saver' | 'balanced' | 'high_performance';

export interface CircuitBreakerStatus {
  provider_id: string;
  state: CircuitState;
  failure_count: number;
  success_count: number;
  last_failure_ms?: number | null;
  total_requests: number;
}

export interface FallbackStep {
  engine_id: string;
  attempt: number;
  success: boolean;
  latency_ms: number;
  reason?: string | null;
}

export interface IntelligenceTranslationMetadata {
  formality: FormalityMode;
  transliterated_input?: string | null;
  protected_terms: string[];
  domain_glossary_used?: string | null;
  fallback_trail: FallbackStep[];
  engine_selected: string;
  chrf_estimated?: number | null;
}

export interface IntelligenceTranslationRequest {
  text: string;
  source_lang?: LanguageCode | null;
  target_lang: LanguageCode;
  formality: FormalityMode;
  enable_transliteration: boolean;
  domain_glossary?: string | null;
}

export interface IntelligenceTranslationResponse {
  source_text: string;
  translated_text: string;
  detected_lang: LanguageCode;
  target_lang: LanguageCode;
  metadata: IntelligenceTranslationMetadata;
}

export interface DomainGlossaryEntry {
  id: string;
  domain: string;
  source_term: string;
  target_term: string;
  source_lang: LanguageCode;
  target_lang: LanguageCode;
  is_exact: boolean;
  created_at: string;
}

export interface EvaluationResult {
  sample_id: string;
  category: string;
  hypothesis: string;
  reference: string;
  chrf_score: number;
  wer_score: number;
  cer_score: number;
}

export interface EvaluationReport {
  total_samples: number;
  mean_chrf: number;
  mean_wer: number;
  mean_cer: number;
  samples: EvaluationResult[];
  evaluated_at: string;
}

export interface TransliterationRequest {
  text: string;
  target_lang: LanguageCode;
}

export interface TransliterationResponse {
  input: string;
  transliterated: string;
  target_lang: LanguageCode;
}

export interface CreateGlossaryTermCmdRequest {
  domain: string;
  source_term: string;
  target_term: string;
  source_lang: LanguageCode;
  target_lang: LanguageCode;
  is_exact: boolean;
}
