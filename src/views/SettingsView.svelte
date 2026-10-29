<script lang="ts">
  import { onMount } from 'svelte';
  import {
    checkDatabaseHealth,
    exportUserData,
    getAudioDevices,
    getAudioOutputDevices,
    getPrivacySettings,
    purgeUserData,
    savePrivacySettings,
  } from '../lib/ipc';
  import {
    SUPPORTED_LANGUAGES,
    type AudioDeviceItem,
    type DatabaseHealthInfo,
    type DataPrivacySettings,
    type EnginePreference,
    type LanguageCode,
    type ThemeMode,
    type UserSettings,
  } from '../lib/types';

  export let settings: UserSettings | null = null;
  export let onSaveSettings: (settings: UserSettings) => Promise<void>;
  export let onNavigate: ((view: string) => void) | undefined = undefined;

  let theme: ThemeMode = settings?.theme ?? 'system';
  let sourceLang: LanguageCode = settings?.source_language ?? 'en-US';
  let targetLang: LanguageCode = settings?.target_language ?? 'kn-IN';
  let fontScale: number = settings?.font_scale ?? 1.0;
  let enginePref: EnginePreference = settings?.engine_preference ?? 'auto_hybrid';
  let offlineOnly: boolean = settings?.offline_only ?? false;
  let vadSensitivity: number = settings?.audio?.vad_sensitivity ?? 0.5;
  let echoCancel: boolean = settings?.audio?.echo_cancellation ?? true;
  let noiseSuppress: boolean = settings?.audio?.noise_suppression ?? true;
  let inputDevices: AudioDeviceItem[] = [];
  let outputDevices: AudioDeviceItem[] = [];
  let selectedInputDevice = settings?.audio?.input_device_id ?? '';
  let selectedOutputDevice = settings?.audio?.output_device_id ?? '';

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
          input_device_id: selectedInputDevice || null,
          output_device_id: selectedOutputDevice || null,
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

  let privacySettings: DataPrivacySettings | null = null;
  let isExporting = false;
  let exportChecksum = '';
  let exportId = '';
  let showPurgeModal = false;
  let purgePhrase = '';
  let isPurging = false;
  let purgeStatusMsg = '';

  async function loadPrivacy() {
    try {
      privacySettings = await getPrivacySettings();
    } catch (e) {
      console.error('Failed to load privacy settings:', e);
    }
  }

  async function handleTogglePrivacy() {
    if (!privacySettings) return;
    try {
      await savePrivacySettings(privacySettings);
    } catch (e) {
      console.error('Failed to update privacy settings:', e);
    }
  }

  async function handleExport() {
    isExporting = true;
    exportChecksum = '';
    try {
      const archive = await exportUserData();
      exportChecksum = archive.checksum_sha256;
      exportId = archive.export_id;
      const blob = new Blob([archive.raw_json], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `lingua-user-export-${archive.export_id.slice(0, 8)}.json`;
      a.click();
      URL.revokeObjectURL(url);
    } catch (e) {
      console.error('Export failed:', e);
    } finally {
      isExporting = false;
    }
  }

  async function handlePurge() {
    if (purgePhrase !== 'I UNDERSTAND THIS ACTION IS IRREVERSIBLE') return;
    isPurging = true;
    purgeStatusMsg = '';
    try {
      const res = await purgeUserData(purgePhrase);
      purgeStatusMsg = `Purged ${res.purged_history_rows} history, ${res.purged_sessions_rows} sessions, ${res.purged_cards_rows} flashcard logs, and ${res.purged_media_rows} media files.`;
      purgePhrase = '';
      setTimeout(() => {
        showPurgeModal = false;
      }, 3000);
    } catch (e: any) {
      purgeStatusMsg = `Purge error: ${e?.message || e}`;
    } finally {
      isPurging = false;
    }
  }

  onMount(async () => {
    loadPrivacy();
    try {
      inputDevices = await getAudioDevices();
      outputDevices = await getAudioOutputDevices();
    } catch (e) {
      console.warn('Failed to load audio hardware devices:', e);
    }
  });
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
    <div class="card-header-row">
      <div>
        <h2>Audio & Conversation Hardware</h2>
        <p class="section-desc">Hardware capture filters, VAD sensitivity, and acoustic feedback elimination.</p>
      </div>
      {#if onNavigate}
        <button class="btn-secondary" on:click={() => onNavigate?.('downloads')}>
          Model Manager ➔
        </button>
      {/if}
    </div>

    <div class="form-grid">
      <div class="form-group">
        <label for="input-device">
          <span class="icon">🎙️</span> Microphone (Audio Input)
        </label>
        <select id="input-device" bind:value={selectedInputDevice}>
          <option value="">Default System Microphone</option>
          {#each inputDevices as dev}
            <option value={dev.id}>{dev.name}</option>
          {/each}
        </select>
      </div>

      <div class="form-group">
        <label for="output-device">
          <span class="icon">🔊</span> Speaker / Headphones (Audio Output)
        </label>
        <select id="output-device" bind:value={selectedOutputDevice}>
          <option value="">Default System Output</option>
          {#each outputDevices as dev}
            <option value={dev.id}>{dev.name}</option>
          {/each}
        </select>
      </div>

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

  <!-- Data Privacy & Governance -->
  <section class="settings-card glass-panel" aria-labelledby="privacy-header">
    <div class="card-header-row">
      <div>
        <h2 id="privacy-header">Data Privacy & Local Governance</h2>
        <p class="section-desc">
          Zero-transcript guarantees (Contract Rule 7), local retention controls, authenticated export, and atomic purge.
        </p>
      </div>
      <span class="badge badge-active">🔒 Zero-Transcript Guarantee</span>
    </div>

    {#if privacySettings}
      <div class="privacy-controls-grid">
        <label class="checkbox-label">
          <input
            type="checkbox"
            bind:checked={privacySettings.analytics_enabled}
            on:change={handleTogglePrivacy}
          />
          <span>Share Anonymous Diagnostics (No User Utterances or Transcripts)</span>
        </label>

        <label class="checkbox-label">
          <input
            type="checkbox"
            bind:checked={privacySettings.microphone_consent}
            on:change={handleTogglePrivacy}
          />
          <span>Microphone Permission Consent for Live Speech & Pronunciation</span>
        </label>

        <div class="form-group">
          <label for="retention-slider">
            Transcript Retention Period: {privacySettings.local_transcripts_retention_days} days
          </label>
          <input
            id="retention-slider"
            type="range"
            min="0"
            max="365"
            step="1"
            bind:value={privacySettings.local_transcripts_retention_days}
            on:change={handleTogglePrivacy}
          />
        </div>
      </div>
    {/if}

    <div class="privacy-actions-row">
      <button class="btn-secondary" on:click={handleExport} disabled={isExporting}>
        {isExporting ? 'Exporting Archive...' : '📦 Export My Data (JSON + SHA-256)'}
      </button>

      <button class="btn-danger" on:click={() => (showPurgeModal = true)}>
        🗑️ Purge All My Data (Right to be Forgotten)
      </button>
    </div>

    {#if exportChecksum}
      <div class="export-feedback">
        [OK] Export ID: <code>{exportId.slice(0, 8)}</code> | SHA-256 Checksum: <code>{exportChecksum}</code>
      </div>
    {/if}

    {#if showPurgeModal}
      <div class="modal-backdrop">
        <div class="modal-card">
          <h3>Confirm Complete Data Purge</h3>
          <p class="warning-text">
            ⚠️ This will permanently and irreversibly erase all translation history, speech sessions, flashcard review logs, and media files from your local SQLite database.
          </p>
          <p>To confirm, type <code>I UNDERSTAND THIS ACTION IS IRREVERSIBLE</code> below:</p>
          <input
            type="text"
            class="confirm-input"
            bind:value={purgePhrase}
            placeholder="I UNDERSTAND THIS ACTION IS IRREVERSIBLE"
          />

          {#if purgeStatusMsg}
            <div class="purge-msg">{purgeStatusMsg}</div>
          {/if}

          <div class="modal-btn-row">
            <button class="btn-danger" on:click={handlePurge} disabled={purgePhrase !== 'I UNDERSTAND THIS ACTION IS IRREVERSIBLE' || isPurging}>
              {isPurging ? 'Purging...' : 'Permanently Purge Data'}
            </button>
            <button class="btn-secondary" on:click={() => (showPurgeModal = false)}>
              Cancel
            </button>
          </div>
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

  .privacy-controls-grid {
    display: flex;
    flex-direction: column;
    gap: 16px;
    margin-top: 8px;
  }

  .privacy-actions-row {
    display: flex;
    gap: 16px;
    flex-wrap: wrap;
    margin-top: 8px;
  }

  .btn-danger {
    padding: 10px 18px;
    border-radius: var(--radius-md);
    background: #dc2626;
    color: #fff;
    border: none;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .btn-danger:hover:not(:disabled) {
    background: #b91c1c;
    box-shadow: 0 4px 12px rgba(220, 38, 38, 0.4);
  }

  .btn-secondary {
    padding: 10px 18px;
    border-radius: var(--radius-md);
    background: var(--bg-surface-elevated);
    color: var(--text-primary);
    border: 1px solid var(--border-subtle);
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .btn-secondary:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.1);
  }

  .export-feedback {
    background: rgba(16, 185, 129, 0.15);
    border: 1px solid #10b981;
    border-radius: var(--radius-md);
    padding: 10px 14px;
    color: #ecfdf5;
    font-size: 0.85rem;
  }

  .modal-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.7);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal-card {
    background: #0f172a;
    border: 1px solid #ef4444;
    border-radius: var(--radius-lg);
    padding: 24px;
    max-width: 520px;
    width: 90%;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .warning-text {
    color: #f87171;
    font-size: 0.9rem;
    line-height: 1.5;
  }

  .confirm-input {
    padding: 10px;
    border-radius: var(--radius-md);
    border: 1px solid rgba(255, 255, 255, 0.2);
    background: #1e293b;
    color: #f8fafc;
    font-family: var(--font-mono);
    font-size: 0.85rem;
  }

  .modal-btn-row {
    display: flex;
    gap: 12px;
    margin-top: 10px;
  }

  .purge-msg {
    color: #34d399;
    font-size: 0.85rem;
    font-weight: 600;
  }
</style>
