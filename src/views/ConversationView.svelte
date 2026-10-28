<script lang="ts">
  import { SUPPORTED_LANGUAGES, type LanguageCode } from '../lib/types';

  let speakerALang: LanguageCode = 'kn-IN';
  let speakerBLang: LanguageCode = 'en-US';
  let isListening = false;
  let activeSpeaker: 'A' | 'B' = 'A';

  function toggleListening() {
    isListening = !isListening;
  }
</script>

<div class="conversation-view">
  <!-- Conversation Header & Speakers -->
  <div class="speakers-card glass-panel">
    <div class="speaker-col">
      <div class="speaker-tag">
        <span class="avatar">👤</span>
        <div>
          <h3>Speaker 1</h3>
          <span class="sub-tag">Local Participant</span>
        </div>
      </div>
      <select bind:value={speakerALang} class="lang-select">
        {#each SUPPORTED_LANGUAGES as lang}
          <option value={lang.code}>{lang.native} ({lang.name})</option>
        {/each}
      </select>
    </div>

    <div class="center-divider">
      <span class="badge badge-info">Bidirectional</span>
    </div>

    <div class="speaker-col">
      <div class="speaker-tag">
        <span class="avatar">👥</span>
        <div>
          <h3>Speaker 2</h3>
          <span class="sub-tag">Remote / Interlocutor</span>
        </div>
      </div>
      <select bind:value={speakerBLang} class="lang-select">
        {#each SUPPORTED_LANGUAGES as lang}
          <option value={lang.code}>{lang.native} ({lang.name})</option>
        {/each}
      </select>
    </div>
  </div>

  <!-- Dialogue Transcript Area -->
  <div class="dialogue-card glass-panel">
    <div class="dialogue-header">
      <span class="section-tag">Live Session Captions</span>
      <span class="badge badge-pending">Audio Pipeline Scheduled (Stage 3 & 4)</span>
    </div>

    <div class="dialogue-body">
      <div class="empty-state">
        <div class="empty-icon">🎙️</div>
        <h4>Real-Time Voice Conversation Baseline</h4>
        <p>
          Push-to-talk state machine, 16kHz audio ring buffers, and acoustic echo cancellation (AEC) architecture are defined in <code>docs/architecture/audio-inference-pipeline.md</code>.
        </p>
        <p class="empty-sub">
          Streaming ASR integration with sherpa-onnx and neural TTS voice synthesis will be activated in <strong>Stage 3</strong> and <strong>Stage 4</strong>.
        </p>
      </div>
    </div>

    <!-- Push-To-Talk Control Bar -->
    <div class="ptt-bar">
      <div class="ptt-controls">
        <button
          class="ptt-btn {isListening ? 'listening pulsing' : ''}"
          on:click={toggleListening}
        >
          <span class="mic-icon">🎤</span>
          <span>{isListening ? 'Listening (Stage 0 Mock-Free Standby)...' : 'Hold / Tap to Speak'}</span>
        </button>
      </div>
      <span class="ptt-hint">Push-To-Talk baseline ready for native audio callback binding</span>
    </div>
  </div>
</div>

<style>
  .conversation-view {
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  .speakers-card {
    padding: 24px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 20px;
    flex-wrap: wrap;
  }

  .speaker-col {
    display: flex;
    flex-direction: column;
    gap: 12px;
    flex: 1;
    min-width: 220px;
  }

  .speaker-tag {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .avatar {
    font-size: 1.75rem;
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-full);
    width: 48px;
    height: 48px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .sub-tag {
    font-size: 0.8rem;
    color: var(--text-muted);
  }

  .lang-select {
    width: 100%;
    font-weight: 600;
  }

  .center-divider {
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .dialogue-card {
    padding: 24px;
    min-height: 400px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .dialogue-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .section-tag {
    font-weight: 600;
    color: var(--text-secondary);
    font-size: 0.95rem;
  }

  .dialogue-body {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    border: 1px dashed var(--border-subtle);
    border-radius: var(--radius-md);
    padding: 32px;
  }

  .empty-state {
    text-align: center;
    max-width: 540px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
  }

  .empty-icon {
    font-size: 2.5rem;
    margin-bottom: 4px;
  }

  .empty-state h4 {
    font-size: 1.15rem;
  }

  .empty-state p {
    font-size: 0.9rem;
    color: var(--text-secondary);
    line-height: 1.5;
  }

  .empty-sub {
    font-size: 0.85rem !important;
    color: var(--text-accent) !important;
  }

  .empty-state code {
    background: var(--bg-surface);
    padding: 2px 6px;
    border-radius: 4px;
    font-family: var(--font-mono);
  }

  .ptt-bar {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
    padding-top: 12px;
    border-top: 1px solid var(--border-subtle);
  }

  .ptt-btn {
    background: var(--primary-gradient);
    color: #ffffff;
    padding: 14px 28px;
    border-radius: var(--radius-full);
    font-weight: 700;
    font-size: 1rem;
    display: inline-flex;
    align-items: center;
    gap: 10px;
    box-shadow: var(--shadow-md);
  }

  .ptt-btn.listening {
    background: var(--accent-gradient);
  }

  .mic-icon {
    font-size: 1.25rem;
  }

  .ptt-hint {
    font-size: 0.8rem;
    color: var(--text-muted);
  }
</style>
