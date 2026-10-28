<script lang="ts">
  import { SUPPORTED_LANGUAGES, type LanguageCode } from '../lib/types';

  let searchTerm = '';
  let selectedLang: LanguageCode = 'kn-IN';

  const mockDictionaryFeatures = [
    { title: 'SQLite FTS5 Full-Text Search', desc: 'Sub-millisecond prefix, suffix, and fuzzy term search.' },
    { title: 'Grammatical Metadata', desc: 'Parts of speech, noun gender/case inflections, and verb conjugations.' },
    { title: 'Phonetic IPA & Transliteration', desc: 'Pronunciation guides and romanized search for Kannada and Tamil.' },
    { title: 'Curated Examples & Idioms', desc: 'Real sentence contexts with verifiable linguistic provenance.' },
  ];
</script>

<div class="dictionary-view">
  <!-- Search Card -->
  <div class="search-card glass-panel">
    <h2>Multilingual Dictionary & Glossary</h2>
    <p class="search-desc">
      Search definitions, grammatical forms, and idioms across Kannada, Tamil, and English.
    </p>

    <div class="search-bar-row">
      <select bind:value={selectedLang} class="lang-select">
        {#each SUPPORTED_LANGUAGES as lang}
          <option value={lang.code}>{lang.native} ({lang.name})</option>
        {/each}
      </select>

      <div class="search-input-wrap">
        <input
          type="text"
          bind:value={searchTerm}
          placeholder="Search words (e.g., ನಮಸ್ಕಾರ, வணக்கம், harmony)..."
          class="search-input indic-text"
        />
        {#if searchTerm}
          <button class="clear-btn" on:click={() => (searchTerm = '')}>×</button>
        {/if}
      </div>

      <button class="btn-primary" disabled={!searchTerm.trim()}>
        Search
      </button>
    </div>
  </div>

  <!-- Stage 0 Global Contract Transparency Card -->
  <div class="contract-card glass-panel">
    <div class="contract-header">
      <span class="badge badge-pending">Stage 0 Baseline</span>
      <span class="badge badge-active">SQLite FTS5 Architecture Ready</span>
    </div>
    <h3>Dictionary Engine Pipeline (Stage 1 Deliverable)</h3>
    <p>
      The SQLite database schema and FTS5 indexing engine are fully established in the Rust core. Real, licensed dictionary corpora (with validated provenance and copyright attribution) will be imported and indexed during <strong>Stage 1: Dictionary & Text Translation</strong>.
    </p>

    <div class="features-grid">
      {#each mockDictionaryFeatures as feat}
        <div class="feature-box">
          <h4>{feat.title}</h4>
          <p>{feat.desc}</p>
        </div>
      {/each}
    </div>
  </div>
</div>

<style>
  .dictionary-view {
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  .search-card {
    padding: 28px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .search-desc {
    color: var(--text-secondary);
    font-size: 0.95rem;
    margin-bottom: 8px;
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
    min-width: 260px;
    position: relative;
    display: flex;
    align-items: center;
  }

  .search-input {
    width: 100%;
    padding-right: 36px;
    font-size: 1.05rem;
  }

  .clear-btn {
    position: absolute;
    right: 12px;
    background: transparent;
    color: var(--text-muted);
    font-size: 1.25rem;
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .contract-card {
    padding: 28px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .contract-header {
    display: flex;
    gap: 10px;
  }

  .contract-card h3 {
    font-size: 1.2rem;
  }

  .contract-card p {
    color: var(--text-secondary);
    line-height: 1.6;
    max-width: 800px;
  }

  .features-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
    gap: 16px;
    margin-top: 8px;
  }

  .feature-box {
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: 16px;
  }

  .feature-box h4 {
    font-size: 0.95rem;
    margin-bottom: 6px;
    color: var(--text-accent);
  }

  .feature-box p {
    font-size: 0.85rem;
    color: var(--text-secondary);
    line-height: 1.5;
  }
</style>
