<script lang="ts">
  import { onMount } from 'svelte';
  import {
    SUPPORTED_LANGUAGES,
    type LanguageCode,
    type UserSettings,
    type TranslateCmdResponse,
    type DictionaryEntry,
    type TranslationRecord,
    type FavoriteItem,
  } from '../lib/types';
  import {
    translateText,
    searchDictionary,
    detectLanguage,
    listTranslationHistory,
    clearTranslationHistory,
    addFavorite,
    listFavorites,
    removeFavorite,
  } from '../lib/ipc';

  export let settings: UserSettings | null = null;
  export let onSaveSettings: (settings: UserSettings) => Promise<void>;

  let sourceText = '';
  let sourceLang: LanguageCode = settings?.source_language ?? 'en-US';
  let targetLang: LanguageCode = settings?.target_language ?? 'kn-IN';
  let allowCloud = !(settings?.offline_only ?? false);

  let isTranslating = false;
  let translationResult: TranslateCmdResponse | null = null;
  let translationError: string | null = null;
  let copied = false;
  let isFavorited = false;

  let debounceTimer: ReturnType<typeof setTimeout> | null = null;

  // Word Drill-down State
  let activeDrilldownEntry: DictionaryEntry | null = null;
  let isDrilldownLoading = false;

  // History & Favorites Drawer
  let showHistory = false;
  let historyList: TranslationRecord[] = [];
  let favoritesList: FavoriteItem[] = [];

  $: if (settings) {
    sourceLang = settings.source_language;
    targetLang = settings.target_language;
    allowCloud = !settings.offline_only;
  }

  onMount(async () => {
    await refreshHistoryAndFavorites();
  });

  async function refreshHistoryAndFavorites() {
    try {
      historyList = await listTranslationHistory(15);
      favoritesList = await listFavorites();
      checkIfFavorited();
    } catch (e) {
      console.error('Failed to load history/favorites:', e);
    }
  }

  function checkIfFavorited() {
    if (!translationResult) {
      isFavorited = false;
      return;
    }
    isFavorited = favoritesList.some(
      (f) =>
        f.source_text.trim().toLowerCase() === translationResult?.original_text.trim().toLowerCase() &&
        f.source_language === translationResult?.source_language &&
        f.target_language === translationResult?.target_language
    );
  }

  // Handle Input Changes with Adaptive Debounce
  function handleInput() {
    if (debounceTimer) clearTimeout(debounceTimer);

    if (!sourceText.trim()) {
      translationResult = null;
      translationError = null;
      isTranslating = false;
      return;
    }

    // Quick auto-detection for Kannada and Tamil
    detectLanguage(sourceText).then((detected) => {
      if (detected && detected !== sourceLang && (detected === 'kn-IN' || detected === 'ta-IN')) {
        sourceLang = detected as LanguageCode;
        if (targetLang === sourceLang) {
          targetLang = sourceLang === 'kn-IN' ? 'en-US' : 'kn-IN';
        }
      }
    });

    debounceTimer = setTimeout(() => {
      executeTranslation();
    }, 250);
  }

  async function executeTranslation() {
    const text = sourceText.trim();
    if (!text) return;

    isTranslating = true;
    translationError = null;

    try {
      const res = await translateText({
        source_language: sourceLang,
        target_language: targetLang,
        text,
        allow_cloud: allowCloud,
      });
      translationResult = res;
      checkIfFavorited();
      await refreshHistoryAndFavorites();
    } catch (e: any) {
      translationResult = null;
      translationError = e?.message || 'Translation failed';
    } finally {
      isTranslating = false;
    }
  }

  async function handleSwap() {
    const temp = sourceLang;
    sourceLang = targetLang;
    targetLang = temp;

    if (translationResult) {
      sourceText = translationResult.translated_text;
      translationResult = null;
    }

    if (settings) {
      const updated = {
        ...settings,
        source_language: sourceLang,
        target_language: targetLang,
      };
      await onSaveSettings(updated);
    }

    executeTranslation();
  }

  function handleKeyDown(event: KeyboardEvent) {
    if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 's') {
      event.preventDefault();
      handleSwap();
    }
  }

  async function copyText(text: string) {
    await navigator.clipboard.writeText(text);
    copied = true;
    setTimeout(() => (copied = false), 2000);
  }

  async function toggleFavorite() {
    if (!translationResult) return;

    if (isFavorited) {
      const existing = favoritesList.find(
        (f) =>
          f.source_text.trim().toLowerCase() === translationResult?.original_text.trim().toLowerCase() &&
          f.source_language === translationResult?.source_language &&
          f.target_language === translationResult?.target_language
      );
      if (existing) {
        await removeFavorite(existing.id);
      }
    } else {
      await addFavorite({
        source_language: translationResult.source_language,
        target_language: translationResult.target_language,
        source_text: translationResult.original_text,
        translated_text: translationResult.translated_text,
      });
    }
    await refreshHistoryAndFavorites();
  }

  async function drilldownWord(word: string, lang: LanguageCode) {
    const cleaned = word.replace(/[.,/#!$%^&*;:{}=\-_`~()?"'–—]/g, '').trim();
    if (!cleaned) return;

    isDrilldownLoading = true;
    try {
      const res = await searchDictionary(cleaned, lang, 1);
      if (res.entries.length > 0) {
        activeDrilldownEntry = res.entries[0];
      } else {
        // Fallback search across any language
        const fallback = await searchDictionary(cleaned, null, 1);
        activeDrilldownEntry = fallback.entries.length > 0 ? fallback.entries[0] : null;
      }
    } catch (e) {
      console.error('Drilldown failed:', e);
    } finally {
      isDrilldownLoading = false;
    }
  }

  function restoreHistory(rec: TranslationRecord) {
    sourceLang = rec.source_language;
    targetLang = rec.target_language;
    sourceText = rec.source_text;
    showHistory = false;
    executeTranslation();
  }

  function insertSample(text: string, src: LanguageCode, tgt: LanguageCode) {
    sourceText = text;
    sourceLang = src;
    targetLang = tgt;
    executeTranslation();
  }
</script>

<svelte:window on:keydown={handleKeyDown} />

<div class="translate-view">
  <!-- Controls Bar -->
  <div class="controls-card glass-panel">
    <div class="top-row">
      <div class="lang-selector-group">
        <select bind:value={sourceLang} on:change={executeTranslation} class="lang-select">
          {#each SUPPORTED_LANGUAGES as lang}
            <option value={lang.code}>{lang.native} ({lang.name})</option>
          {/each}
        </select>

        <button class="swap-btn" on:click={handleSwap} title="Swap Languages (Ctrl+S)">
          ⇄
        </button>

        <select bind:value={targetLang} on:change={executeTranslation} class="lang-select">
          {#each SUPPORTED_LANGUAGES as lang}
            <option value={lang.code}>{lang.native} ({lang.name})</option>
          {/each}
        </select>
      </div>

      <div class="mode-toggles">
        <label class="cloud-toggle" title="Allow HTTP Cloud translation when terms are absent from offline lexicon">
          <input
            type="checkbox"
            bind:checked={allowCloud}
            on:change={executeTranslation}
          />
          <span class="toggle-label">{allowCloud ? 'Cloud Online' : 'Offline Lexicon'}</span>
        </label>

        <button
          class="btn-icon"
          class:active={showHistory}
          on:click={() => (showHistory = !showHistory)}
          title="Translation History & Favorites"
        >
          🕒 History ({historyList.length})
        </button>
      </div>
    </div>

    <!-- Quick test samples -->
    <div class="samples-row">
      <span class="samples-label">Quick Samples:</span>
      <button class="sample-chip" on:click={() => insertSample('ನಮಸ್ಕಾರ', 'kn-IN', 'en-US')}>
        ಕನ್ನಡ: ನಮಸ್ಕಾರ
      </button>
      <button class="sample-chip" on:click={() => insertSample('வணக்கம்', 'ta-IN', 'en-US')}>
        தமிழ்: வணக்கம்
      </button>
      <button class="sample-chip" on:click={() => insertSample('Water', 'en-US', 'kn-IN')}>
        English: Water ➔ ಕನ್ನಡ
      </button>
      <button class="sample-chip" on:click={() => insertSample('Peace', 'en-US', 'ta-IN')}>
        English: Peace ➔ தமிழ்
      </button>
      <button class="sample-chip" on:click={() => insertSample('ಸ್ನೇಹ', 'kn-IN', 'ta-IN')}>
        ಕನ್ನಡ ➔ தமிழ்: ಸ್ನೇಹ
      </button>
    </div>
  </div>

  <!-- Translation Panels -->
  <div class="panels-grid">
    <!-- Source Panel -->
    <div class="panel-box glass-panel">
      <div class="panel-header">
        <span class="panel-tag">Source Text ({sourceLang})</span>
        <span class="char-count">{sourceText.length} characters</span>
      </div>
      <textarea
        bind:value={sourceText}
        on:input={handleInput}
        placeholder="Type or paste words/sentences to translate..."
        rows="7"
        class="text-area indic-text"
      ></textarea>

      <!-- Word drill-down chips on source tokens -->
      {#if sourceText.trim()}
        <div class="token-chips">
          <span class="tokens-label">Inspect words:</span>
          {#each sourceText.trim().split(/\s+/) as word}
            <button
              class="token-chip"
              on:click={() => drilldownWord(word, sourceLang)}
              title="Click to view dictionary definition for '{word}'"
            >
              {word}
            </button>
          {/each}
        </div>
      {/if}

      <div class="panel-footer">
        <button
          class="btn-secondary"
          on:click={() => {
            sourceText = '';
            translationResult = null;
            translationError = null;
          }}
          disabled={!sourceText}
        >
          Clear
        </button>
        {#if sourceText}
          <button class="btn-secondary" on:click={() => copyText(sourceText)}>
            {copied ? 'Copied!' : 'Copy'}
          </button>
        {/if}
      </div>
    </div>

    <!-- Output Panel -->
    <div class="panel-box glass-panel output-panel">
      <div class="panel-header">
        <span class="panel-tag">Translated Text ({targetLang})</span>
        {#if translationResult}
          <div class="badge-group">
            <span class="badge {translationResult.is_offline ? 'badge-offline' : 'badge-online'}">
              {translationResult.is_offline ? '⚡ Offline Lexicon' : '🌐 Cloud HTTP'}
            </span>
            <span class="latency-badge">{translationResult.latency_ms} ms</span>
          </div>
        {/if}
      </div>

      <div class="output-content">
        {#if isTranslating}
          <div class="translating-indicator">
            <div class="spinner"></div>
            <span>Translating via production engine...</span>
          </div>
        {:else if translationError}
          <div class="error-notice">
            <div class="error-icon">⚠️</div>
            <div class="error-body">
              <h4>Translation Issue</h4>
              <p>{translationError}</p>
              {#if !allowCloud}
                <button class="btn-primary-sm" on:click={() => { allowCloud = true; executeTranslation(); }}>
                  Enable Cloud Provider
                </button>
              {/if}
            </div>
          </div>
        {:else if translationResult}
          <div class="result-text indic-text">
            {translationResult.translated_text}
          </div>

          <!-- Clickable translated tokens -->
          <div class="token-chips">
            <span class="tokens-label">Dictionary drill-down:</span>
            {#each translationResult.translated_text.split(/\s+/) as word}
              <button
                class="token-chip accent-chip"
                on:click={() => drilldownWord(word, targetLang)}
                title="Click to view definition for '{word}'"
              >
                {word}
              </button>
            {/each}
          </div>
        {:else}
          <div class="placeholder-prompt">
            <span class="prompt-icon">📖</span>
            <p>Start typing or select a quick sample above.</p>
            <span class="prompt-sub">Real-time bilingual dictionary lookup and verified translation.</span>
          </div>
        {/if}
      </div>

      <div class="panel-footer">
        {#if translationResult}
          <div class="footer-actions">
            <button
              class="btn-secondary"
              on:click={() => copyText(translationResult?.translated_text ?? '')}
            >
              {copied ? 'Copied!' : '📋 Copy'}
            </button>
            <button
              class="btn-secondary"
              class:favorited={isFavorited}
              on:click={toggleFavorite}
              title={isFavorited ? 'Remove from Favorites' : 'Add to Favorites'}
            >
              {isFavorited ? '⭐ Saved' : '☆ Save'}
            </button>
          </div>
          <span class="engine-info">Engine: <code>{translationResult.engine_id}</code></span>
        {:else}
          <span class="engine-info">Stage 1 Active: Bilingual Lexicon + HTTP Provider</span>
        {/if}
      </div>
    </div>
  </div>

  <!-- Word Drill-down Card Modal -->
  {#if activeDrilldownEntry}
    <div class="drilldown-card glass-panel animate-fade">
      <div class="drilldown-header">
        <div class="word-title-group">
          <h3 class="indic-text">{activeDrilldownEntry.word}</h3>
          <span class="drilldown-pos">{activeDrilldownEntry.part_of_speech}</span>
          {#if activeDrilldownEntry.pronunciation_ipa}
            <span class="drilldown-ipa">{activeDrilldownEntry.pronunciation_ipa}</span>
          {/if}
          {#if activeDrilldownEntry.transliteration}
            <span class="drilldown-translit">{activeDrilldownEntry.transliteration}</span>
          {/if}
        </div>
        <button class="close-btn" on:click={() => (activeDrilldownEntry = null)}>×</button>
      </div>

      <div class="drilldown-body">
        <p class="definition-text"><strong>Definition:</strong> {activeDrilldownEntry.definition}</p>

        {#if activeDrilldownEntry.examples.length > 0}
          <div class="drilldown-section">
            <span class="section-title">Example:</span>
            <div class="example-quote indic-text">
              <p>"{activeDrilldownEntry.examples[0].source}"</p>
              <p class="example-trans">↳ {activeDrilldownEntry.examples[0].target}</p>
            </div>
          </div>
        {/if}

        {#if activeDrilldownEntry.synonyms.length > 0}
          <div class="drilldown-section">
            <span class="section-title">Synonyms:</span>
            <span class="synonym-list indic-text">{activeDrilldownEntry.synonyms.join(' • ')}</span>
          </div>
        {/if}

        <div class="provenance-tag">
          Provenance: {activeDrilldownEntry.provenance}
        </div>
      </div>
    </div>
  {/if}

  <!-- History & Favorites Drawer -->
  {#if showHistory}
    <div class="history-card glass-panel animate-fade">
      <div class="history-header">
        <h3>Translation History & Saved Words</h3>
        <div class="history-actions">
          <button class="btn-secondary-sm" on:click={async () => { await clearTranslationHistory(); await refreshHistoryAndFavorites(); }}>
            Clear History
          </button>
          <button class="close-btn" on:click={() => (showHistory = false)}>×</button>
        </div>
      </div>

      {#if historyList.length === 0}
        <p class="empty-state">No translations recorded yet.</p>
      {:else}
        <div class="history-list">
          {#each historyList as item}
            <button type="button" class="history-item" on:click={() => restoreHistory(item)}>
              <div class="history-meta">
                <span class="pair-tag">{item.source_language} ➔ {item.target_language}</span>
                <span class="engine-tag">{item.engine_used} ({item.latency_ms} ms)</span>
              </div>
              <div class="history-text indic-text">
                <strong>{item.source_text}</strong> ➔ {item.translated_text}
              </div>
            </button>
          {/each}
        </div>
      {/if}
    </div>
  {/if}
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

  .top-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-wrap: wrap;
    gap: 16px;
  }

  .lang-selector-group {
    display: flex;
    align-items: center;
    gap: 12px;
    flex: 1;
    min-width: 300px;
  }

  .lang-select {
    flex: 1;
    min-width: 160px;
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
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .swap-btn:hover {
    border-color: var(--text-accent);
    color: var(--text-accent);
    transform: rotate(180deg);
  }

  .mode-toggles {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .cloud-toggle {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 0.85rem;
    cursor: pointer;
    background: var(--bg-surface);
    padding: 6px 12px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-subtle);
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
    font-weight: 600;
  }

  .sample-chip {
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    color: var(--text-primary);
    padding: 6px 14px;
    border-radius: var(--radius-full);
    font-size: 0.85rem;
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .sample-chip:hover {
    background: var(--bg-elevated);
    border-color: var(--border-focus);
    color: var(--text-accent);
  }

  .panels-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 20px;
  }

  @media (max-width: 768px) {
    .panels-grid {
      grid-template-columns: 1fr;
    }
  }

  .panel-box {
    padding: 20px;
    display: flex;
    flex-direction: column;
    min-height: 280px;
  }

  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
  }

  .panel-tag {
    font-size: 0.85rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-muted);
  }

  .char-count {
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .text-area {
    flex: 1;
    resize: none;
    background: transparent;
    border: none;
    outline: none;
    font-size: 1.15rem;
    line-height: 1.6;
    color: var(--text-primary);
  }

  .token-chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    align-items: center;
    margin: 12px 0 6px 0;
    padding-top: 8px;
    border-top: 1px dashed var(--border-subtle);
  }

  .tokens-label {
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .token-chip {
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    color: var(--text-primary);
    padding: 2px 8px;
    border-radius: 4px;
    font-size: 0.82rem;
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .token-chip:hover {
    background: var(--bg-elevated);
    border-color: var(--border-focus);
    color: var(--text-accent);
  }

  .accent-chip {
    border-color: var(--text-accent);
    color: var(--text-accent);
  }

  .output-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    justify-content: flex-start;
  }

  .result-text {
    font-size: 1.25rem;
    line-height: 1.6;
    font-weight: 500;
    color: var(--text-primary);
    padding: 8px 0;
    min-height: 80px;
  }

  .badge-group {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .badge-offline {
    background: rgba(16, 185, 129, 0.15);
    color: #10b981;
    border: 1px solid rgba(16, 185, 129, 0.3);
  }

  .badge-online {
    background: rgba(59, 130, 246, 0.15);
    color: #3b82f6;
    border: 1px solid rgba(59, 130, 246, 0.3);
  }

  .latency-badge {
    font-size: 0.75rem;
    background: var(--bg-surface);
    padding: 2px 6px;
    border-radius: 4px;
    color: var(--text-muted);
  }

  .translating-indicator {
    display: flex;
    align-items: center;
    gap: 12px;
    color: var(--text-accent);
    font-size: 0.95rem;
    padding: 40px 0;
  }

  .spinner {
    width: 20px;
    height: 20px;
    border: 2px solid var(--border-subtle);
    border-top-color: var(--text-accent);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .error-notice {
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid rgba(239, 68, 68, 0.3);
    border-radius: var(--radius-sm);
    padding: 16px;
    display: flex;
    gap: 12px;
    margin: 12px 0;
  }

  .error-icon {
    font-size: 1.5rem;
  }

  .error-body h4 {
    margin: 0 0 4px 0;
    color: #ef4444;
  }

  .error-body p {
    margin: 0 0 10px 0;
    font-size: 0.9rem;
    color: var(--text-secondary);
  }

  .placeholder-prompt {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 40px 20px;
    color: var(--text-muted);
    text-align: center;
    gap: 8px;
  }

  .prompt-icon {
    font-size: 2rem;
    opacity: 0.6;
  }

  .prompt-sub {
    font-size: 0.8rem;
    opacity: 0.8;
  }

  .panel-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding-top: 12px;
    border-top: 1px solid var(--border-subtle);
    margin-top: 12px;
  }

  .footer-actions {
    display: flex;
    gap: 8px;
  }

  .favorited {
    color: #f59e0b !important;
    border-color: #f59e0b !important;
  }

  .engine-info {
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  /* Drill-down Card Modal */
  .drilldown-card {
    padding: 20px;
    border: 1px solid var(--text-accent);
    border-radius: var(--radius-md);
  }

  .drilldown-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 12px;
  }

  .word-title-group {
    display: flex;
    align-items: baseline;
    gap: 12px;
    flex-wrap: wrap;
  }

  .word-title-group h3 {
    margin: 0;
    font-size: 1.5rem;
    color: var(--text-accent);
  }

  .drilldown-pos {
    background: var(--bg-surface);
    padding: 2px 8px;
    border-radius: 4px;
    font-size: 0.8rem;
    color: var(--text-secondary);
  }

  .drilldown-ipa, .drilldown-translit {
    font-family: monospace;
    font-size: 0.85rem;
    color: var(--text-muted);
  }

  .close-btn {
    background: transparent;
    border: none;
    font-size: 1.5rem;
    cursor: pointer;
    color: var(--text-muted);
  }

  .close-btn:hover {
    color: var(--text-primary);
  }

  .definition-text {
    font-size: 1.05rem;
    margin: 0 0 12px 0;
  }

  .drilldown-section {
    margin-bottom: 10px;
    font-size: 0.95rem;
  }

  .section-title {
    font-weight: 600;
    color: var(--text-secondary);
  }

  .example-quote {
    background: var(--bg-surface);
    padding: 10px 14px;
    border-left: 3px solid var(--text-accent);
    border-radius: 0 4px 4px 0;
    margin-top: 4px;
  }

  .example-quote p {
    margin: 2px 0;
  }

  .example-trans {
    color: var(--text-secondary);
    font-size: 0.9rem;
  }

  .provenance-tag {
    font-size: 0.75rem;
    color: var(--text-muted);
    margin-top: 12px;
    padding-top: 8px;
    border-top: 1px dashed var(--border-subtle);
  }

  /* History Drawer */
  .history-card {
    padding: 20px;
  }

  .history-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
  }

  .history-actions {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .history-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
    max-height: 250px;
    overflow-y: auto;
  }

  .history-item {
    background: var(--bg-surface);
    padding: 10px 14px;
    border-radius: var(--radius-sm);
    cursor: pointer;
    border: 1px solid var(--border-subtle);
    transition: all var(--transition-fast);
    text-align: left;
    width: 100%;
  }

  .history-item:hover {
    border-color: var(--text-accent);
    background: var(--bg-elevated);
  }

  .history-meta {
    display: flex;
    justify-content: space-between;
    font-size: 0.75rem;
    color: var(--text-muted);
    margin-bottom: 4px;
  }

  .btn-primary-sm, .btn-secondary-sm {
    padding: 4px 10px;
    font-size: 0.8rem;
    border-radius: 4px;
    cursor: pointer;
  }

  .btn-primary-sm {
    background: var(--text-accent);
    color: #fff;
    border: none;
  }

  .btn-secondary-sm {
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    color: var(--text-primary);
  }

  .btn-icon {
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    color: var(--text-primary);
    padding: 6px 12px;
    border-radius: var(--radius-sm);
    cursor: pointer;
  }

  .btn-icon.active {
    border-color: var(--text-accent);
    color: var(--text-accent);
  }

  .animate-fade {
    animation: fadeIn 0.2s ease-in;
  }

  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(-4px); }
    to { opacity: 1; transform: translateY(0); }
  }
</style>
