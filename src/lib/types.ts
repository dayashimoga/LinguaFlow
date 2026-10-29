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
  language_learning: CapabilityState;
  enterprise_platform: CapabilityState;
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

export interface LexiconSense {
  sense_index: number;
  part_of_speech: string;
  definition: string;
  register: string;
  synonyms: string[];
  antonyms: string[];
  examples: DictionaryExample[];
}

export interface RichDictionaryEntry {
  id: string;
  headword: string;
  normalized_stem: string;
  language: LanguageCode;
  ipa_pronunciation?: string | null;
  native_phonetic_respelling?: string | null;
  part_of_speech: string;
  gender_inflection_notes?: string | null;
  register: string;
  senses: LexiconSense[];
  bilingual_examples: DictionaryExample[];
  synonyms: string[];
  antonyms: string[];
  derived_forms: string[];
  target_equivalents: Record<string, string[]>;
  frequency_rank: number;
  provenance: string;
}

export interface DictionarySearchCmdRequest {
  query: string;
  language?: LanguageCode | null;
  part_of_speech?: string | null;
  limit?: number;
}

export interface DictionarySearchCmdResponse {
  entries: DictionaryEntry[];
  rich_entries?: RichDictionaryEntry[];
  resolved_stem?: string | null;
  did_you_mean?: string[];
  query: string;
  total_matches: number;
  latency_ms?: number;
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

// Stage 6 Types: Media, Captions, OCR, Document & Integrations

export type MediaFormat = 'wav' | 'mp3' | 'mp4' | 'webm' | 'ogg';
export type MediaProgressStatus = 'queued' | 'processing' | 'completed' | 'cancelled' | 'failed';
export type MediaSubtitleExportFormat = 'srt' | 'vtt' | 'txt' | 'json';

export interface MediaSubtitleSegment {
  id: string;
  session_id: string;
  seq_id: number;
  start_ms: number;
  end_ms: number;
  source_text: string;
  translated_text: string;
}

export interface MediaSession {
  id: string;
  title: string;
  file_name: string;
  file_size_bytes: number;
  duration_ms: number;
  source_lang: LanguageCode;
  target_lang: LanguageCode;
  format: MediaFormat;
  status: MediaProgressStatus;
  created_at: string;
}

export interface OverlaySettings {
  enabled: boolean;
  always_on_top: boolean;
  x: number;
  y: number;
  width: number;
  height: number;
  font_size: number;
  font_family: string;
  text_color: string;
  background_color: string;
  opacity: number;
  source_lang: LanguageCode;
  target_lang: LanguageCode;
  show_source: boolean;
  show_translated: boolean;
}

export type SystemAudioPermission = 'not_determined' | 'granted' | 'denied' | 'restricted';

export interface OcrBoundingBox {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface OcrTextBlock {
  id: string;
  text: string;
  confidence: number;
  bounding_box: OcrBoundingBox;
  line_number: number;
  translated_text?: string | null;
}

export interface OcrResult {
  blocks: OcrTextBlock[];
  detected_language: LanguageCode;
  full_text: string;
  translated_full_text?: string | null;
  processing_time_ms: number;
}

export type DocumentFormat = 'txt' | 'markdown' | 'pdf' | 'csv' | 'html';
export type DocumentJobStatus = 'queued' | 'processing' | 'completed' | 'failed' | 'cancelled';

export interface DocumentBlock {
  index: number;
  block_type: string;
  source_text: string;
  translated_text: string;
  page_number?: number | null;
}

export interface DocumentTranslationJob {
  id: string;
  file_name: string;
  format: DocumentFormat;
  file_size_bytes: number;
  source_lang: LanguageCode;
  target_lang: LanguageCode;
  total_blocks: number;
  completed_blocks: number;
  status: DocumentJobStatus;
  created_at: string;
}

export interface GlobalShortcutConfig {
  quick_translate: string;
  overlay_toggle: string;
  enabled: boolean;
}

// ============================================================================
// Stage 7: Learning & Production Certification Types
// ============================================================================

export type ReviewRating = 0 | 1 | 2 | 3 | 4 | 5;

export interface LearningCard {
  id: string;
  deck_id: string;
  source_lang: LanguageCode;
  target_lang: LanguageCode;
  source_term: string;
  target_term: string;
  romanization?: string | null;
  ipa?: string | null;
  part_of_speech?: string | null;
  example_source?: string | null;
  example_target?: string | null;
  grammar_notes?: string | null;
  repetition: number;
  interval_days: number;
  ease_factor: number;
  next_review_at: string;
  total_reviews: number;
  created_at: string;
}

export interface LearningDeck {
  id: string;
  source_lang: LanguageCode;
  target_lang: LanguageCode;
  title: string;
  description: string;
  total_cards: number;
  created_at: string;
}

export interface ReviewResult {
  card_id: string;
  rating: number;
  new_repetition: number;
  new_interval_days: number;
  new_ease_factor: number;
  next_review_at: string;
}

export interface LearnerProgress {
  total_cards_studied: number;
  cards_due_today: number;
  cards_mastered: number;
  current_streak_days: number;
  retention_rate: number;
  study_time_minutes: number;
}

export interface PronunciationAssessmentRequest {
  reference_text: string;
  spoken_transcript: string;
  language: LanguageCode;
}

export interface PronunciationAssessmentResult {
  accuracy_score: number;
  is_passing: boolean;
  chrf: number;
  wer: number;
  cer: number;
  phoneme_feedback: string[];
}

export interface ListeningChallenge {
  id: string;
  prompt_audio_text: string;
  language: LanguageCode;
  correct_answer: string;
  options: string[];
  explanation: string;
}

export interface GrammarGuideTopic {
  id: string;
  language: LanguageCode;
  title: string;
  category: string;
  content_markdown: string;
  examples: string[];
}

export interface DataPrivacySettings {
  analytics_enabled: boolean;
  local_transcripts_retention_days: number;
  microphone_consent: boolean;
  camera_consent: boolean;
  cloud_processing_consent: boolean;
  policy_version_accepted: string;
}

export interface ExportUserDataArchive {
  export_id: string;
  exported_at: string;
  app_version: string;
  translation_history_count: number;
  learning_cards_count: number;
  checksum_sha256: string;
  raw_json: string;
}

export interface PurgeUserDataResult {
  purged_history_rows: number;
  purged_sessions_rows: number;
  purged_cards_rows: number;
  purged_media_rows: number;
}

export interface PlatformReleaseStatus {
  platform: string;
  binary_name: string;
  is_signed: boolean;
  signature_details: string;
  limitations: string[];
}

export interface ListLearningDecksCmdResponse {
  decks: LearningDeck[];
}

export interface GetLearningDeckCardsCmdRequest {
  deck_id: string;
}

export interface GetLearningDeckCardsCmdResponse {
  cards: LearningCard[];
}

export interface ReviewCardCmdRequest {
  card_id: string;
  rating: number;
}

export interface ReviewCardCmdResponse {
  result: ReviewResult;
}

export interface GetLearnerProgressCmdResponse {
  progress: LearnerProgress;
}

export interface AssessPronunciationCmdRequest {
  request: PronunciationAssessmentRequest;
}

export interface AssessPronunciationCmdResponse {
  result: PronunciationAssessmentResult;
}

export interface GetListeningChallengesCmdRequest {
  language: LanguageCode;
  count: number;
}

export interface GetListeningChallengesCmdResponse {
  challenges: ListeningChallenge[];
}

export interface GetGrammarTopicsCmdRequest {
  language: LanguageCode;
}

export interface GetGrammarTopicsCmdResponse {
  topics: GrammarGuideTopic[];
}

export interface GetDataPrivacySettingsCmdResponse {
  settings: DataPrivacySettings;
}

export interface SaveDataPrivacySettingsCmdRequest {
  settings: DataPrivacySettings;
}

export interface ExportUserDataCmdResponse {
  archive: ExportUserDataArchive;
}

export interface PurgeUserDataCmdRequest {
  confirm_phrase: string;
}

export interface PurgeUserDataCmdResponse {
  result: PurgeUserDataResult;
}

export interface GetPlatformReleaseStatusCmdResponse {
  status: PlatformReleaseStatus[];
}

// =========================================================================
// Stage 8 Types: Commercial & Enterprise Platform
// =========================================================================

export type AccountTier = 'free' | 'pro' | 'enterprise';
export type AccountStatus = 'active' | 'suspended' | 'pending_verification';

export interface UserAccount {
  id: string;
  email: string;
  display_name: string;
  tier: AccountTier;
  is_anonymous: boolean;
  status: AccountStatus;
  created_at: string;
}

export type DeviceType =
  | 'desktop_windows'
  | 'desktop_mac'
  | 'desktop_linux'
  | 'mobile_android'
  | 'mobile_ios'
  | 'web_browser';

export interface DeviceRegistration {
  id: string;
  user_id: string;
  device_name: string;
  device_type: DeviceType;
  public_key_fingerprint: string;
  registered_at: string;
  last_seen_at: string;
}

export interface ApiKeyRecord {
  id: string;
  user_id: string;
  org_id: string | null;
  key_hash: string;
  key_prefix: string;
  name: string;
  rate_limit_per_min: number;
  is_active: boolean;
  created_at: string;
  expires_at: string | null;
}

export type OrgRole = 'owner' | 'admin' | 'linguist_editor' | 'member' | 'viewer';

export interface Organization {
  id: string;
  name: string;
  slug: string;
  max_seats: number;
  sso_enabled: boolean;
  data_retention_days: number;
  enforce_zero_transcript_logging: boolean;
  created_at: string;
}

export interface OrgMember {
  id: string;
  org_id: string;
  user_id: string;
  email: string;
  display_name: string;
  role: OrgRole;
  joined_at: string;
}

export type OrgGlossaryStatus = 'draft' | 'pending_review' | 'approved' | 'deprecated';

export interface OrgGlossary {
  id: string;
  org_id: string;
  name: string;
  description: string;
  source_lang: LanguageCode;
  target_lang: LanguageCode;
  status: OrgGlossaryStatus;
  terms_count: number;
  created_at: string;
  updated_at: string;
}

export interface OrgGlossaryTerm {
  id: string;
  glossary_id: string;
  source_term: string;
  target_term: string;
  notes: string | null;
  case_sensitive: boolean;
}

export interface UsageSummary {
  account_id: string;
  tier: AccountTier;
  words_used: number;
  words_limit: number;
  speech_seconds_used: number;
  speech_seconds_limit: number;
  ocr_pages_used: number;
  ocr_pages_limit: number;
  current_period_cents: number;
  projected_month_end_cents: number;
}

export type InvoiceStatus = 'pending' | 'paid' | 'refunded';

export interface InvoiceLineItem {
  description: string;
  quantity: number;
  unit_price_cents: number;
  total_cents: number;
}

export interface BillingInvoiceRecord {
  id: string;
  account_id: string;
  org_id: string | null;
  invoice_number: string;
  billing_period: string;
  amount_cents: number;
  currency: string;
  status: InvoiceStatus;
  line_items: InvoiceLineItem[];
  created_at: string;
}

export interface AuditEvent {
  id: string;
  org_id: string | null;
  actor_id: string;
  actor_email: string;
  action: string;
  resource_type: string;
  resource_id: string;
  details: string;
  ip_address: string;
  timestamp: string;
}

export interface WorkerMetrics {
  active_workers: number;
  queue_depth: number;
  max_queue_capacity: number;
  processed_tasks: number;
  failed_tasks: number;
  avg_latency_ms: number;
  is_accepting_tasks: boolean;
}

export interface GetAccountInfoCmdRequest {
  account_id?: string;
}

export interface GetAccountInfoCmdResponse {
  account: UserAccount;
}

export interface RegisterDeviceCmdRequest {
  account_id: string;
  device_name: string;
  device_type: DeviceType;
  public_key_fingerprint: string;
}

export interface RegisterDeviceCmdResponse {
  device: DeviceRegistration;
}

export interface ListDevicesCmdResponse {
  devices: DeviceRegistration[];
}

export interface CreateApiKeyCmdRequest {
  user_id: string;
  org_id?: string | null;
  name: string;
}

export interface CreateApiKeyCmdResponse {
  api_key_record: ApiKeyRecord;
  raw_key: string;
}

export interface ListApiKeysCmdResponse {
  api_keys: ApiKeyRecord[];
}

export interface GetUsageSummaryCmdResponse {
  summary: UsageSummary;
}

export interface ListInvoicesCmdResponse {
  invoices: BillingInvoiceRecord[];
}

export interface CreateOrgCmdRequest {
  name: string;
  slug: string;
  owner_id: string;
}

export interface OrgCmdResponse {
  organization: Organization;
}

export interface ListOrgMembersCmdResponse {
  members: OrgMember[];
}

export interface ListOrgGlossariesCmdResponse {
  glossaries: OrgGlossary[];
}

export interface CreateOrgGlossaryCmdRequest {
  org_id: string;
  name: string;
  description: string;
  source_lang: LanguageCode;
  target_lang: LanguageCode;
}

export interface ListAuditEventsCmdResponse {
  events: AuditEvent[];
}

export interface GetWorkerMetricsCmdResponse {
  metrics: WorkerMetrics;
}

