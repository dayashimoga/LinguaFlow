<script lang="ts">
  import { onMount } from 'svelte';
  import { SUPPORTED_LANGUAGES, type LanguageCode, type DictionaryEntry } from '../lib/types';
  import { searchDictionary } from '../lib/ipc';

  let searchTerm = '';
  let selectedLang: LanguageCode | 'all' = 'all';
  let limit = 20;

  let isSearching = false;
  let searchResults: DictionaryEntry[] = [];
  let searchDurationMs = 0;
  let hasSearched = false;
  let searchError: string | null = null;

  let debounceTimer: ReturnType<typeof setTimeout> | null = null;

  onMount(() => {
    // Initial search to showcase seeded entries
    executeSearch('ನಮ');
  });

  function handleSearchInput() {
    if (debounceTimer) clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
      executeSearch(searchTerm);
    }, 200);
  }

  async function executeSearch(query: string) {
    const q = query.trim();
    if (!q) {
      searchResults = [];
      hasSearched = false;
      return;
    }

    isSearching = true;
    searchError = null;
    const start = performance.now();

    try {
      const lang = selectedLang === 'all' ? null : selectedLang;
      const res = await searchDictionary(q, lang, limit);
      searchResults = res.entries;
      searchDurationMs = Math.round(performance.now() - start);
      hasSearched = true;
    } catch (e: any) {
      searchError = e?.message || 'Dictionary search failed';
      searchResults = [];
    } finally {
      isSearching = false;
    }
  }

  function setQuickTerm(term: string, lang: LanguageCode | 'all') {
    searchTerm = term;
    selectedLang = lang;
    executeSearch(term);
  }
</script>

<div class="dictionary-view">
  <!-- Search Header Card -->
  <div class="search-card glass-panel">
    <div class="search-title-row">
      <h2>Multilingual Dictionary & FTS5 Glossary</h2>
      <span class="badge badge-active">SQLite FTS5 Verified</span>
    </div>
    <p class="search-desc">
      Search definitions, grammatical forms, IPA phonetics, and contextual examples across Kannada, Tamil, and English.
    </p>

    <div class="search-bar-row">
      <select bind:value={selectedLang} on:change={() => executeSearch(searchTerm)} class="lang-select">
        <option value="all">All Languages</option>
        {#each SUPPORTED_LANGUAGES as lang}
          <option value={lang.code}>{lang.native} ({lang.name})</option>
        {/each}
      </select>

      <div class="search-input-wrap">
        <input
          type="text"
          bind:value={searchTerm}
          on:input={handleSearchInput}
          placeholder="Search words, prefix or definition (e.g. ನಮಸ್ಕಾರ, வணக்கம், water, peace)..."
          class="search-input indic-text"
        />
        {#if searchTerm}
          <button class="clear-btn" on:click={() => { searchTerm = ''; searchResults = []; hasSearched = false; }}>×</button>
        {/if}
      </div>

      <button
        class="btn-primary"
        on:click={() => executeSearch(searchTerm)}
        disabled={!searchTerm.trim() || isSearching}
      >
        {isSearching ? 'Searching...' : 'Search'}
      </button>
    </div>

    <!-- Quick Seed Queries -->
    <div class="quick-chips-row">
      <span class="chips-label">Popular Entries:</span>
      <button class="chip" on:click={() => setQuickTerm('ನಮಸ್ಕಾರ', 'kn-IN')}>ನಮಸ್ಕಾರ</button>
      <button class="chip" on:click={() => setQuickTerm('வணக்கம்', 'ta-IN')}>வணக்கம்</button>
      <button class="chip" on:click={() => setQuickTerm('Water', 'en-US')}>Water</button>
      <button class="chip" on:click={() => setQuickTerm('ಶಾಂತಿ', 'kn-IN')}>ಶಾಂತಿ (Peace)</button>
      <button class="chip" on:click={() => setQuickTerm('அன்பு', 'ta-IN')}>அன்பு (Love)</button>
      <button class="chip" on:click={() => setQuickTerm('greeting', 'all')}>FTS: greeting</button>
    </div>
  </div>

  <!-- Search Results Meta -->
  {#if hasSearched}
    <div class="results-stats">
      <span>Found <strong>{searchResults.length}</strong> matching entries in <strong>{searchDurationMs} ms</strong></span>
      <span class="stat-engine">Engine: SQLite FTS5 Index</span>
    </div>
  {/if}

  <!-- Error display -->
  {#if searchError}
    <div class="error-notice glass-panel">
      <span>⚠️ {searchError}</span>
    </div>
  {/if}

  <!-- Results Cards List -->
  {#if searchResults.length > 0}
    <div class="entries-list">
      {#each searchResults as entry}
        <div class="entry-card glass-panel">
          <div class="entry-header">
            <div class="entry-word-group">
              <h3 class="indic-text">{entry.word}</h3>
              <span class="lang-pill">{entry.language}</span>
              <span class="pos-pill">{entry.part_of_speech}</span>
            </div>

            <div class="phonetics-group">
              {#if entry.pronunciation_ipa}
                <span class="ipa-pill" title="International Phonetic Alphabet">
                  {entry.pronunciation_ipa}
                </span>
              {/if}
              {#if entry.transliteration}
                <span class="translit-pill" title="Latin Transliteration">
                  {entry.transliteration}
                </span>
              {/if}
            </div>
          </div>

          <p class="entry-definition">{entry.definition}</p>

          {#if entry.alternate_meanings && entry.alternate_meanings.length > 0}
            <div class="entry-section">
              <span class="section-label">Alternate meanings:</span>
              <span class="alt-meanings">{entry.alternate_meanings.join(', ')}</span>
            </div>
          {/if}

          {#if entry.examples && entry.examples.length > 0}
            <div class="entry-section">
              <span class="section-label">Contextual Example:</span>
              <div class="example-box indic-text">
                <p class="ex-source">"{entry.examples[0].source}"</p>
                <p class="ex-target">↳ {entry.examples[0].target}</p>
              </div>
            </div>
          {/if}

          {#if entry.synonyms && entry.synonyms.length > 0}
            <div class="entry-section">
              <span class="section-label">Synonyms:</span>
              <span class="tag-row indic-text">
                {#each entry.synonyms as syn}
                  <span class="syn-tag">{syn}</span>
                {/each}
              </span>
            </div>
          {/if}

          {#if entry.antonyms && entry.antonyms.length > 0}
            <div class="entry-section">
              <span class="section-label">Antonyms:</span>
              <span class="tag-row indic-text">
                {#each entry.antonyms as ant}
                  <span class="ant-tag">{ant}</span>
                {/each}
              </span>
            </div>
          {/if}

          <div class="entry-footer">
            <span class="provenance-text">Provenance: {entry.provenance}</span>
            <span class="entry-id">ID: <code>{entry.id}</code></span>
          </div>
        </div>
      {/each}
    </div>
  {:else if hasSearched && !isSearching}
    <div class="empty-results glass-panel">
      <span class="empty-icon">🔍</span>
      <h4>No dictionary entries found for "{searchTerm}"</h4>
      <p>Try searching by root prefix, transliteration, or definition keywords.</p>
    </div>
  {/if}
</div>

<style>
  .dictionary-view {
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  .search-card {
    padding: 24px;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .search-title-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-wrap: wrap;
    gap: 10px;
  }

  .search-title-row h2 {
    margin: 0;
  }

  .search-desc {
    color: var(--text-secondary);
    font-size: 0.95rem;
    margin: 0;
  }

  .search-bar-row {
    display: flex;
    gap: 12px;
    flex-wrap: wrap;
  }

  .lang-select {
    min-width: 170px;
    font-weight: 600;
  }

  .search-input-wrap {
    flex: 1;
    position: relative;
    min-width: 250px;
  }

  .search-input {
    width: 100%;
    padding-right: 36px;
    font-size: 1.05rem;
  }

  .clear-btn {
    position: absolute;
    right: 10px;
    top: 50%;
    transform: translateY(-50%);
    background: transparent;
    border: none;
    font-size: 1.25rem;
    color: var(--text-muted);
    cursor: pointer;
  }

  .quick-chips-row {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
    font-size: 0.85rem;
  }

  .chips-label {
    color: var(--text-muted);
    font-weight: 600;
  }

  .chip {
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    color: var(--text-primary);
    padding: 4px 12px;
    border-radius: var(--radius-full);
    font-size: 0.82rem;
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .chip:hover {
    border-color: var(--text-accent);
    color: var(--text-accent);
    background: var(--bg-elevated);
  }

  .results-stats {
    display: flex;
    justify-content: space-between;
    font-size: 0.85rem;
    color: var(--text-muted);
    padding: 0 4px;
  }

  .entries-list {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .entry-card {
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    border: 1px solid var(--border-subtle);
    transition: all var(--transition-fast);
  }

  .entry-card:hover {
    border-color: var(--text-accent);
  }

  .entry-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    flex-wrap: wrap;
    gap: 12px;
  }

  .entry-word-group {
    display: flex;
    align-items: baseline;
    gap: 10px;
    flex-wrap: wrap;
  }

  .entry-word-group h3 {
    margin: 0;
    font-size: 1.6rem;
    color: var(--text-accent);
  }

  .lang-pill {
    background: rgba(59, 130, 246, 0.15);
    color: #3b82f6;
    padding: 2px 8px;
    border-radius: 4px;
    font-size: 0.75rem;
    font-weight: 600;
  }

  .pos-pill {
    background: var(--bg-surface);
    padding: 2px 8px;
    border-radius: 4px;
    font-size: 0.8rem;
    color: var(--text-secondary);
    font-style: italic;
  }

  .phonetics-group {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .ipa-pill, .translit-pill {
    font-family: monospace;
    font-size: 0.85rem;
    background: var(--bg-surface);
    padding: 2px 8px;
    border-radius: 4px;
    color: var(--text-muted);
    border: 1px solid var(--border-subtle);
  }

  .entry-definition {
    font-size: 1.1rem;
    line-height: 1.6;
    margin: 0;
  }

  .entry-section {
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-size: 0.95rem;
  }

  .section-label {
    font-size: 0.8rem;
    font-weight: 700;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .example-box {
    background: var(--bg-surface);
    padding: 12px 16px;
    border-left: 3px solid var(--text-accent);
    border-radius: 0 4px 4px 0;
  }

  .example-box p {
    margin: 2px 0;
  }

  .ex-target {
    color: var(--text-secondary);
    font-size: 0.92rem;
  }

  .tag-row {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
  }

  .syn-tag {
    background: rgba(16, 185, 129, 0.1);
    color: #10b981;
    padding: 2px 8px;
    border-radius: 4px;
    font-size: 0.85rem;
  }

  .ant-tag {
    background: rgba(239, 68, 68, 0.1);
    color: #ef4444;
    padding: 2px 8px;
    border-radius: 4px;
    font-size: 0.85rem;
  }

  .entry-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 0.75rem;
    color: var(--text-muted);
    padding-top: 10px;
    border-top: 1px dashed var(--border-subtle);
    margin-top: 4px;
  }

  .empty-results {
    padding: 40px;
    text-align: center;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    color: var(--text-muted);
  }

  .empty-icon {
    font-size: 2.5rem;
    opacity: 0.6;
  }

  .empty-results h4 {
    margin: 0;
    color: var(--text-primary);
  }

  .error-notice {
    padding: 14px;
    color: #ef4444;
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid rgba(239, 68, 68, 0.3);
  }
</style>
