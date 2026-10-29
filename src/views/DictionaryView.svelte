<script lang="ts">
  import { onMount } from 'svelte';
  import { SUPPORTED_LANGUAGES, type LanguageCode, type DictionaryEntry, type RichDictionaryEntry } from '../lib/types';
  import { searchDictionary, addFavorite, listFavorites, removeFavorite, synthesizeSpeech } from '../lib/ipc';

  let searchTerm = '';
  let selectedLang: string = 'all';
  let selectedPos: string = 'all';
  let limit = 25;

  let isSearching = false;
  let searchResults: DictionaryEntry[] = [];
  let richResults: RichDictionaryEntry[] = [];
  let resolvedStem: string | null = null;
  let didYouMean: string[] = [];
  let searchDurationMs = 0;
  let hasSearched = false;
  let searchError: string | null = null;

  let favoritesMap: Record<string, string> = {}; // word -> favorite_id
  let playingAudioWord: string | null = null;
  let copiedWord: string | null = null;
  let dictSpeed = 1.0;
  let activeAudioContext: AudioContext | null = null;
  let activeAudioSource: AudioBufferSourceNode | null = null;

  let debounceTimer: ReturnType<typeof setTimeout> | null = null;

  const POS_FILTER_PILLS = [
    { id: 'all', label: 'All' },
    { id: 'noun', label: 'Nouns' },
    { id: 'verb', label: 'Verbs' },
    { id: 'adjective', label: 'Adjectives' },
    { id: 'medical', label: 'Medical' },
    { id: 'logistics', label: 'Logistics' },
  ];

  onMount(async () => {
    await loadFavorites();
    // Default search for common entry to demonstrate FTS5 instant lookup
    executeSearch('adventure');
  });

  async function loadFavorites() {
    try {
      const favs = await listFavorites();
      const map: Record<string, string> = {};
      for (const f of favs) {
        map[f.source_text.toLowerCase()] = f.id;
      }
      favoritesMap = map;
    } catch {
      // Ignore initial favorite fetch errors
    }
  }

  function handleSearchInput() {
    if (debounceTimer) clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
      executeSearch(searchTerm);
    }, 120); // 120ms debounced reactive search (Subphase 9.3.1)
  }

  async function executeSearch(query: string) {
    const q = query.trim();
    if (!q) {
      searchResults = [];
      richResults = [];
      resolvedStem = null;
      didYouMean = [];
      hasSearched = false;
      return;
    }

    isSearching = true;
    searchError = null;
    const start = performance.now();

    try {
      // Determine language filter parameter
      let langParam: string | null = null;
      if (selectedLang !== 'all') {
        if (selectedLang.startsWith('en')) langParam = 'en-US';
        else if (selectedLang.startsWith('kn')) langParam = 'kn-IN';
        else if (selectedLang.startsWith('ta')) langParam = 'ta-IN';
        else langParam = selectedLang;
      }

      const res = await searchDictionary(q, langParam, limit, selectedPos === 'all' ? null : selectedPos);
      searchResults = res.entries || [];
      richResults = res.rich_entries || [];
      resolvedStem = res.resolved_stem || null;
      didYouMean = res.did_you_mean || [];
      searchDurationMs = res.latency_ms || Math.round(performance.now() - start);
      hasSearched = true;
    } catch (e: any) {
      searchError = e?.message || 'Dictionary search failed';
      searchResults = [];
      richResults = [];
    } finally {
      isSearching = false;
    }
  }

  function setPosFilter(pos: string) {
    selectedPos = pos;
    executeSearch(searchTerm);
  }

  function setLanguageFilter(lang: string) {
    selectedLang = lang;
    executeSearch(searchTerm);
  }

  function drillDownWord(term: string) {
    searchTerm = term;
    executeSearch(term);
  }

  async function toggleFavorite(entry: RichDictionaryEntry | DictionaryEntry) {
    const wordKey = ('headword' in entry ? entry.headword : entry.word).toLowerCase();
    const existingId = favoritesMap[wordKey];

    if (existingId) {
      try {
        await removeFavorite(existingId);
        delete favoritesMap[wordKey];
        favoritesMap = { ...favoritesMap };
      } catch (err) {
        console.error('Failed to remove favorite:', err);
      }
    } else {
      const headword = 'headword' in entry ? entry.headword : entry.word;
      const def = 'headword' in entry 
        ? (entry.senses?.[0]?.definition || headword) 
        : entry.definition;
      const lang = entry.language;
      const tgtLang = lang === 'en-US' ? 'kn-IN' : 'en-US';

      try {
        const fav = await addFavorite({
          source_language: lang,
          target_language: tgtLang,
          source_text: headword,
          translated_text: def,
          notes: `POS: ${entry.part_of_speech}`,
        });
        favoritesMap[wordKey] = fav.id;
        favoritesMap = { ...favoritesMap };
      } catch (err) {
        console.error('Failed to add favorite:', err);
      }
    }
  }

  function copyHeadword(text: string) {
    navigator.clipboard.writeText(text);
    copiedWord = text;
    setTimeout(() => {
      copiedWord = null;
    }, 2000);
  }

  async function playSpeech(text: string, lang: LanguageCode) {
    if (activeAudioSource) {
      try { activeAudioSource.stop(); } catch (_) {}
      activeAudioSource = null;
    }
    if (activeAudioContext) {
      try { activeAudioContext.close(); } catch (_) {}
      activeAudioContext = null;
    }
    if (playingAudioWord === text) {
      playingAudioWord = null;
      return;
    }

    playingAudioWord = text;

    try {
      const resp = await synthesizeSpeech({
        text,
        language: lang,
        speed: dictSpeed,
        pitch: 1.0,
      });

      if (resp && resp.audio_samples && resp.audio_samples.length > 0) {
        const AudioCtx = window.AudioContext || (window as any).webkitAudioContext;
        activeAudioContext = new AudioCtx({ sampleRate: resp.sample_rate || 16000 });
        const buffer = activeAudioContext.createBuffer(1, resp.audio_samples.length, resp.sample_rate || 16000);
        const channelData = buffer.getChannelData(0);
        for (let i = 0; i < resp.audio_samples.length; i++) {
          const s = resp.audio_samples[i];
          channelData[i] = Math.abs(s) > 1.0 ? s / 32768.0 : s;
        }

        activeAudioSource = activeAudioContext.createBufferSource();
        activeAudioSource.buffer = buffer;
        activeAudioSource.connect(activeAudioContext.destination);
        activeAudioSource.onended = () => {
          playingAudioWord = null;
        };
        activeAudioSource.start();
        return;
      }
    } catch (e) {
      console.warn('Neural TTS synthesis fallback to browser Web Speech API:', e);
    }

    // Fallback to browser SpeechSynthesis
    if ('speechSynthesis' in window) {
      window.speechSynthesis.cancel();
      const utterance = new SpeechSynthesisUtterance(text);
      if (lang === 'kn-IN') utterance.lang = 'kn-IN';
      else if (lang === 'ta-IN') utterance.lang = 'ta-IN';
      else utterance.lang = 'en-US';
      utterance.rate = dictSpeed;
      utterance.onend = () => { playingAudioWord = null; };
      utterance.onerror = () => { playingAudioWord = null; };
      window.speechSynthesis.speak(utterance);
    } else {
      playingAudioWord = null;
    }
  }
</script>

<div class="dictionary-view">
  <!-- Search Header Card -->
  <div class="search-card glass-panel">
    <div class="search-title-row">
      <div>
        <h2>Multilingual Dictionary & Rich Lexicon</h2>
        <p class="search-desc">
          High-precision SQLite FTS5 search with agglutinative Indic morphology across Kannada, Tamil, and English.
        </p>
      </div>
      <div class="header-badges">
        <span class="badge badge-active">SQLite FTS5 unicode61</span>
        <span class="badge badge-verified">BM25 Ranked</span>
      </div>
    </div>

    <!-- Search Controls Bar -->
    <div class="search-bar-row">
      <div class="lang-select-wrap">
        <select bind:value={selectedLang} on:change={() => setLanguageFilter(selectedLang)} class="lang-select">
          <option value="all">🌐 All Languages</option>
          <option value="en-kn">English → Kannada</option>
          <option value="en-ta">English → Tamil</option>
          <option value="kn-en">Kannada → English</option>
          <option value="ta-en">Tamil → English</option>
          <option value="kn-ta">Kannada ↔ Tamil</option>
          <option value="en-US">English (en-US)</option>
          <option value="kn-IN">ಕನ್ನಡ — Kannada (kn-IN)</option>
          <option value="ta-IN">தமிழ் — Tamil (ta-IN)</option>
        </select>
      </div>

      <div class="search-input-wrap">
        <span class="search-icon">🔍</span>
        <input
          type="text"
          bind:value={searchTerm}
          on:input={handleSearchInput}
          placeholder="Search headword, stem, or definition (e.g., adventure, ಸಾಹಸ, சாகசம், doctor, ಸರಕು)..."
          class="search-input indic-text"
        />
        {#if searchTerm}
          <button class="clear-btn" on:click={() => { searchTerm = ''; searchResults = []; richResults = []; hasSearched = false; }}>✕</button>
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

    <!-- Part-of-Speech Filter Pills -->
    <div class="filter-pills-row">
      <span class="filter-label">Filter Category:</span>
      <div class="pills-container">
        {#each POS_FILTER_PILLS as pill}
          <button
            class="pos-pill-btn {selectedPos === pill.id ? 'active' : ''}"
            on:click={() => setPosFilter(pill.id)}
          >
            {pill.label}
          </button>
        {/each}
      </div>
    </div>

    <!-- Quick Entry Chips -->
    <div class="quick-chips-row">
      <span class="chips-label">Popular Queries:</span>
      <button class="chip" on:click={() => drillDownWord('adventure')}>adventure</button>
      <button class="chip" on:click={() => drillDownWord('ಸಾಹಸ')}>ಸಾಹಸ (Adventure)</button>
      <button class="chip" on:click={() => drillDownWord('சாகசம்')}>சாகசம் (Adventure)</button>
      <button class="chip" on:click={() => drillDownWord('doctor')}>doctor</button>
      <button class="chip" on:click={() => drillDownWord('ವೈದ್ಯ')}>ವೈದ್ಯ (Doctor)</button>
      <button class="chip" on:click={() => drillDownWord('freight')}>freight</button>
      <button class="chip" on:click={() => drillDownWord('ಸರಕು')}>ಸರಕು (Cargo)</button>
      <button class="chip" on:click={() => drillDownWord('peace')}>peace</button>
      <button class="chip" on:click={() => drillDownWord('water')}>water</button>
    </div>
  </div>

  <!-- Search Results Meta & Banners -->
  {#if hasSearched}
    <div class="results-stats">
      <span>
        Found <strong>{richResults.length || searchResults.length}</strong> matching entry(ies) in <strong>{searchDurationMs} ms</strong>
      </span>
      <span class="stat-engine">BM25 Column Weighting (10.0, 5.0, 2.0, 1.0)</span>
    </div>

    <!-- Morphological Stemming Resolution Banner (Subphase 9.2.2) -->
    {#if resolvedStem}
      <div class="morph-banner glass-panel">
        <span class="morph-icon">🌿</span>
        <span>
          Agglutinative inflection detected: <strong>"{searchTerm}"</strong> resolved to base stem <strong>"{resolvedStem}"</strong>
        </span>
      </div>
    {/if}

    <!-- Did You Mean Fallback Suggestions (Subphase 9.2.3) -->
    {#if didYouMean.length > 0}
      <div class="did-you-mean-banner glass-panel">
        <span class="dym-icon">💡</span>
        <span>Did you mean:</span>
        <div class="dym-chips">
          {#each didYouMean as suggestion}
            <button class="dym-chip" on:click={() => drillDownWord(suggestion)}>
              {suggestion}
            </button>
          {/each}
        </div>
      </div>
    {/if}
  {/if}

  <!-- Error display -->
  {#if searchError}
    <div class="error-notice glass-panel">
      <span>⚠️ {searchError}</span>
    </div>
  {/if}

  <!-- Rich Lexicon Entries List -->
  {#if richResults.length > 0}
    <div class="entries-list">
      {#each richResults as entry}
        <div class="entry-card glass-panel">
          <div class="entry-header">
            <div class="entry-word-group">
              <h3 class="indic-text headword-text">{entry.headword}</h3>
              <span class="lang-pill">{entry.language}</span>
              <span class="pos-pill">{entry.part_of_speech}</span>
              {#if entry.register && entry.register !== 'general'}
                <span class="register-pill">{entry.register}</span>
              {/if}
            </div>

            <div class="entry-actions">
              <!-- Audio Speech Button & Speed Chips -->
              <div class="audio-action-group">
                <button
                  class="action-btn {playingAudioWord === entry.headword ? 'active-audio' : ''}"
                  on:click={() => playSpeech(entry.headword, entry.language)}
                  title="Listen to authentic pronunciation"
                >
                  {playingAudioWord === entry.headword ? '🔊 Playing...' : '🔊 Listen'}
                </button>
                <div class="dict-speed-chips" title="Playback Speed">
                  {#each [0.75, 1.0, 1.25] as speed}
                    <button
                      class="dict-speed-chip {dictSpeed === speed ? 'active' : ''}"
                      on:click={() => (dictSpeed = speed)}
                    >
                      {speed}x
                    </button>
                  {/each}
                </div>
              </div>

              <!-- Copy Button -->
              <button
                class="action-btn"
                on:click={() => copyHeadword(entry.headword)}
                title="Copy headword to clipboard"
              >
                {copiedWord === entry.headword ? '✓ Copied' : '📋 Copy'}
              </button>

              <!-- Favorite Toggle Button -->
              <button
                class="action-btn fav-btn {favoritesMap[entry.headword.toLowerCase()] ? 'is-fav' : ''}"
                on:click={() => toggleFavorite(entry)}
                title="Save to flashcard favorites (SM-2)"
              >
                {favoritesMap[entry.headword.toLowerCase()] ? '★ Saved' : '☆ Save'}
              </button>
            </div>
          </div>

          <!-- Phonetics & Transliteration Row -->
          <div class="phonetics-group">
            {#if entry.ipa_pronunciation}
              <span class="ipa-pill" title="International Phonetic Alphabet">
                IPA: <code>{entry.ipa_pronunciation}</code>
              </span>
            {/if}
            {#if entry.native_phonetic_respelling}
              <span class="translit-pill" title="ISO-15919 Romanized Transliteration">
                Phonetic: <strong>{entry.native_phonetic_respelling}</strong>
              </span>
            {/if}
          </div>

          <!-- Cross-Lingual Target Equivalents (Subphase 9.3.2) -->
          {#if Object.keys(entry.target_equivalents).length > 0}
            <div class="equivalents-box">
              <span class="box-title">Cross-Language Target Equivalents:</span>
              <div class="equiv-grid">
                {#if entry.target_equivalents['kn-IN']}
                  <div class="equiv-row">
                    <span class="equiv-lang">ಕನ್ನಡ (kn-IN):</span>
                    <div class="equiv-chips indic-text">
                      {#each entry.target_equivalents['kn-IN'] as eq}
                        <button class="equiv-chip" on:click={() => drillDownWord(eq)}>{eq}</button>
                      {/each}
                    </div>
                  </div>
                {/if}
                {#if entry.target_equivalents['ta-IN']}
                  <div class="equiv-row">
                    <span class="equiv-lang">தமிழ் (ta-IN):</span>
                    <div class="equiv-chips indic-text">
                      {#each entry.target_equivalents['ta-IN'] as eq}
                        <button class="equiv-chip" on:click={() => drillDownWord(eq)}>{eq}</button>
                      {/each}
                    </div>
                  </div>
                {/if}
                {#if entry.target_equivalents['en-US']}
                  <div class="equiv-row">
                    <span class="equiv-lang">English (en-US):</span>
                    <div class="equiv-chips">
                      {#each entry.target_equivalents['en-US'] as eq}
                        <button class="equiv-chip" on:click={() => drillDownWord(eq)}>{eq}</button>
                      {/each}
                    </div>
                  </div>
                {/if}
              </div>
            </div>
          {/if}

          <!-- Numbered Senses Grouped by POS -->
          {#if entry.senses && entry.senses.length > 0}
            <div class="senses-container">
              <span class="box-title">Definitions & Senses:</span>
              {#each entry.senses as sense}
                <div class="sense-item">
                  <span class="sense-num">{sense.sense_index}.</span>
                  <div class="sense-body">
                    <span class="sense-pos">[{sense.part_of_speech}]</span>
                    <span class="sense-def indic-text">{sense.definition}</span>
                  </div>
                </div>
              {/each}
            </div>
          {/if}

          <!-- Contextual Bilingual Examples -->
          {#if entry.bilingual_examples && entry.bilingual_examples.length > 0}
            <div class="entry-section">
              <span class="section-label">Contextual Examples:</span>
              <div class="examples-list">
                {#each entry.bilingual_examples as ex}
                  <div class="example-box indic-text">
                    <div class="example-content">
                      <p class="ex-source">"{ex.source}"</p>
                      <p class="ex-target">↳ {ex.target}</p>
                    </div>
                    <button
                      class="mini-audio-btn {playingAudioWord === ex.source ? 'active-audio' : ''}"
                      on:click={() => playSpeech(ex.source, entry.language)}
                      title="Listen to example sentence"
                    >
                      🔊
                    </button>
                  </div>
                {/each}
              </div>
            </div>
          {/if}

          <!-- Synonyms & Antonyms with Drill-Down Chips -->
          {#if entry.synonyms && entry.synonyms.length > 0}
            <div class="entry-section">
              <span class="section-label">Synonyms:</span>
              <div class="tag-row indic-text">
                {#each entry.synonyms as syn}
                  <button class="syn-chip" on:click={() => drillDownWord(syn)}>{syn}</button>
                {/each}
              </div>
            </div>
          {/if}

          {#if entry.antonyms && entry.antonyms.length > 0}
            <div class="entry-section">
              <span class="section-label">Antonyms:</span>
              <div class="tag-row indic-text">
                {#each entry.antonyms as ant}
                  <button class="ant-chip" on:click={() => drillDownWord(ant)}>{ant}</button>
                {/each}
              </div>
            </div>
          {/if}

          <!-- Derived Forms -->
          {#if entry.derived_forms && entry.derived_forms.length > 0}
            <div class="entry-section">
              <span class="section-label">Derived Forms & Inflections:</span>
              <div class="tag-row indic-text">
                {#each entry.derived_forms as df}
                  <button class="derived-chip" on:click={() => drillDownWord(df)}>{df}</button>
                {/each}
              </div>
            </div>
          {/if}

          <!-- Provenance Footer -->
          <div class="entry-footer">
            <span class="provenance-text">Provenance: {entry.provenance}</span>
            <span class="entry-id">ID: <code>{entry.id}</code></span>
          </div>
        </div>
      {/each}
    </div>
  {:else if searchResults.length > 0}
    <!-- Legacy Fallback Rendering -->
    <div class="entries-list">
      {#each searchResults as entry}
        <div class="entry-card glass-panel">
          <div class="entry-header">
            <div class="entry-word-group">
              <h3 class="indic-text headword-text">{entry.word}</h3>
              <span class="lang-pill">{entry.language}</span>
              <span class="pos-pill">{entry.part_of_speech}</span>
            </div>
          </div>
          <p class="entry-definition indic-text">{entry.definition}</p>
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
    gap: 16px;
  }

  .search-title-row {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    flex-wrap: wrap;
    gap: 12px;
  }

  .search-title-row h2 {
    margin: 0 0 4px 0;
  }

  .search-desc {
    color: var(--text-secondary);
    font-size: 0.95rem;
    margin: 0;
  }

  .header-badges {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }

  .badge-active {
    background: rgba(36, 200, 219, 0.15);
    color: var(--primary-color, #24c8db);
    border: 1px solid rgba(36, 200, 219, 0.3);
    padding: 4px 10px;
    border-radius: 12px;
    font-size: 0.8rem;
    font-weight: 600;
  }

  .badge-verified {
    background: rgba(46, 204, 113, 0.15);
    color: #2ecc71;
    border: 1px solid rgba(46, 204, 113, 0.3);
    padding: 4px 10px;
    border-radius: 12px;
    font-size: 0.8rem;
    font-weight: 600;
  }

  .search-bar-row {
    display: flex;
    gap: 12px;
    flex-wrap: wrap;
    align-items: center;
  }

  .lang-select-wrap {
    min-width: 190px;
  }

  .lang-select {
    width: 100%;
    padding: 10px 14px;
    border-radius: 8px;
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid var(--border-color);
    color: var(--text-primary);
    font-size: 0.95rem;
    outline: none;
    cursor: pointer;
  }

  .search-input-wrap {
    flex: 1;
    min-width: 280px;
    position: relative;
    display: flex;
    align-items: center;
  }

  .search-icon {
    position: absolute;
    left: 12px;
    color: var(--text-secondary);
    pointer-events: none;
  }

  .search-input {
    width: 100%;
    padding: 10px 36px 10px 36px;
    border-radius: 8px;
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid var(--border-color);
    color: var(--text-primary);
    font-size: 1rem;
    outline: none;
    transition: border-color 0.15s ease;
  }

  .search-input:focus {
    border-color: var(--primary-color, #24c8db);
    box-shadow: 0 0 0 2px rgba(36, 200, 219, 0.2);
  }

  .clear-btn {
    position: absolute;
    right: 12px;
    background: none;
    border: none;
    color: var(--text-secondary);
    font-size: 1.1rem;
    cursor: pointer;
  }

  .btn-primary {
    padding: 10px 22px;
    border-radius: 8px;
    background: var(--primary-color, #24c8db);
    color: #121820;
    font-weight: 600;
    border: none;
    cursor: pointer;
    transition: opacity 0.15s ease;
  }

  .btn-primary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  /* Filter Pills */
  .filter-pills-row {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-wrap: wrap;
  }

  .filter-label, .chips-label {
    font-size: 0.85rem;
    color: var(--text-secondary);
    font-weight: 600;
  }

  .pills-container {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }

  .pos-pill-btn {
    padding: 4px 12px;
    border-radius: 16px;
    font-size: 0.82rem;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid var(--border-color);
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .pos-pill-btn.active {
    background: rgba(36, 200, 219, 0.2);
    border-color: var(--primary-color, #24c8db);
    color: var(--primary-color, #24c8db);
    font-weight: 600;
  }

  .quick-chips-row {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
    padding-top: 4px;
    border-top: 1px solid rgba(255, 255, 255, 0.06);
  }

  .chip {
    padding: 3px 10px;
    border-radius: 12px;
    font-size: 0.82rem;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid var(--border-color);
    color: var(--text-secondary);
    cursor: pointer;
  }

  .chip:hover {
    background: rgba(255, 255, 255, 0.1);
    color: var(--text-primary);
  }

  /* Meta & Banners */
  .results-stats {
    display: flex;
    justify-content: space-between;
    font-size: 0.9rem;
    color: var(--text-secondary);
    padding: 0 4px;
  }

  .morph-banner, .did-you-mean-banner {
    padding: 12px 18px;
    display: flex;
    align-items: center;
    gap: 12px;
    border-radius: 8px;
    font-size: 0.92rem;
  }

  .morph-banner {
    border-left: 4px solid #2ecc71;
    background: rgba(46, 204, 113, 0.08);
  }

  .did-you-mean-banner {
    border-left: 4px solid #f39c12;
    background: rgba(243, 156, 18, 0.08);
  }

  .dym-chips {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }

  .dym-chip {
    padding: 3px 10px;
    border-radius: 12px;
    background: rgba(243, 156, 18, 0.2);
    border: 1px solid #f39c12;
    color: #f39c12;
    font-weight: 600;
    cursor: pointer;
  }

  /* Entries List & Cards */
  .entries-list {
    display: flex;
    flex-direction: column;
    gap: 18px;
  }

  .entry-card {
    padding: 22px;
    display: flex;
    flex-direction: column;
    gap: 14px;
    border-radius: 10px;
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
    align-items: center;
    gap: 10px;
    flex-wrap: wrap;
  }

  .headword-text {
    margin: 0;
    font-size: 1.6rem;
    font-weight: 700;
    color: var(--text-primary);
  }

  .lang-pill {
    padding: 3px 8px;
    border-radius: 6px;
    font-size: 0.78rem;
    background: rgba(255, 255, 255, 0.08);
    font-weight: 600;
  }

  .pos-pill {
    padding: 3px 8px;
    border-radius: 6px;
    font-size: 0.78rem;
    background: rgba(52, 152, 219, 0.15);
    color: #3498db;
    font-weight: 600;
  }

  .register-pill {
    padding: 3px 8px;
    border-radius: 6px;
    font-size: 0.78rem;
    background: rgba(155, 89, 182, 0.15);
    color: #9b59b6;
    font-weight: 600;
  }

  .entry-actions {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }

  .audio-action-group {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid var(--border-color);
    border-radius: 6px;
    padding: 2px;
  }

  .dict-speed-chips {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    padding-right: 4px;
  }

  .dict-speed-chip {
    background: transparent;
    border: none;
    color: var(--text-secondary);
    font-size: 0.70rem;
    padding: 2px 4px;
    border-radius: 3px;
    cursor: pointer;
    font-weight: 500;
    transition: all 0.15s ease;
  }

  .dict-speed-chip:hover {
    color: var(--text-primary);
  }

  .dict-speed-chip.active {
    background: var(--primary-color, #24c8db);
    color: #0f172a;
    font-weight: 700;
  }

  .action-btn {
    padding: 5px 12px;
    border-radius: 6px;
    font-size: 0.82rem;
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid var(--border-color);
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .action-btn:hover {
    background: rgba(255, 255, 255, 0.12);
    color: var(--text-primary);
  }

  .action-btn.active-audio {
    border-color: #2ecc71;
    color: #2ecc71;
    background: rgba(46, 204, 113, 0.1);
  }

  .fav-btn.is-fav {
    background: rgba(241, 196, 15, 0.15);
    border-color: #f1c40f;
    color: #f1c40f;
    font-weight: 600;
  }

  .phonetics-group {
    display: flex;
    gap: 12px;
    flex-wrap: wrap;
  }

  .ipa-pill, .translit-pill {
    font-size: 0.88rem;
    color: var(--text-secondary);
  }

  .ipa-pill code {
    color: #e67e22;
  }

  /* Cross-Lingual Equivalents */
  .equivalents-box {
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    padding: 12px 16px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .box-title {
    font-size: 0.85rem;
    color: var(--text-secondary);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .equiv-grid {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .equiv-row {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-wrap: wrap;
  }

  .equiv-lang {
    font-size: 0.88rem;
    font-weight: 600;
    min-width: 120px;
    color: var(--text-secondary);
  }

  .equiv-chips {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
  }

  .equiv-chip {
    padding: 3px 10px;
    border-radius: 6px;
    font-size: 0.95rem;
    background: rgba(36, 200, 219, 0.1);
    border: 1px solid rgba(36, 200, 219, 0.25);
    color: var(--text-primary);
    cursor: pointer;
    font-weight: 600;
  }

  .equiv-chip:hover {
    background: rgba(36, 200, 219, 0.2);
    border-color: var(--primary-color, #24c8db);
  }

  /* Senses Container */
  .senses-container {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .sense-item {
    display: flex;
    gap: 8px;
    align-items: baseline;
    line-height: 1.5;
  }

  .sense-num {
    font-weight: 700;
    color: var(--text-secondary);
    min-width: 18px;
  }

  .sense-pos {
    color: #3498db;
    font-style: italic;
    font-size: 0.88rem;
    margin-right: 6px;
  }

  .sense-def {
    font-size: 1rem;
    color: var(--text-primary);
  }

  /* Examples & Chips */
  .entry-section {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .section-label {
    font-size: 0.85rem;
    color: var(--text-secondary);
    font-weight: 600;
  }

  .example-box {
    background: rgba(0, 0, 0, 0.15);
    padding: 10px 14px;
    border-radius: 6px;
    border-left: 3px solid var(--primary-color, #24c8db);
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }

  .example-content {
    flex: 1;
  }

  .mini-audio-btn {
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid var(--border-color);
    border-radius: 6px;
    padding: 6px 10px;
    font-size: 0.85rem;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .mini-audio-btn:hover {
    background: rgba(255, 255, 255, 0.14);
    transform: scale(1.05);
  }

  .ex-source {
    margin: 0 0 4px 0;
    color: var(--text-primary);
  }

  .ex-target {
    margin: 0;
    color: var(--text-secondary);
  }

  .tag-row {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
  }

  .syn-chip, .ant-chip, .derived-chip {
    padding: 3px 9px;
    border-radius: 6px;
    font-size: 0.85rem;
    cursor: pointer;
    border: none;
  }

  .syn-chip {
    background: rgba(46, 204, 113, 0.12);
    color: #2ecc71;
  }

  .ant-chip {
    background: rgba(231, 76, 60, 0.12);
    color: #e74c3c;
  }

  .derived-chip {
    background: rgba(255, 255, 255, 0.06);
    color: var(--text-secondary);
  }

  .syn-chip:hover, .ant-chip:hover, .derived-chip:hover {
    filter: brightness(1.2);
  }

  .entry-footer {
    display: flex;
    justify-content: space-between;
    font-size: 0.78rem;
    color: var(--text-secondary);
    padding-top: 10px;
    border-top: 1px solid rgba(255, 255, 255, 0.05);
  }

  .empty-results {
    padding: 40px 20px;
    text-align: center;
    border-radius: 10px;
  }

  .empty-icon {
    font-size: 2.5rem;
    display: block;
    margin-bottom: 12px;
  }
</style>
