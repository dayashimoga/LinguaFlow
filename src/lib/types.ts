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


