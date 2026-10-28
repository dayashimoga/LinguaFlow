<script lang="ts">
  import { checkDatabaseHealth } from '../lib/ipc';
  import {
    SUPPORTED_LANGUAGES,
    type DatabaseHealthInfo,
    type EnginePreference,
    type LanguageCode,
    type ThemeMode,
    type UserSettings,
  } from '../lib/types';

  export let settings: UserSettings | null = null;
  export let onSaveSettings: (settings: UserSettings) => Promise<void>;

  let theme: ThemeMode = settings?.theme ?? 'system';
  let sourceLang: LanguageCode = settings?.source_language ?? 'en-US';
  let targetLang: LanguageCode = settings?.target_language ?? 'kn-IN';
  let fontScale: number = settings?.font_scale ?? 1.0;
  let enginePref: EnginePreference = settings?.engine_preference ?? 'auto_hybrid';
  let offlineOnly: boolean = settings?.offline_only ?? false;
  let vadSensitivity: number = settings?.audio?.vad_sensitivity ?? 0.5;
  let echoCancel: boolean = settings?.audio?.echo_cancellation ?? true;
  let noiseSuppress: boolean = settings?.audio?.noise_suppression ?? true;

  let saving = false;
  let saveSuccess = false;
  let healthInfo: DatabaseHealthInfo | null = null;
  let checkingHealth = false;

  async function handleSave() {
    saving = true;
    saveSuccess = false;
    try {
      const updated: UserSettings = {
        theme,
        source_language: sourceLang,
        target_language: targetLang,
        font_scale: Number(fontScale),
        engine_preference: enginePref,
        offline_only: offlineOnly,
        audio: {
          input_device_id: settings?.audio?.input_device_id ?? null,
          output_device_id: settings?.audio?.output_device_id ?? null,
          sample_rate: 16000,
          channels: 1,
          vad_sensitivity: Number(vadSensitivity),
          echo_cancellation: echoCancel,
          noise_suppression: noiseSuppress,
        },
        updated_at: new Date().toISOString(),
      };
      await onSaveSettings(updated);
      saveSuccess = true;
      setTimeout(() => (saveSuccess = false), 2500);
    } finally {
      saving = false;
    }
  }

  async function runHealthCheck() {
    checkingHealth = true;
    try {
      healthInfo = await checkDatabaseHealth();
    } catch (e: any) {
      healthInfo = {
        status: 'error',
        path: 'failed',
        schema_version: 0,
        settings_persisted: false,
        error: e?.message || 'Failed health check',
      };
    } finally {
      checkingHealth = false;
    }
  }
</script>

<div class="settings-view">
  <!-- General Preferences -->
  <section class="settings-card glass-panel">
    <h2>General & Appearance Preferences</h2>
    <p class="section-desc">Manage theme rendering, default languages, and responsive font scaling.</p>

    <div class="form-grid">
      <div class="form-group">
        <label for="theme-select">Theme Mode</label>
        <select id="theme-select" bind:value={theme}>
          <option value="system">System Default</option>
          <option value="light">Light Theme</option>
          <option value="dark">Dark Theme</option>
        </select>
      </div>

      <div class="form-group">
        <label for="engine-pref">Engine Mode Preference</label>
        <select id="engine-pref" bind:value={enginePref}>
          <option value="auto_hybrid">Auto-Hybrid (Local First with Cloud Fallback)</option>
          <option value="offline_only">Offline Only (Zero External Transmissions)</option>
          <option value="online_only">Online Only</option>
        </select>
      </div>

      <div class="form-group">
        <label for="source-lang">Default Source Language</label>
        <select id="source-lang" bind:value={sourceLang}>
          {#each SUPPORTED_LANGUAGES as lang}
            <option value={lang.code}>{lang.native} ({lang.name})</option>
          {/each}
        </select>
      </div>

      <div class="form-group">
        <label for="target-lang">Default Target Language</label>
        <select id="target-lang" bind:value={targetLang}>
          {#each SUPPORTED_LANGUAGES as lang}
            <option value={lang.code}>{lang.native} ({lang.name})</option>
          {/each}
        </select>
      </div>

      <div class="form-group full-width">
        <label for="font-scale">Typography Scale Factor: {fontScale.toFixed(2)}x</label>
        <input
          id="font-scale"
          type="range"
          min="0.8"
          max="1.5"
          step="0.05"
          bind:value={fontScale}
        />
        <div class="scale-preview indic-text" style="font-size: {fontScale}rem;">
          ಕನ್ನಡ ಲಿಪಿ ಪರೀಕ್ಷೆ • தமிழ் எழுத்து வடிவம் • English Typography Preview
        </div>
      </div>
    </div>
  </section>

  <!-- Audio & Speech Settings -->
  <section class="settings-card glass-panel">
    <h2>Audio & Conversation Hardware</h2>
    <p class="section-desc">Hardware capture filters, VAD sensitivity, and acoustic feedback elimination.</p>

    <div class="form-grid">
      <div class="form-group">
        <label for="vad-slider">VAD Speech Sensitivity: {vadSensitivity}</label>
        <input
          id="vad-slider"
          type="range"
          min="0.1"
          max="0.9"
          step="0.05"
          bind:value={vadSensitivity}
        />
      </div>

      <div class="checkbox-group">
        <label class="checkbox-label">
          <input type="checkbox" bind:checked={echoCancel} />
          <span>Acoustic Echo Cancellation (AEC)</span>
        </label>
        <label class="checkbox-label">
          <input type="checkbox" bind:checked={noiseSuppress} />
          <span>Background Noise Suppression</span>
        </label>
      </div>
    </div>
  </section>

  <!-- SQLite Database Diagnostics -->
  <section class="settings-card glass-panel">
    <div class="card-header-row">
      <div>
        <h2>Database Diagnostics & Storage Health</h2>
        <p class="section-desc">Embedded SQLite persistence health, WAL status, and schema migrations.</p>
      </div>
      <button class="btn-secondary" on:click={runHealthCheck} disabled={checkingHealth}>
        {checkingHealth ? 'Inspecting...' : 'Run Diagnostics'}
      </button>
    </div>

    {#if healthInfo}
      <div class="diag-results">
        <div class="diag-item">
          <span class="d-label">Health Status:</span>
          <span class="badge {healthInfo.status === 'healthy' ? 'badge-active' : 'badge-pending'}">
            {healthInfo.status}
          </span>
        </div>
        <div class="diag-item">
          <span class="d-label">Database Path:</span>
          <code>{healthInfo.path}</code>
        </div>
        <div class="diag-item">
          <span class="d-label">Schema Version:</span>
          <span>v{healthInfo.schema_version}</span>
        </div>
        <div class="diag-item">
          <span class="d-label">Settings Persisted:</span>
          <span>{healthInfo.settings_persisted ? 'Yes' : 'No'}</span>
        </div>
      </div>
    {/if}
  </section>

  <!-- Actions -->
  <div class="actions-row">
    <button class="btn-primary" on:click={handleSave} disabled={saving}>
      {saving ? 'Saving Preferences...' : 'Save Settings to SQLite'}
    </button>
    {#if saveSuccess}
      <span class="save-toast">Preferences successfully saved to SQLite!</span>
    {/if}
  </div>
</div>

<style>
  .settings-view {
    display: flex;
    flex-direction: column;
    gap: 24px;
  }

  .settings-card {
    padding: 28px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .card-header-row {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    flex-wrap: wrap;
    gap: 16px;
  }

  .section-desc {
    color: var(--text-secondary);
    font-size: 0.95rem;
    line-height: 1.5;
  }

  .form-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
    gap: 20px;
    margin-top: 8px;
  }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .form-group.full-width {
    grid-column: 1 / -1;
  }

  label {
    font-size: 0.9rem;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .scale-preview {
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: 16px;
    margin-top: 8px;
    color: var(--text-accent);
    line-height: 1.6;
  }

  .checkbox-group {
    display: flex;
    flex-direction: column;
    gap: 12px;
    justify-content: center;
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 10px;
    cursor: pointer;
    font-size: 0.95rem;
    color: var(--text-primary);
  }

  .checkbox-label input {
    width: 18px;
    height: 18px;
    accent-color: var(--primary-500);
  }

  .diag-results {
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: 18px;
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
    gap: 14px;
  }

  .diag-item {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .d-label {
    font-size: 0.8rem;
    color: var(--text-muted);
    font-weight: 600;
  }

  .diag-item code {
    background: var(--bg-surface-elevated);
    padding: 2px 6px;
    border-radius: 4px;
    font-family: var(--font-mono);
    font-size: 0.85rem;
  }

  .actions-row {
    display: flex;
    align-items: center;
    gap: 16px;
    padding-top: 8px;
  }

  .save-toast {
    color: #34d399;
    font-weight: 600;
    font-size: 0.95rem;
  }
</style>
