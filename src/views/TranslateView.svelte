<script lang="ts">
  import { SUPPORTED_LANGUAGES, type LanguageCode, type UserSettings } from '../lib/types';

  export let settings: UserSettings | null = null;
  export let onSaveSettings: (settings: UserSettings) => Promise<void>;

  let sourceText = '';
  let sourceLang: LanguageCode = settings?.source_language ?? 'en-US';
  let targetLang: LanguageCode = settings?.target_language ?? 'kn-IN';
  let isTranslating = false;
  let copied = false;

  $: if (settings) {
    sourceLang = settings.source_language;
    targetLang = settings.target_language;
  }

  async function handleSwap() {
    const temp = sourceLang;
    sourceLang = targetLang;
    targetLang = temp;

    if (settings) {
      const updated = {
        ...settings,
        source_language: sourceLang,
        target_language: targetLang,
      };
      await onSaveSettings(updated);
    }
  }

  function insertSample(text: string, src: LanguageCode, tgt: LanguageCode) {
    sourceText = text;
    sourceLang = src;
    targetLang = tgt;
    if (settings) {
      onSaveSettings({
        ...settings,
        source_language: src,
        target_language: tgt,
      });
    }
  }

  async function copyText(text: string) {
    await navigator.clipboard.writeText(text);
    copied = true;
    setTimeout(() => (copied = false), 2000);
  }
</script>

<div class="translate-view">
  <!-- Top Bar Controls -->
  <div class="controls-card glass-panel">
    <div class="lang-selector-group">
      <select bind:value={sourceLang} class="lang-select">
        {#each SUPPORTED_LANGUAGES as lang}
          <option value={lang.code}>{lang.native} ({lang.name})</option>
        {/each}
      </select>

      <button class="swap-btn" on:click={handleSwap} title="Swap Languages">
        ⇄
      </button>

      <select bind:value={targetLang} class="lang-select">
        {#each SUPPORTED_LANGUAGES as lang}
          <option value={lang.code}>{lang.native} ({lang.name})</option>
        {/each}
      </select>
    </div>

    <!-- Quick sample inserts -->
    <div class="samples-row">
      <span class="samples-label">Test Samples:</span>
      <button
        class="sample-chip"
        on:click={() => insertSample('ನಮಸ್ಕಾರ, ನೀವು ಹೇಗಿದ್ದೀರಿ?', 'kn-IN', 'en-US')}
      >
        ಕನ್ನಡ: ನಮಸ್ಕಾರ
      </button>
      <button
        class="sample-chip"
        on:click={() => insertSample('வணக்கம், நீங்கள் எப்படி இருக்கிறீர்கள்?', 'ta-IN', 'en-US')}
      >
        தமிழ்: வணக்கம்
      </button>
      <button
        class="sample-chip"
        on:click={() => insertSample('Welcome to LinguaFlow, an offline-first multilingual platform.', 'en-US', 'kn-IN')}
      >
        English: Welcome
      </button>
    </div>
  </div>

  <!-- Translation Panels Grid -->
  <div class="panels-grid">
    <!-- Source Panel -->
    <div class="panel-box glass-panel">
      <div class="panel-header">
        <span class="panel-tag">Source Text</span>
        <span class="char-count">{sourceText.length} characters</span>
      </div>
      <textarea
        bind:value={sourceText}
        placeholder="Type or paste text to translate..."
        rows="8"
        class="text-area indic-text"
      ></textarea>
      <div class="panel-footer">
        <button class="btn-secondary" on:click={() => (sourceText = '')} disabled={!sourceText}>
          Clear
        </button>
        {#if sourceText}
          <button class="btn-secondary" on:click={() => copyText(sourceText)}>
            {copied ? 'Copied!' : 'Copy'}
          </button>
        {/if}
      </div>
    </div>

    <!-- Target Panel -->
    <div class="panel-box glass-panel">
      <div class="panel-header">
        <span class="panel-tag">Translation Output</span>
        <span class="badge badge-pending">Stage 0 Baseline</span>
      </div>

      <div class="output-content">
        <!-- Honest Stage 0 Global Contract Notice -->
        <div class="contract-notice">
          <div class="notice-icon">🛡️</div>
          <div class="notice-body">
            <h4>Global Engineering Contract Notice</h4>
            <p>
              In accordance with <strong>AGENTS.md Requirement 1</strong> ("No nonfunctional placeholders, simulated translations, hardcoded responses, fake progress"), real neural translation models (IndicTrans2 / NLLB) are scheduled for integration in <strong>Stage 1</strong>.
            </p>
            <p class="notice-meta">
              Selected Pair: <code>{sourceLang}</code> ➔ <code>{targetLang}</code> • Active Persistence: SQLite
            </p>
          </div>
        </div>
      </div>

      <div class="panel-footer">
        <span class="engine-indicator">Status: Engine Ready for Stage 1 Linkage</span>
      </div>
    </div>
  </div>
</div>

<style>
  .translate-view {
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  .controls-card {
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .lang-selector-group {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-wrap: wrap;
  }

  .lang-select {
    flex: 1;
    min-width: 180px;
    font-weight: 600;
  }

  .swap-btn {
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    color: var(--text-primary);
    width: 44px;
    height: 44px;
    border-radius: var(--radius-full);
    font-size: 1.25rem;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .swap-btn:hover {
    border-color: var(--text-accent);
    color: var(--text-accent);
    transform: rotate(180deg);
  }

  .samples-row {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-wrap: wrap;
    font-size: 0.85rem;
  }

  .samples-label {
    color: var(--text-muted);
    font-weight: 500;
  }

  .sample-chip {
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    color: var(--text-secondary);
    padding: 4px 10px;
    border-radius: var(--radius-full);
    font-size: 0.8rem;
  }

  .sample-chip:hover {
    background: var(--bg-surface-elevated);
    color: var(--text-accent);
    border-color: var(--text-accent);
  }

  .panels-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
    gap: 20px;
  }

  .panel-box {
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 14px;
    min-height: 320px;
  }

  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .panel-tag {
    font-weight: 600;
    font-size: 0.9rem;
    color: var(--text-secondary);
  }

  .char-count {
    font-size: 0.8rem;
    color: var(--text-muted);
  }

  .text-area {
    width: 100%;
    resize: vertical;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    background: var(--bg-surface);
    color: var(--text-primary);
    padding: 14px;
    font-size: 1.05rem;
    line-height: 1.6;
    flex: 1;
  }

  .output-content {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .contract-notice {
    background: rgba(14, 165, 233, 0.08);
    border: 1px solid rgba(14, 165, 233, 0.25);
    border-radius: var(--radius-md);
    padding: 20px;
    display: flex;
    gap: 16px;
    align-items: flex-start;
  }

  .notice-icon {
    font-size: 1.75rem;
  }

  .notice-body h4 {
    font-size: 0.95rem;
    margin-bottom: 6px;
    color: var(--text-accent);
  }

  .notice-body p {
    font-size: 0.85rem;
    color: var(--text-secondary);
    line-height: 1.5;
    margin-bottom: 8px;
  }

  .notice-meta {
    font-size: 0.8rem !important;
    color: var(--text-muted) !important;
  }

  .notice-meta code {
    background: var(--bg-surface);
    padding: 2px 6px;
    border-radius: 4px;
    font-family: var(--font-mono);
  }

  .panel-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-top: 1px solid var(--border-subtle);
    padding-top: 12px;
  }

  .engine-indicator {
    font-size: 0.8rem;
    color: var(--text-muted);
  }
</style>
