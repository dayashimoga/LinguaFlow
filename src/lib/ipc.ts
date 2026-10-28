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
