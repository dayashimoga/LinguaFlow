import type {
  AppInfo,
  CapabilityMatrix,
  DatabaseHealthInfo,
  IpcResponse,
  ModelPackInfo,
  UserSettings,
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


