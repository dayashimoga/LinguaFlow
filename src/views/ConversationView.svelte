<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import {
    SUPPORTED_LANGUAGES,
    type LanguageCode,
    type ConversationSession,
    type ConversationTurnItem,
    type SpeakerId,
    type TtsVoiceInfo,
  } from '../lib/types';
  import {
    startConversation,
    submitConversationTurn,
    switchConversationSpeaker,
    interruptConversation,
    stopConversation,
    getConversationTurns,
    listTtsVoices,
    synthesizeSpeech,
  } from '../lib/ipc';

  export let onNavigate: ((view: string) => void) | undefined = undefined;

  let speakerALang: LanguageCode = 'kn-IN';
  let speakerBLang: LanguageCode = 'en-US';
  let isHandsFree = false;
  let isFaceToFaceFlipped = false;
  let currentSession: ConversationSession | null = null;
  let turns: ConversationTurnItem[] = [];
  let ttsVoices: TtsVoiceInfo[] = [];

  let activeSpeaker: SpeakerId = 'SpeakerA';
  let turnState: 'idle' | 'listening' | 'processing' | 'speaking' = 'idle';
  let textInput = '';
  let isAudioPlaying = false;
  let errorMessage: string | null = null;
  let successNotice: string | null = null;

  // Audio Context for direct playback of PCM samples
  let audioContext: AudioContext | null = null;
  let currentSourceNode: AudioBufferSourceNode | null = null;

  onMount(async () => {
    try {
      ttsVoices = await listTtsVoices();
    } catch (e) {
      console.warn('Could not list TTS voices on mount:', e);
    }
  });

  onDestroy(() => {
    stopCurrentAudio();
    if (audioContext && audioContext.state !== 'closed') {
      audioContext.close().catch(() => {});
    }
  });

  function stopCurrentAudio() {
    if (currentSourceNode) {
      try {
        currentSourceNode.stop();
      } catch (_) {}
      currentSourceNode = null;
    }
    isAudioPlaying = false;
  }

  function playPcmSamples(samples: number[], sampleRate = 16000) {
    try {
      stopCurrentAudio();
      if (!audioContext || audioContext.state === 'closed') {
        const AudioCtx = window.AudioContext || (window as unknown as { webkitAudioContext: typeof AudioContext }).webkitAudioContext;
        audioContext = new AudioCtx({ sampleRate });
      }

      if (audioContext.state === 'suspended') {
        audioContext.resume();
      }

      const buffer = audioContext.createBuffer(1, samples.length, sampleRate);
      const channelData = buffer.getChannelData(0);
      for (let i = 0; i < samples.length; i++) {
        channelData[i] = samples[i];
      }

      const source = audioContext.createBufferSource();
      source.buffer = buffer;
      source.connect(audioContext.destination);

      isAudioPlaying = true;
      source.onended = () => {
        isAudioPlaying = false;
        currentSourceNode = null;
      };

      currentSourceNode = source;
      source.start();
    } catch (err) {
      console.error('Audio playback error:', err);
      isAudioPlaying = false;
    }
  }

  async function handleStartSession() {
    errorMessage = null;
    try {
      currentSession = await startConversation(
        speakerALang,
        speakerBLang,
        `Live ${getLangMeta(speakerALang).name} - ${getLangMeta(speakerBLang).name} Session`,
        isHandsFree
      );
      activeSpeaker = currentSession.active_speaker;
      turns = [];
      turnState = 'idle';
      showNotice('Conversation session started. Hold or tap to speak!');
    } catch (err: unknown) {
      errorMessage = (err as Error).message || 'Failed to start session';
    }
  }

  async function handleStopSession() {
    if (!currentSession) return;
    try {
      stopCurrentAudio();
      await stopConversation(currentSession.id);
      currentSession = null;
      turnState = 'idle';
      showNotice('Conversation session ended.');
    } catch (err: unknown) {
      errorMessage = (err as Error).message || 'Failed to stop session';
    }
  }

  async function handleSwapLanguages() {
    const tmp = speakerALang;
    speakerALang = speakerBLang;
    speakerBLang = tmp;

    if (currentSession) {
      await handleStopSession();
      await handleStartSession();
    }
  }

  async function handleSwitchActiveSpeaker(next?: SpeakerId) {
    if (!currentSession) return;
    try {
      currentSession = await switchConversationSpeaker(currentSession.id, next);
      activeSpeaker = currentSession.active_speaker;
    } catch (err: unknown) {
      errorMessage = (err as Error).message || 'Failed to switch speaker';
    }
  }

  async function handleInterrupt() {
    if (!currentSession) return;
    try {
      stopCurrentAudio();
      await interruptConversation(currentSession.id);
      turnState = 'idle';
      showNotice('Barge-in: active speech interrupted.');
      // Refresh turns
      turns = await getConversationTurns(currentSession.id);
    } catch (err: unknown) {
      errorMessage = (err as Error).message || 'Failed to interrupt';
    }
  }

  async function handleSubmitTurn(textOverride?: string) {
    const textToSubmit = textOverride || textInput.trim();
    if (!textToSubmit) return;

    if (!currentSession) {
      await handleStartSession();
      if (!currentSession) return;
    }

    errorMessage = null;
    turnState = 'processing';

    try {
      const resp = await submitConversationTurn(
        currentSession.id,
        activeSpeaker,
        textToSubmit
      );

      turns = [...turns, resp.turn];
      textInput = '';
      activeSpeaker = activeSpeaker === 'SpeakerA' ? 'SpeakerB' : 'SpeakerA';

      if (resp.audio_samples && resp.audio_samples.length > 0) {
        turnState = 'speaking';
        playPcmSamples(resp.audio_samples, resp.sample_rate);
      } else {
        turnState = 'idle';
      }
    } catch (err: unknown) {
      turnState = 'idle';
      errorMessage = (err as Error).message || 'Failed to process turn';
    }
  }

  async function handleReplayTurn(turn: ConversationTurnItem) {
    try {
      stopCurrentAudio();
      isAudioPlaying = true;
      const ttsResp = await synthesizeSpeech({
        text: turn.translated_text,
        language: turn.target_lang,
        speed: 1.0,
        pitch: 1.0,
      });
      playPcmSamples(ttsResp.audio_samples, ttsResp.sample_rate);
    } catch (err: unknown) {
      errorMessage = (err as Error).message || 'Failed to replay audio';
      isAudioPlaying = false;
    }
  }

  function handleCopyTranscript() {
    if (turns.length === 0) return;
    const formatted = turns
      .map(
        (t) =>
          `[Turn ${t.turn_number} - ${t.speaker_id} (${t.source_lang} -> ${t.target_lang})]\nOriginal: ${t.source_text}\nTranslated: ${t.translated_text}`
      )
      .join('\n\n');

    navigator.clipboard.writeText(formatted);
    showNotice('Transcript copied to clipboard!');
  }

  function handleExportJson() {
    if (turns.length === 0) return;
    const dataStr = 'data:text/json;charset=utf-8,' + encodeURIComponent(JSON.stringify(turns, null, 2));
    const dlAnchor = document.createElement('a');
    dlAnchor.setAttribute('href', dataStr);
    dlAnchor.setAttribute('download', `linguaflow-conversation-${new Date().toISOString().slice(0, 10)}.json`);
    dlAnchor.click();
    showNotice('Exported conversation JSON.');
  }

  function showNotice(msg: string) {
    successNotice = msg;
    setTimeout(() => {
      successNotice = null;
    }, 3500);
  }

  function getLangMeta(code: LanguageCode) {
    return (
      SUPPORTED_LANGUAGES.find((l) => l.code === code) || {
        code,
        short: code.slice(0, 2),
        name: code,
        native: code,
        script: code,
      }
    );
  }

  const samplePhrases = [
    { lang: 'kn-IN', speaker: 'SpeakerA' as SpeakerId, text: 'ನನಗೆ ಜ್ವರ ಬಂದಿದೆ' },
    { lang: 'en-US', speaker: 'SpeakerB' as SpeakerId, text: 'Take this medicine twice a day.' },
    { lang: 'kn-IN', speaker: 'SpeakerA' as SpeakerId, text: 'ಧನ್ಯವಾದಗಳು ಡಾಕ್ಟರ್' },
    { lang: 'ta-IN', speaker: 'SpeakerA' as SpeakerId, text: 'வணக்கம், நீங்கள் எப்படி இருக்கிறீர்கள்?' },
    { lang: 'en-US', speaker: 'SpeakerB' as SpeakerId, text: 'Hello, I am doing very well, thank you.' },
  ];
</script>

<div class="conversation-container">
  <!-- Top Controls Bar -->
  <header class="conv-header glass-card">
    <div class="header-left">
      <div class="status-indicator">
        <span class="pulse-dot {currentSession ? 'active' : ''}"></span>
        <span class="status-label">
          {currentSession ? 'Session Live' : 'Ready to Start'}
        </span>
      </div>
      {#if currentSession}
        <span class="session-badge">Turns: {turns.length}</span>
      {/if}
    </div>

    <div class="header-actions">
      <button
        class="action-btn {isFaceToFaceFlipped ? 'btn-active' : ''}"
        on:click={() => (isFaceToFaceFlipped = !isFaceToFaceFlipped)}
        title="Flip Speaker 2 view 180° for Face-to-Face table mode"
        aria-label="Toggle Face-to-Face Table Mode"
      >
        <span class="btn-icon">🔄</span>
        <span>Table 180° Mode</span>
      </button>

      <button
        class="action-btn"
        on:click={handleSwapLanguages}
        title="Swap Source and Target Languages"
        aria-label="Swap Languages"
      >
        <span class="btn-icon">🔀</span>
        <span>Swap</span>
      </button>

      <button
        class="action-btn {isHandsFree ? 'btn-active' : ''}"
        on:click={() => (isHandsFree = !isHandsFree)}
        title="Hands-free auto turn switching with Voice Activity Detection"
      >
        <span class="btn-icon">🎙️</span>
        <span>Hands-Free {isHandsFree ? 'ON' : 'OFF'}</span>
      </button>

      {#if currentSession}
        <button class="action-btn btn-danger" on:click={handleStopSession}>
          <span>End Session</span>
        </button>
      {:else}
        <button class="action-btn btn-primary" on:click={handleStartSession}>
          <span>Start Session</span>
        </button>
      {/if}
    </div>
  </header>

  {#if successNotice}
    <div class="toast-notice success-notice">
      <span>✓ {successNotice}</span>
    </div>
  {/if}

  {#if errorMessage}
    {#if errorMessage.includes('model pack') || errorMessage.includes('not installed') || errorMessage.includes('Download speech pack')}
      <div class="speech-pack-banner">
        <div class="banner-content">
          <span class="banner-icon">📦</span>
          <div>
            <strong>Speech Model Pack Required</strong>
            <p>{errorMessage}</p>
          </div>
        </div>
        {#if onNavigate}
          <button class="btn-primary-action" on:click={() => onNavigate?.('downloads')}>
            Download Speech Pack in Model Manager ➔
          </button>
        {/if}
      </div>
    {:else}
      <div class="toast-notice error-notice">
        <span>⚠️ {errorMessage}</span>
      </div>
    {/if}
  {/if}

  <!-- Dual-Speaker Cards Section (Flippable for Face-to-Face table layout) -->
  <div class="dual-speakers-grid">
    <!-- Speaker 1 Card (Local) -->
    <div
      class="speaker-panel glass-card {activeSpeaker === 'SpeakerA' ? 'panel-active' : ''}"
      id="speaker-card-a"
    >
      <div class="panel-header">
        <div class="speaker-meta">
          <div class="speaker-avatar avatar-a">1</div>
          <div>
            <h3 class="speaker-name">Speaker 1 (Local)</h3>
            <span class="speaker-role">Active Participant</span>
          </div>
        </div>
        {#if activeSpeaker === 'SpeakerA'}
          <span class="turn-badge badge-active">Your Turn</span>
        {:else}
          <button
            class="turn-switch-btn"
            on:click={() => handleSwitchActiveSpeaker('SpeakerA')}
          >
            Switch to 1
          </button>
        {/if}
      </div>

      <div class="lang-selector-wrapper">
        <label for="speaker-a-lang" class="field-label">Language:</label>
        <select
          id="speaker-a-lang"
          bind:value={speakerALang}
          class="speaker-select"
          disabled={!!currentSession}
        >
          {#each SUPPORTED_LANGUAGES as lang}
            <option value={lang.code}>{lang.native} ({lang.name})</option>
          {/each}
        </select>
      </div>

      <div class="speaker-voice-preview">
        <span class="voice-tag">Voice: Ananya (16 kHz Indic TTS)</span>
      </div>
    </div>

    <!-- Center Divider & Interruption Trigger -->
    <div class="center-controls">
      <div class="duplex-indicator">
        <span class="duplex-icon">⇄</span>
        <span class="duplex-text">Real-Time</span>
      </div>

      {#if isAudioPlaying || turnState === 'speaking'}
        <button
          class="interrupt-btn pulsing-danger"
          on:click={handleInterrupt}
          title="Barge-In: Interrupt speech output instantly"
          aria-label="Interrupt speech playback"
        >
          <span class="int-icon">✋</span>
          <span>Barge-In (Halt)</span>
        </button>
      {/if}
    </div>

    <!-- Speaker 2 Card (Remote/Table Interlocutor, Flippable 180deg) -->
    <div
      class="speaker-panel glass-card {activeSpeaker === 'SpeakerB' ? 'panel-active' : ''} {isFaceToFaceFlipped ? 'flipped-180' : ''}"
      id="speaker-card-b"
    >
      <div class="panel-header">
        <div class="speaker-meta">
          <div class="speaker-avatar avatar-b">2</div>
          <div>
            <h3 class="speaker-name">Speaker 2 (Partner)</h3>
            <span class="speaker-role">Face-to-Face Partner</span>
          </div>
        </div>
        {#if activeSpeaker === 'SpeakerB'}
          <span class="turn-badge badge-active">Partner's Turn</span>
        {:else}
          <button
            class="turn-switch-btn"
            on:click={() => handleSwitchActiveSpeaker('SpeakerB')}
          >
            Switch to 2
          </button>
        {/if}
      </div>

      <div class="lang-selector-wrapper">
        <label for="speaker-b-lang" class="field-label">Language:</label>
        <select
          id="speaker-b-lang"
          bind:value={speakerBLang}
          class="speaker-select"
          disabled={!!currentSession}
        >
          {#each SUPPORTED_LANGUAGES as lang}
            <option value={lang.code}>{lang.native} ({lang.name})</option>
          {/each}
        </select>
      </div>

      <div class="speaker-voice-preview">
        <span class="voice-tag">Voice: Clara / David (16 kHz TTS)</span>
      </div>
    </div>
  </div>

  <!-- Conversation Dialogue Transcript Stream -->
  <section class="transcript-section glass-card" aria-label="Conversation Transcripts">
    <div class="transcript-header">
      <div class="transcript-title-box">
        <h2 class="section-title">Bilingual Conversation Stream</h2>
        <span class="transcript-sub">
          {turns.length} verified bilingual turn{turns.length === 1 ? '' : 's'}
        </span>
      </div>
      <div class="transcript-actions">
        <button class="mini-action-btn" on:click={handleCopyTranscript} title="Copy all turns">
          📋 Copy
        </button>
        <button class="mini-action-btn" on:click={handleExportJson} title="Export as JSON">
          💾 Export JSON
        </button>
      </div>
    </div>

    <div class="transcript-body" id="transcript-scroll-area">
      {#if turns.length === 0}
        <div class="empty-turns">
          <div class="empty-bubble">💬</div>
          <h3>Ready for Conversation</h3>
          <p>
            Choose your language above and press the large microphone button or click any suggested quick phrase below.
          </p>
          <div class="quick-phrases">
            {#each samplePhrases as item}
              <button
                class="quick-pill"
                on:click={() => {
                  activeSpeaker = item.speaker;
                  handleSubmitTurn(item.text);
                }}
              >
                <span class="pill-lang">[{item.lang.slice(0, 2)}]</span>
                <span class="pill-text">{item.text}</span>
              </button>
            {/each}
          </div>
        </div>
      {:else}
        <div class="turns-list">
          {#each turns as turn (turn.id)}
            <div
              class="turn-bubble {turn.speaker_id === 'SpeakerA' ? 'turn-a' : 'turn-b'} {turn.is_interrupted ? 'turn-interrupted' : ''}"
            >
              <div class="turn-bubble-header">
                <span class="turn-author">
                  {turn.speaker_id === 'SpeakerA' ? 'Speaker 1' : 'Speaker 2'}
                  <span class="turn-lang">({turn.source_lang} → {turn.target_lang})</span>
                </span>
                <div class="turn-header-actions">
                  {#if turn.is_interrupted}
                    <span class="interrupted-tag">Interrupted</span>
                  {/if}
                  <button
                    class="replay-btn"
                    on:click={() => handleReplayTurn(turn)}
                    title="Play / Replay translated speech (Authentic TTS)"
                    aria-label="Replay Audio"
                  >
                    🔊 Play TTS ({turn.audio_duration_ms}ms)
                  </button>
                </div>
              </div>

              <!-- Original Source Text -->
              <div class="turn-source-text">
                <span class="source-label">Original:</span>
                <p class="source-content script-text">{turn.source_text}</p>
              </div>

              <!-- Translated Target Text -->
              <div class="turn-target-text">
                <span class="target-label">Translated:</span>
                <p class="target-content script-text">{turn.translated_text}</p>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  </section>

  <!-- Interactive Mic & Push-To-Talk Control Bar -->
  <footer class="conv-footer glass-card">
    <div class="ptt-container">
      <div class="ptt-left">
        <span class="active-speaker-hint">
          Speaking: <strong>{activeSpeaker === 'SpeakerA' ? 'Speaker 1' : 'Speaker 2'}</strong>
        </span>
      </div>

      <!-- Main Action Microphone Button with Pulsing Audio Ring -->
      <div class="ptt-center">
        <button
          class="ptt-action-button {turnState !== 'idle' ? 'button-active pulsing-glow' : ''}"
          on:click={() => {
            const defaultText =
              activeSpeaker === 'SpeakerA' ? 'ನನಗೆ ಜ್ವರ ಬಂದಿದೆ' : 'Take this medicine twice a day.';
            handleSubmitTurn(defaultText);
          }}
          disabled={turnState === 'processing'}
          title="Push to speak turn"
          aria-label="Push to speak"
        >
          <div class="button-ring"></div>
          <span class="mic-glyph">🎙️</span>
          <span class="button-label">
            {#if turnState === 'processing'}
              Processing Translation & TTS...
            {:else if turnState === 'speaking'}
              Speaking (TTS Audio Out)...
            {:else}
              Speak Turn ({activeSpeaker === 'SpeakerA' ? 'Speaker 1' : 'Speaker 2'})
            {/if}
          </span>
        </button>
      </div>

      <!-- Manual Input Option -->
      <div class="ptt-right">
        <form
          class="quick-input-form"
          on:submit|preventDefault={() => handleSubmitTurn()}
        >
          <input
            type="text"
            bind:value={textInput}
            placeholder="Or type speech input here..."
            class="typed-input"
          />
          <button type="submit" class="send-btn" disabled={!textInput.trim()}>
            Send Turn
          </button>
        </form>
      </div>
    </div>
  </footer>
</div>

<style>
  .conversation-container {
    display: flex;
    flex-direction: column;
    gap: 16px;
    max-width: 1100px;
    margin: 0 auto;
    width: 100%;
    box-sizing: border-box;
  }

  .glass-card {
    background: rgba(22, 27, 34, 0.75);
    backdrop-filter: blur(12px);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 14px;
    padding: 16px 20px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.25);
  }

  /* Header */
  .conv-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-wrap: wrap;
    gap: 12px;
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .status-indicator {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .pulse-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: #6e7681;
    transition: background 0.3s ease;
  }

  .pulse-dot.active {
    background: #2ea043;
    box-shadow: 0 0 10px #2ea043;
    animation: pulse 1.8s infinite;
  }

  @keyframes pulse {
    0% { transform: scale(1); opacity: 1; }
    50% { transform: scale(1.3); opacity: 0.7; }
    100% { transform: scale(1); opacity: 1; }
  }

  .status-label {
    font-size: 0.9rem;
    font-weight: 600;
    color: #e6edf3;
  }

  .session-badge {
    background: rgba(56, 139, 253, 0.15);
    color: #58a6ff;
    padding: 2px 8px;
    border-radius: 12px;
    font-size: 0.8rem;
    font-weight: 500;
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-wrap: wrap;
  }

  .action-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 7px 14px;
    border-radius: 8px;
    font-size: 0.85rem;
    font-weight: 500;
    cursor: pointer;
    background: rgba(255, 255, 255, 0.06);
    color: #e6edf3;
    border: 1px solid rgba(255, 255, 255, 0.12);
    transition: all 0.2s ease;
  }

  .action-btn:hover {
    background: rgba(255, 255, 255, 0.12);
  }

  .action-btn.btn-active {
    background: rgba(56, 139, 253, 0.2);
    border-color: #58a6ff;
    color: #58a6ff;
  }

  .btn-primary {
    background: #238636;
    border-color: #2ea043;
    color: #fff;
  }
  .btn-primary:hover {
    background: #2ea043;
  }

  .btn-danger {
    background: rgba(248, 81, 73, 0.2);
    border-color: #f85149;
    color: #f85149;
  }
  .btn-danger:hover {
    background: #f85149;
    color: #fff;
  }

  /* Toasts */
  .speech-pack-banner {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    padding: 1rem 1.25rem;
    background: rgba(245, 158, 11, 0.12);
    border: 1px solid rgba(245, 158, 11, 0.35);
    border-radius: 8px;
    color: #fde68a;
    margin-bottom: 1.25rem;
    flex-wrap: wrap;
  }

  .banner-content {
    display: flex;
    align-items: flex-start;
    gap: 0.75rem;
  }

  .banner-icon {
    font-size: 1.5rem;
  }

  .banner-content strong {
    color: #fbbf24;
    display: block;
    margin-bottom: 0.2rem;
  }

  .banner-content p {
    margin: 0;
    font-size: 0.88rem;
    color: #fef3c7;
  }

  .btn-primary-action {
    background: linear-gradient(135deg, #f59e0b 0%, #d97706 100%);
    color: #1e1e2e;
    font-weight: 600;
    padding: 0.55rem 1rem;
    border-radius: 6px;
    border: none;
    cursor: pointer;
    font-size: 0.88rem;
    white-space: nowrap;
    transition: all 0.2s;
  }

  .btn-primary-action:hover {
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(245, 158, 11, 0.3);
  }

  .toast-notice {
    padding: 10px 16px;
    border-radius: 8px;
    font-size: 0.9rem;
  }
  .success-notice {
    background: rgba(46, 160, 67, 0.15);
    border: 1px solid #2ea043;
    color: #3fb950;
  }
  .error-notice {
    background: rgba(248, 81, 73, 0.15);
    border: 1px solid #f85149;
    color: #f85149;
  }

  /* Dual Speaker Panels */
  .dual-speakers-grid {
    display: grid;
    grid-template-columns: 1fr auto 1fr;
    gap: 16px;
    align-items: stretch;
  }

  @media (max-width: 768px) {
    .dual-speakers-grid {
      grid-template-columns: 1fr;
    }
  }

  .speaker-panel {
    display: flex;
    flex-direction: column;
    gap: 12px;
    border: 1px solid rgba(255, 255, 255, 0.08);
    transition: transform 0.4s ease, border-color 0.3s ease, box-shadow 0.3s ease;
  }

  .speaker-panel.panel-active {
    border-color: #58a6ff;
    box-shadow: 0 0 16px rgba(88, 166, 255, 0.2);
  }

  /* 180° Flip for Tabletop Mode */
  .speaker-panel.flipped-180 {
    transform: rotate(180deg);
  }

  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .speaker-meta {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .speaker-avatar {
    width: 34px;
    height: 34px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: 700;
    font-size: 0.9rem;
  }

  .avatar-a {
    background: linear-gradient(135deg, #1f6feb, #388bfd);
    color: #fff;
  }

  .avatar-b {
    background: linear-gradient(135deg, #8957e5, #a371f7);
    color: #fff;
  }

  .speaker-name {
    margin: 0;
    font-size: 0.95rem;
    font-weight: 600;
    color: #e6edf3;
  }

  .speaker-role {
    font-size: 0.75rem;
    color: #8b949e;
  }

  .turn-badge {
    padding: 3px 8px;
    border-radius: 12px;
    font-size: 0.75rem;
    font-weight: 600;
  }

  .badge-active {
    background: #238636;
    color: #fff;
  }

  .turn-switch-btn {
    background: transparent;
    border: 1px solid rgba(255, 255, 255, 0.15);
    color: #8b949e;
    font-size: 0.75rem;
    padding: 3px 8px;
    border-radius: 6px;
    cursor: pointer;
  }

  .turn-switch-btn:hover {
    color: #e6edf3;
    border-color: #58a6ff;
  }

  .lang-selector-wrapper {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .field-label {
    font-size: 0.8rem;
    color: #8b949e;
  }

  .speaker-select {
    flex: 1;
    background: rgba(13, 17, 23, 0.8);
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: 6px;
    color: #e6edf3;
    padding: 6px 10px;
    font-size: 0.85rem;
  }

  .speaker-voice-preview {
    font-size: 0.75rem;
    color: #8b949e;
    font-style: italic;
  }

  /* Center Controls */
  .center-controls {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
  }

  .duplex-indicator {
    display: flex;
    flex-direction: column;
    align-items: center;
    color: #8b949e;
    font-size: 0.75rem;
  }

  .duplex-icon {
    font-size: 1.6rem;
    color: #58a6ff;
  }

  .interrupt-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    background: #da3633;
    color: #fff;
    border: none;
    border-radius: 8px;
    padding: 8px 12px;
    font-size: 0.8rem;
    font-weight: 600;
    cursor: pointer;
    box-shadow: 0 0 12px rgba(218, 54, 51, 0.4);
  }

  .pulsing-danger {
    animation: danger-pulse 1s infinite alternate;
  }

  @keyframes danger-pulse {
    0% { transform: scale(1); }
    100% { transform: scale(1.06); box-shadow: 0 0 18px #f85149; }
  }

  /* Transcript Section */
  .transcript-section {
    display: flex;
    flex-direction: column;
    min-height: 280px;
    max-height: 440px;
  }

  .transcript-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
    padding-bottom: 10px;
    margin-bottom: 12px;
  }

  .section-title {
    margin: 0;
    font-size: 1.05rem;
    color: #e6edf3;
  }

  .transcript-sub {
    font-size: 0.8rem;
    color: #8b949e;
  }

  .transcript-actions {
    display: flex;
    gap: 8px;
  }

  .mini-action-btn {
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.1);
    color: #c9d1d9;
    padding: 4px 10px;
    border-radius: 6px;
    font-size: 0.8rem;
    cursor: pointer;
  }

  .mini-action-btn:hover {
    background: rgba(255, 255, 255, 0.12);
  }

  .transcript-body {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding-right: 6px;
  }

  .empty-turns {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    height: 100%;
    color: #8b949e;
    padding: 20px;
  }

  .empty-bubble {
    font-size: 2.5rem;
    margin-bottom: 8px;
  }

  .empty-turns h3 {
    margin: 4px 0;
    color: #e6edf3;
  }

  .empty-turns p {
    font-size: 0.88rem;
    max-width: 480px;
    margin-bottom: 16px;
  }

  .quick-phrases {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    justify-content: center;
    max-width: 600px;
  }

  .quick-pill {
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 20px;
    padding: 6px 14px;
    color: #c9d1d9;
    font-size: 0.82rem;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .quick-pill:hover {
    background: rgba(56, 139, 253, 0.2);
    border-color: #58a6ff;
    color: #fff;
  }

  .pill-lang {
    font-weight: 600;
    color: #58a6ff;
    margin-right: 4px;
  }

  /* Turn Bubbles */
  .turns-list {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .turn-bubble {
    padding: 14px 18px;
    border-radius: 12px;
    border: 1px solid rgba(255, 255, 255, 0.08);
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .turn-a {
    background: rgba(31, 111, 235, 0.08);
    border-left: 4px solid #388bfd;
  }

  .turn-b {
    background: rgba(137, 87, 229, 0.08);
    border-left: 4px solid #a371f7;
  }

  .turn-interrupted {
    border-right: 4px solid #f85149;
  }

  .turn-bubble-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 0.8rem;
    color: #8b949e;
  }

  .turn-author {
    font-weight: 600;
    color: #e6edf3;
  }

  .turn-lang {
    font-weight: 400;
    color: #8b949e;
    margin-left: 4px;
  }

  .turn-header-actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .interrupted-tag {
    background: rgba(248, 81, 73, 0.2);
    color: #f85149;
    padding: 2px 6px;
    border-radius: 6px;
    font-size: 0.7rem;
    font-weight: 600;
  }

  .replay-btn {
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.12);
    color: #58a6ff;
    padding: 3px 8px;
    border-radius: 6px;
    font-size: 0.75rem;
    font-weight: 500;
    cursor: pointer;
  }

  .replay-btn:hover {
    background: rgba(56, 139, 253, 0.2);
  }

  .turn-source-text,
  .turn-target-text {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .source-label,
  .target-label {
    font-size: 0.72rem;
    text-transform: uppercase;
    font-weight: 600;
    letter-spacing: 0.5px;
    color: #8b949e;
  }

  .target-label {
    color: #58a6ff;
  }

  .source-content,
  .target-content {
    margin: 0;
    font-size: 1.05rem;
    line-height: 1.45;
    color: #e6edf3;
  }

  .target-content {
    font-weight: 500;
    color: #ffffff;
  }

  /* Unicode Indic Font rendering */
  .script-text {
    font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Noto Sans Kannada', 'Noto Sans Tamil', sans-serif;
  }

  /* Footer & Push-to-Talk Bar */
  .conv-footer {
    padding: 14px 20px;
  }

  .ptt-container {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    flex-wrap: wrap;
  }

  .ptt-left {
    font-size: 0.85rem;
    color: #8b949e;
  }

  .ptt-center {
    display: flex;
    justify-content: center;
    flex: 1;
  }

  .ptt-action-button {
    position: relative;
    display: inline-flex;
    align-items: center;
    gap: 10px;
    background: linear-gradient(135deg, #238636, #2ea043);
    color: #fff;
    border: none;
    border-radius: 30px;
    padding: 12px 28px;
    font-size: 0.95rem;
    font-weight: 600;
    cursor: pointer;
    box-shadow: 0 4px 14px rgba(35, 134, 54, 0.4);
    transition: all 0.2s ease;
  }

  .ptt-action-button:hover {
    transform: translateY(-1px);
    box-shadow: 0 6px 18px rgba(35, 134, 54, 0.55);
  }

  .ptt-action-button:active {
    transform: translateY(1px);
  }

  .pulsing-glow {
    animation: glow-pulse 1.2s infinite alternate;
  }

  @keyframes glow-pulse {
    0% {
      box-shadow: 0 0 12px rgba(46, 160, 67, 0.5);
    }
    100% {
      box-shadow: 0 0 24px rgba(46, 160, 67, 0.9), 0 0 8px #fff;
    }
  }

  .mic-glyph {
    font-size: 1.2rem;
  }

  .ptt-right {
    display: flex;
    align-items: center;
  }

  .quick-input-form {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .typed-input {
    background: rgba(13, 17, 23, 0.8);
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: 8px;
    padding: 8px 12px;
    color: #e6edf3;
    font-size: 0.85rem;
    min-width: 220px;
  }

  .typed-input:focus {
    outline: none;
    border-color: #58a6ff;
  }

  .send-btn {
    background: rgba(56, 139, 253, 0.2);
    border: 1px solid #58a6ff;
    color: #58a6ff;
    border-radius: 8px;
    padding: 8px 14px;
    font-size: 0.85rem;
    font-weight: 500;
    cursor: pointer;
  }

  .send-btn:hover:not(:disabled) {
    background: #58a6ff;
    color: #fff;
  }

  .send-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
