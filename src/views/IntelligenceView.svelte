<script lang="ts">
  import { onMount } from 'svelte';
  import {
    getCircuitBreakers,
    resetCircuitBreaker,
    listGlossaryTerms,
    createGlossaryTerm,
    deleteGlossaryTerm,
    runEvaluation,
    getLatestEvaluationReport,
    setResourceProfile,
    transliterateText,
    translateWithIntelligence,
  } from '../lib/ipc';
  import type {
    CircuitBreakerStatus,
    DomainGlossaryEntry,
    EvaluationReport,
    FormalityMode,
    IntelligenceTranslationResponse,
    LanguageCode,
    ResourceProfile,
  } from '../lib/types';
  import { SUPPORTED_LANGUAGES } from '../lib/types';

  // Sub-tabs
  type TabType = 'overview' | 'glossary' | 'breakers' | 'evaluation';
  let activeTab: TabType = 'overview';

  // Live Circuit Breakers
  let circuitBreakers: CircuitBreakerStatus[] = [];
  let isBreakersLoading = false;

  // Domain Glossaries
  let glossaryTerms: DomainGlossaryEntry[] = [];
  let selectedDomain = 'all';
  let newDomain = 'tech';
  let newSourceLang: LanguageCode = 'en-US';
  let newTargetLang: LanguageCode = 'kn-IN';
  let newSourceTerm = '';
  let newTargetTerm = '';
  let isGlossaryLoading = false;
  let showAddGlossaryModal = false;

  // Evaluation
  let evaluationReport: EvaluationReport | null = null;
  let isEvaluating = false;

  // Resource Profile
  let activeProfile: ResourceProfile = 'balanced';
  let profileMessage = '';

  // Interactive Intelligence Playground
  let playText = 'namaskara';
  let playSourceLang: LanguageCode = 'kn-IN';
  let playTargetLang: LanguageCode = 'en-US';
  let playFormality: FormalityMode = 'formal';
  let playTransliterate = true;
  let playGlossaryDomain = 'tech';
  let playResult: IntelligenceTranslationResponse | null = null;
  let isPlayTranslating = false;
  let playError: string | null = null;

  onMount(async () => {
    await refreshAll();
  });

  async function refreshAll() {
    await Promise.all([
      loadCircuitBreakers(),
      loadGlossary(),
      loadEvaluationReport(),
    ]);
  }

  async function loadCircuitBreakers() {
    isBreakersLoading = true;
    try {
      circuitBreakers = await getCircuitBreakers();
    } catch (e) {
      console.error('Failed to load circuit breakers:', e);
    } finally {
      isBreakersLoading = false;
    }
  }

  async function handleResetBreaker(providerId: string) {
    try {
      await resetCircuitBreaker(providerId);
      await loadCircuitBreakers();
    } catch (e) {
      console.error('Failed to reset circuit breaker:', e);
    }
  }

  async function loadGlossary() {
    isGlossaryLoading = true;
    try {
      const dom = selectedDomain === 'all' ? undefined : selectedDomain;
      glossaryTerms = await listGlossaryTerms(dom);
    } catch (e) {
      console.error('Failed to load glossaries:', e);
    } finally {
      isGlossaryLoading = false;
    }
  }

  async function handleAddGlossary() {
    if (!newSourceTerm.trim() || !newTargetTerm.trim()) return;
    try {
      await createGlossaryTerm({
        domain: newDomain,
        source_term: newSourceTerm.trim(),
        target_term: newTargetTerm.trim(),
        source_lang: newSourceLang,
        target_lang: newTargetLang,
        is_exact: true,
      });
      newSourceTerm = '';
      newTargetTerm = '';
      showAddGlossaryModal = false;
      await loadGlossary();
    } catch (e) {
      console.error('Failed to add glossary term:', e);
    }
  }

  async function handleDeleteGlossary(id: string) {
    try {
      await deleteGlossaryTerm(id);
      await loadGlossary();
    } catch (e) {
      console.error('Failed to delete glossary term:', e);
    }
  }

  async function loadEvaluationReport() {
    try {
      evaluationReport = await getLatestEvaluationReport();
    } catch (e) {
      console.error('Failed to load evaluation report:', e);
    }
  }

  async function handleRunEvaluation() {
    isEvaluating = true;
    try {
      evaluationReport = await runEvaluation();
    } catch (e) {
      console.error('Failed to run evaluation:', e);
    } finally {
      isEvaluating = false;
    }
  }

  async function handleProfileChange(profile: ResourceProfile) {
    activeProfile = profile;
    try {
      await setResourceProfile(profile);
      profileMessage = `Profile updated to ${profile.replace('_', ' ').toUpperCase()} (LRU cache resized)`;
      setTimeout(() => (profileMessage = ''), 3000);
    } catch (e) {
      console.error('Failed to set resource profile:', e);
    }
  }

  async function executePlaygroundTranslation() {
    if (!playText.trim()) return;
    isPlayTranslating = true;
    playError = null;
    try {
      playResult = await translateWithIntelligence({
        text: playText.trim(),
        source_lang: playSourceLang,
        target_lang: playTargetLang,
        formality: playFormality,
        enable_transliteration: playTransliterate,
        domain_glossary: playGlossaryDomain === 'none' ? null : playGlossaryDomain,
      });
    } catch (e: any) {
      playError = e?.message || 'Translation failed';
      playResult = null;
    } finally {
      isPlayTranslating = false;
    }
  }
</script>

<div class="intelligence-container">
  <!-- Header Bar -->
  <header class="intelligence-header glass-panel">
    <div class="header-titles">
      <div class="stage-tag">🧠 Translation Intelligence & NLP</div>
      <h2>Translation Intelligence & Optimization</h2>
      <p>Multi-Engine Policies, Circuit Breakers, Formality Modes, Glossaries & chrF/WER Benchmarks</p>
    </div>

    <!-- Resource Profile Selector -->
    <div class="profile-selector">
      <label for="resource-profile">Resource Profile:</label>
      <select
        id="resource-profile"
        value={activeProfile}
        on:change={(e) => handleProfileChange(e.currentTarget.value as ResourceProfile)}
      >
        <option value="power_saver">🌱 Power Saver (500 entries)</option>
        <option value="balanced">⚖️ Balanced (2,000 entries)</option>
        <option value="high_performance">🚀 High Performance (8,000 entries)</option>
      </select>
      {#if profileMessage}
        <span class="profile-msg">{profileMessage}</span>
      {/if}
    </div>
  </header>

  <!-- Navigation Tabs -->
  <nav class="subnav-tabs glass-panel">
    <button
      class="subnav-btn {activeTab === 'overview' ? 'active' : ''}"
      on:click={() => (activeTab = 'overview')}
    >
      ⚡ Intelligence Studio
    </button>
    <button
      class="subnav-btn {activeTab === 'breakers' ? 'active' : ''}"
      on:click={() => (activeTab = 'breakers')}
    >
      🛡️ Circuit Breakers ({circuitBreakers.length})
    </button>
    <button
      class="subnav-btn {activeTab === 'glossary' ? 'active' : ''}"
      on:click={() => (activeTab = 'glossary')}
    >
      📖 Domain Glossaries ({glossaryTerms.length})
    </button>
    <button
      class="subnav-btn {activeTab === 'evaluation' ? 'active' : ''}"
      on:click={() => (activeTab = 'evaluation')}
    >
      📊 Quality Benchmarks (chrF / WER)
    </button>
  </nav>

  <!-- TAB 1: INTELLIGENCE STUDIO -->
  {#if activeTab === 'overview'}
    <div class="studio-grid">
      <!-- Input Panel -->
      <div class="studio-card glass-panel">
        <div class="card-header">
          <h3>Input Text & Intelligence Controls</h3>
          <span class="badge badge-accent">Interactive</span>
        </div>

        <div class="control-row">
          <div class="control-group">
            <label for="source-lang">Source:</label>
            <select id="source-lang" bind:value={playSourceLang}>
              {#each SUPPORTED_LANGUAGES as lang}
                <option value={lang.code}>{lang.native} ({lang.short})</option>
              {/each}
            </select>
          </div>

          <div class="control-group">
            <label for="target-lang">Target:</label>
            <select id="target-lang" bind:value={playTargetLang}>
              {#each SUPPORTED_LANGUAGES as lang}
                <option value={lang.code}>{lang.native} ({lang.short})</option>
              {/each}
            </select>
          </div>

          <div class="control-group">
            <label for="formality-mode">Formality:</label>
            <select id="formality-mode" bind:value={playFormality}>
              <option value="neutral">Neutral</option>
              <option value="formal">Formal / Respectful</option>
              <option value="colloquial">Colloquial / Casual</option>
            </select>
          </div>

          <div class="control-group">
            <label for="glossary-domain">Glossary:</label>
            <select id="glossary-domain" bind:value={playGlossaryDomain}>
              <option value="none">None</option>
              <option value="tech">Technology</option>
              <option value="medical">Medical</option>
              <option value="general">General</option>
            </select>
          </div>
        </div>

        <div class="toggle-row">
          <label class="toggle-label">
            <input type="checkbox" bind:checked={playTransliterate} />
            <span>Phonetic Transliteration (Kanglish / Tanglish Romanized Script)</span>
          </label>
        </div>

        <div class="textarea-wrapper">
          <textarea
            bind:value={playText}
            placeholder="Type text in Kannada, Tamil, English, or Romanized (e.g. 'namaskara', 'vanakkam', 'Antigravity')..."
            rows="4"
          ></textarea>
        </div>

        <button
          class="btn-primary"
          on:click={executePlaygroundTranslation}
          disabled={isPlayTranslating}
        >
          {#if isPlayTranslating}
            <span class="spinner"></span> Processing Intelligence Pipeline...
          {:else}
            ✨ Translate with Intelligence
          {/if}
        </button>
      </div>

      <!-- Result Panel -->
      <div class="studio-card glass-panel">
        <div class="card-header">
          <h3>Intelligent Translation & Observable Fallback Trail</h3>
          {#if playResult}
            <span class="badge badge-success">Engine: {playResult.metadata.engine_selected}</span>
          {/if}
        </div>

        {#if playError}
          <div class="alert alert-error">{playError}</div>
        {/if}

        {#if playResult}
          <div class="result-box">
            <div class="result-label">Translated Text:</div>
            <div class="result-content">{playResult.translated_text}</div>

            {#if playResult.metadata.transliterated_input}
              <div class="meta-pill">
                🔤 Transliterated Script: <strong>{playResult.metadata.transliterated_input}</strong>
              </div>
            {/if}

            {#if playResult.metadata.chrf_estimated !== undefined && playResult.metadata.chrf_estimated !== null}
              <div class="meta-pill">
                🎯 Estimated Quality (chrF): <strong>{playResult.metadata.chrf_estimated.toFixed(1)}%</strong>
              </div>
            {/if}

            <div class="meta-pill">
              🎭 Formality Mode: <strong>{playResult.metadata.formality.toUpperCase()}</strong>
            </div>

            <!-- Fallback Trail -->
            <div class="trail-section">
              <h4>Observable Fallback Trail ({playResult.metadata.fallback_trail.length} step(s)):</h4>
              <div class="trail-list">
                {#each playResult.metadata.fallback_trail as step, idx}
                  <div class="trail-step {step.success ? 'step-success' : 'step-fail'}">
                    <span class="step-num">#{idx + 1}</span>
                    <span class="step-engine">{step.engine_id}</span>
                    <span class="step-latency">{step.latency_ms.toFixed(2)} ms</span>
                    <span class="step-status">{step.success ? '✅ Success' : '⚠️ Diverted / Bypassed'}</span>
                    {#if step.reason}
                      <span class="step-reason">({step.reason})</span>
                    {/if}
                  </div>
                {/each}
              </div>
            </div>
          </div>
        {:else}
          <div class="empty-placeholder">
            <p>Enter text on the left and click "Translate with Intelligence" to view observable fallback step execution, transliteration, and formality adaptation.</p>
          </div>
        {/if}
      </div>
    </div>

  <!-- TAB 2: CIRCUIT BREAKERS -->
  {:else if activeTab === 'breakers'}
    <div class="breakers-grid">
      <div class="section-intro glass-panel">
        <h3>Provider Circuit Breakers</h3>
        <p>
          Circuit breakers prevent cascade failures when upstream translation providers (e.g., cloud endpoints) fail.
          When threshold consecutive errors occur, the circuit transitions from <strong>Closed</strong> to <strong>Open</strong>,
          diverting subsequent queries immediately to verified offline models in &lt; 0.1ms without network timeout delay.
        </p>
      </div>

      <div class="breakers-cards">
        {#each circuitBreakers as cb}
          <div class="breaker-card glass-panel state-{cb.state}">
            <div class="breaker-top">
              <span class="breaker-title">{cb.provider_id}</span>
              <span class="breaker-state-badge state-{cb.state}">{cb.state.toUpperCase()}</span>
            </div>

            <div class="breaker-metrics">
              <div class="metric-item">
                <span class="metric-val">{cb.failure_count}</span>
                <span class="metric-lbl">Failures</span>
              </div>
              <div class="metric-item">
                <span class="metric-val">{cb.success_count}</span>
                <span class="metric-lbl">Successes</span>
              </div>
              <div class="metric-item">
                <span class="metric-val">{cb.total_requests}</span>
                <span class="metric-lbl">Total Requests</span>
              </div>
            </div>

            <div class="breaker-actions">
              <button
                class="btn-secondary btn-sm"
                on:click={() => handleResetBreaker(cb.provider_id)}
              >
                🔄 Reset Breaker to Closed
              </button>
            </div>
          </div>
        {:else}
          <div class="glass-panel empty-placeholder">
            <p>No circuit breakers registered.</p>
          </div>
        {/each}
      </div>
    </div>

  <!-- TAB 3: DOMAIN GLOSSARIES -->
  {:else if activeTab === 'glossary'}
    <div class="glossary-section glass-panel">
      <div class="glossary-toolbar">
        <div class="filter-group">
          <label for="filter-domain">Domain:</label>
          <select id="filter-domain" bind:value={selectedDomain} on:change={loadGlossary}>
            <option value="all">All Domains</option>
            <option value="tech">Technology</option>
            <option value="medical">Medical</option>
            <option value="general">General</option>
          </select>
        </div>

        <button class="btn-primary" on:click={() => (showAddGlossaryModal = true)}>
          ➕ Add Glossary Term
        </button>
      </div>

      {#if showAddGlossaryModal}
        <div class="modal-backdrop">
          <div class="modal-card glass-panel">
            <h3>Add Domain Glossary Term</h3>
            <div class="form-grid">
              <div class="form-group">
                <label for="modal-domain">Domain:</label>
                <input id="modal-domain" bind:value={newDomain} placeholder="e.g. tech, medical" />
              </div>
              <div class="form-group">
                <label for="modal-src-lang">Source Lang:</label>
                <select id="modal-src-lang" bind:value={newSourceLang}>
                  {#each SUPPORTED_LANGUAGES as l}
                    <option value={l.code}>{l.name}</option>
                  {/each}
                </select>
              </div>
              <div class="form-group">
                <label for="modal-tgt-lang">Target Lang:</label>
                <select id="modal-tgt-lang" bind:value={newTargetLang}>
                  {#each SUPPORTED_LANGUAGES as l}
                    <option value={l.code}>{l.name}</option>
                  {/each}
                </select>
              </div>
              <div class="form-group">
                <label for="modal-src-term">Source Term:</label>
                <input id="modal-src-term" bind:value={newSourceTerm} placeholder="e.g. Artificial Intelligence" />
              </div>
              <div class="form-group">
                <label for="modal-tgt-term">Target Term:</label>
                <input id="modal-tgt-term" bind:value={newTargetTerm} placeholder="e.g. ಕೃತಕ ಬುದ್ಧಿಮತ್ತೆ" />
              </div>
            </div>

            <div class="modal-actions">
              <button class="btn-secondary" on:click={() => (showAddGlossaryModal = false)}>Cancel</button>
              <button class="btn-primary" on:click={handleAddGlossary}>Save Term</button>
            </div>
          </div>
        </div>
      {/if}

      <!-- Glossary Table -->
      <div class="table-wrapper">
        <table class="glossary-table">
          <thead>
            <tr>
              <th>Domain</th>
              <th>Source Term</th>
              <th>Language</th>
              <th>Target Term</th>
              <th>Action</th>
            </tr>
          </thead>
          <tbody>
            {#each glossaryTerms as term}
              <tr>
                <td><span class="badge">{term.domain}</span></td>
                <td><strong>{term.source_term}</strong></td>
                <td>{term.source_lang} ➔ {term.target_lang}</td>
                <td class="indic-text">{term.target_term}</td>
                <td>
                  <button
                    class="btn-danger btn-sm"
                    on:click={() => handleDeleteGlossary(term.id)}
                  >
                    Delete
                  </button>
                </td>
              </tr>
            {:else}
              <tr>
                <td colspan="5" class="empty-table">No glossary terms found for domain '{selectedDomain}'.</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    </div>

  <!-- TAB 4: QUALITY EVALUATION BENCHMARKS -->
  {:else if activeTab === 'evaluation'}
    <div class="evaluation-section">
      <div class="eval-header glass-panel">
        <div>
          <h3>Linguistic Quality Evaluation Suite</h3>
          <p>
            Standardized Popović 2015 chrF character n-gram scoring (N=6, &beta;=2), Word Error Rate (WER),
            and Character Error Rate (CER) computed with authentic Levenshtein edit distance alignments.
          </p>
        </div>
        <button
          class="btn-primary"
          on:click={handleRunEvaluation}
          disabled={isEvaluating}
        >
          {#if isEvaluating}
            <span class="spinner"></span> Running Batch Benchmark...
          {:else}
            🚀 Run Evaluation Suite
          {/if}
        </button>
      </div>

      {#if evaluationReport}
        <!-- Score Summary Cards -->
        <div class="metrics-cards">
          <div class="metric-card glass-panel">
            <span class="metric-title">Mean chrF Score</span>
            <span class="metric-number metric-chrf">{evaluationReport.mean_chrf.toFixed(2)}%</span>
            <span class="metric-sub">Character 6-gram F-score (&beta;=2)</span>
          </div>
          <div class="metric-card glass-panel">
            <span class="metric-title">Mean Word Error Rate (WER)</span>
            <span class="metric-number metric-wer">{evaluationReport.mean_wer.toFixed(2)}%</span>
            <span class="metric-sub">Word substitution/deletion/insertion</span>
          </div>
          <div class="metric-card glass-panel">
            <span class="metric-title">Mean Character Error Rate (CER)</span>
            <span class="metric-number metric-cer">{evaluationReport.mean_cer.toFixed(2)}%</span>
            <span class="metric-sub">Character Levenshtein error rate</span>
          </div>
          <div class="metric-card glass-panel">
            <span class="metric-title">Total Benchmark Samples</span>
            <span class="metric-number">{evaluationReport.total_samples}</span>
            <span class="metric-sub">Ordinary speech, idioms, & specialist</span>
          </div>
        </div>

        <!-- Sample Breakdown Table -->
        <div class="table-wrapper glass-panel">
          <table class="eval-table">
            <thead>
              <tr>
                <th>Category</th>
                <th>Hypothesis Output</th>
                <th>Reference Translation</th>
                <th>chrF</th>
                <th>WER</th>
                <th>CER</th>
              </tr>
            </thead>
            <tbody>
              {#each evaluationReport.samples as sample}
                <tr>
                  <td><span class="badge badge-accent">{sample.category}</span></td>
                  <td class="text-mono indic-text">{sample.hypothesis}</td>
                  <td class="text-mono indic-text">{sample.reference}</td>
                  <td class="score-cell {sample.chrf_score >= 80 ? 'good' : 'warn'}">{sample.chrf_score.toFixed(1)}%</td>
                  <td class="score-cell {sample.wer_score <= 20 ? 'good' : 'warn'}">{sample.wer_score.toFixed(1)}%</td>
                  <td class="score-cell {sample.cer_score <= 15 ? 'good' : 'warn'}">{sample.cer_score.toFixed(1)}%</td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {:else}
        <div class="glass-panel empty-placeholder">
          <p>No evaluation report generated yet. Click "Run Evaluation Suite" above to benchmark the active models.</p>
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .intelligence-container {
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
    padding: 1.5rem;
    max-width: 1400px;
    margin: 0 auto;
  }

  .intelligence-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1.5rem;
    border-radius: 1rem;
    background: var(--bg-card, rgba(255, 255, 255, 0.05));
    border: 1px solid var(--border-color, rgba(255, 255, 255, 0.1));
  }

  .stage-tag {
    font-size: 0.75rem;
    font-weight: 700;
    color: var(--primary-accent, #6366f1);
    letter-spacing: 0.1em;
    margin-bottom: 0.25rem;
  }

  .header-titles h2 {
    margin: 0;
    font-size: 1.5rem;
    font-weight: 700;
  }

  .header-titles p {
    margin: 0.25rem 0 0;
    color: var(--text-muted, #94a3b8);
    font-size: 0.9rem;
  }

  .profile-selector {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 0.25rem;
  }

  .profile-selector label {
    font-size: 0.8rem;
    color: var(--text-muted, #94a3b8);
  }

  .profile-selector select {
    padding: 0.5rem 0.75rem;
    border-radius: 0.5rem;
    border: 1px solid var(--border-color, rgba(255, 255, 255, 0.2));
    background: var(--bg-surface, #1e293b);
    color: inherit;
    font-size: 0.875rem;
  }

  .profile-msg {
    font-size: 0.75rem;
    color: #10b981;
  }

  .subnav-tabs {
    display: flex;
    gap: 0.5rem;
    padding: 0.5rem;
    border-radius: 0.75rem;
    background: var(--bg-card, rgba(255, 255, 255, 0.03));
    border: 1px solid var(--border-color, rgba(255, 255, 255, 0.1));
    overflow-x: auto;
  }

  .subnav-btn {
    padding: 0.5rem 1rem;
    border: none;
    background: transparent;
    color: var(--text-muted, #94a3b8);
    font-size: 0.9rem;
    font-weight: 600;
    border-radius: 0.5rem;
    cursor: pointer;
    transition: all 0.2s;
    white-space: nowrap;
  }

  .subnav-btn:hover {
    color: var(--text-color, #fff);
    background: rgba(255, 255, 255, 0.05);
  }

  .subnav-btn.active {
    color: #fff;
    background: var(--primary-accent, #6366f1);
  }

  /* Studio Tab */
  .studio-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1.25rem;
  }

  @media (max-width: 900px) {
    .studio-grid {
      grid-template-columns: 1fr;
    }
  }

  .studio-card {
    padding: 1.5rem;
    border-radius: 1rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
    background: var(--bg-card, rgba(255, 255, 255, 0.05));
    border: 1px solid var(--border-color, rgba(255, 255, 255, 0.1));
  }

  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .card-header h3 {
    margin: 0;
    font-size: 1.15rem;
  }

  .control-row {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 0.75rem;
  }

  .control-group {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .control-group label {
    font-size: 0.75rem;
    color: var(--text-muted, #94a3b8);
  }

  .control-group select {
    padding: 0.4rem 0.6rem;
    border-radius: 0.4rem;
    border: 1px solid var(--border-color, rgba(255, 255, 255, 0.2));
    background: var(--bg-surface, #1e293b);
    color: inherit;
    font-size: 0.85rem;
  }

  .toggle-row {
    font-size: 0.85rem;
  }

  .toggle-label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    cursor: pointer;
  }

  .textarea-wrapper textarea {
    width: 100%;
    padding: 0.75rem;
    border-radius: 0.5rem;
    border: 1px solid var(--border-color, rgba(255, 255, 255, 0.2));
    background: var(--bg-surface, #1e293b);
    color: inherit;
    font-size: 0.95rem;
    resize: vertical;
    box-sizing: border-box;
  }

  .result-box {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .result-label {
    font-size: 0.8rem;
    color: var(--text-muted, #94a3b8);
  }

  .result-content {
    font-size: 1.25rem;
    font-weight: 600;
    line-height: 1.5;
    padding: 0.75rem;
    background: rgba(255, 255, 255, 0.03);
    border-radius: 0.5rem;
  }

  .meta-pill {
    font-size: 0.85rem;
    padding: 0.35rem 0.65rem;
    background: rgba(99, 102, 241, 0.1);
    border: 1px solid rgba(99, 102, 241, 0.3);
    border-radius: 0.4rem;
    color: #cbd5e1;
  }

  .trail-section h4 {
    margin: 0.5rem 0 0.25rem;
    font-size: 0.9rem;
    color: var(--text-muted, #94a3b8);
  }

  .trail-list {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  .trail-step {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.4rem 0.6rem;
    border-radius: 0.4rem;
    font-size: 0.8rem;
  }

  .step-success {
    background: rgba(16, 185, 129, 0.1);
    border: 1px solid rgba(16, 185, 129, 0.3);
  }

  .step-fail {
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid rgba(239, 68, 68, 0.3);
  }

  .step-num {
    font-weight: 700;
  }

  .step-engine {
    font-weight: 600;
  }

  .step-latency {
    color: var(--text-muted, #94a3b8);
    margin-left: auto;
  }

  /* Circuit Breakers */
  .breakers-grid {
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
  }

  .breakers-cards {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
    gap: 1rem;
  }

  .breaker-card {
    padding: 1.25rem;
    border-radius: 0.75rem;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    background: var(--bg-card, rgba(255, 255, 255, 0.05));
    border: 1px solid var(--border-color, rgba(255, 255, 255, 0.1));
  }

  .breaker-top {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .breaker-title {
    font-weight: 700;
    font-size: 1rem;
  }

  .breaker-state-badge {
    padding: 0.2rem 0.5rem;
    border-radius: 0.25rem;
    font-size: 0.75rem;
    font-weight: 700;
  }

  .breaker-state-badge.state-closed {
    background: rgba(16, 185, 129, 0.2);
    color: #10b981;
    border: 1px solid #10b981;
  }

  .breaker-state-badge.state-open {
    background: rgba(239, 68, 68, 0.2);
    color: #ef4444;
    border: 1px solid #ef4444;
  }

  .breaker-state-badge.state-half_open {
    background: rgba(245, 158, 11, 0.2);
    color: #f59e0b;
    border: 1px solid #f59e0b;
  }

  .breaker-metrics {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 0.5rem;
    text-align: center;
  }

  .metric-item {
    background: rgba(255, 255, 255, 0.03);
    padding: 0.5rem;
    border-radius: 0.5rem;
  }

  .metric-val {
    display: block;
    font-size: 1.1rem;
    font-weight: 700;
  }

  .metric-lbl {
    font-size: 0.7rem;
    color: var(--text-muted, #94a3b8);
  }

  /* Glossaries */
  .glossary-toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
  }

  .modal-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }

  .modal-card {
    background: var(--bg-surface, #1e293b);
    padding: 1.5rem;
    border-radius: 1rem;
    width: 450px;
    max-width: 90vw;
  }

  .form-grid {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    margin: 1rem 0;
  }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .form-group input,
  .form-group select {
    padding: 0.5rem;
    border-radius: 0.4rem;
    border: 1px solid var(--border-color, rgba(255, 255, 255, 0.2));
    background: var(--bg-card, #0f172a);
    color: inherit;
  }

  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
  }

  .table-wrapper {
    overflow-x: auto;
  }

  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.9rem;
  }

  th, td {
    padding: 0.75rem;
    text-align: left;
    border-bottom: 1px solid var(--border-color, rgba(255, 255, 255, 0.08));
  }

  th {
    color: var(--text-muted, #94a3b8);
    font-weight: 600;
  }

  /* Evaluation */
  .evaluation-section {
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
  }

  .eval-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1.5rem;
    border-radius: 1rem;
  }

  .metrics-cards {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
    gap: 1rem;
  }

  .metric-card {
    padding: 1.25rem;
    border-radius: 0.75rem;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .metric-title {
    font-size: 0.8rem;
    color: var(--text-muted, #94a3b8);
  }

  .metric-number {
    font-size: 1.75rem;
    font-weight: 800;
  }

  .metric-chrf {
    color: #10b981;
  }

  .metric-wer {
    color: #6366f1;
  }

  .metric-cer {
    color: #38bdf8;
  }

  .metric-sub {
    font-size: 0.75rem;
    color: var(--text-muted, #94a3b8);
  }

  .score-cell.good {
    color: #10b981;
    font-weight: 700;
  }

  .score-cell.warn {
    color: #f59e0b;
    font-weight: 700;
  }

  /* Common utilities */
  .btn-primary {
    padding: 0.6rem 1.25rem;
    background: var(--primary-accent, #6366f1);
    color: #fff;
    border: none;
    border-radius: 0.5rem;
    font-weight: 600;
    cursor: pointer;
    transition: opacity 0.2s;
  }

  .btn-primary:hover:not(:disabled) {
    opacity: 0.9;
  }

  .btn-secondary {
    padding: 0.5rem 1rem;
    background: rgba(255, 255, 255, 0.1);
    color: inherit;
    border: 1px solid var(--border-color, rgba(255, 255, 255, 0.2));
    border-radius: 0.4rem;
    cursor: pointer;
  }

  .btn-danger {
    padding: 0.35rem 0.6rem;
    background: rgba(239, 68, 68, 0.2);
    color: #ef4444;
    border: 1px solid #ef4444;
    border-radius: 0.35rem;
    cursor: pointer;
  }

  .btn-sm {
    font-size: 0.8rem;
  }

  .badge {
    padding: 0.2rem 0.5rem;
    border-radius: 0.25rem;
    font-size: 0.75rem;
    background: rgba(255, 255, 255, 0.1);
  }

  .badge-accent {
    background: rgba(99, 102, 241, 0.2);
    color: #818cf8;
  }

  .badge-success {
    background: rgba(16, 185, 129, 0.2);
    color: #10b981;
  }

  .indic-text {
    font-family: inherit;
  }

  .empty-placeholder {
    padding: 2.5rem;
    text-align: center;
    color: var(--text-muted, #94a3b8);
  }
</style>
