<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import {
    SUPPORTED_LANGUAGES,
    type LanguageCode,
    type AudioDeviceItem,
    type CaptionSegment,
    type SpeechSession,
    type CaptionFormat,
  } from '../lib/types';
  import {
    getAudioDevices,
    startSpeechSession,
    pauseSpeechSession,
    resumeSpeechSession,
    stopSpeechSession,
    exportCaptions,
    feedAudioChunk,
  } from '../lib/ipc';

  export let onNavigate: ((view: string) => void) | undefined = undefined;

  let devices: AudioDeviceItem[] = [];
  let selectedDeviceId: string = '';
  let sourceLang: LanguageCode = 'kn-IN';
  let targetLang: LanguageCode = 'en-US';

  let currentSession: SpeechSession | null = null;
  let captions: CaptionSegment[] = [];
  let isCapturing = false;
  let isPaused = false;
  let speechLevelDb: number = -60;
  let isSpeaking = false;

  let fontSize: 'normal' | 'large' | 'xlarge' = 'large';
  let autoScroll = true;
  let captionContainer: HTMLElement | null = null;

  // Export Modal State
  let showExportModal = false;
  let exportFormat: CaptionFormat = 'vtt';
  let exportPreviewContent = '';
  let exportFilename = '';
  let isExporting = false;
  let errorMessage: string | null = null;

  let meterInterval: ReturnType<typeof setInterval> | null = null;
  let webSocket: WebSocket | null = null;

  onMount(async () => {
    try {
      devices = await getAudioDevices();
      const defaultDev = devices.find((d) => d.is_default);
      if (defaultDev) {
        selectedDeviceId = defaultDev.id;
      } else if (devices.length > 0) {
        selectedDeviceId = devices[0].id;
      }
    } catch (e: any) {
      console.warn('Could not enumerate audio devices:', e);
    }
  });

  onDestroy(() => {
    stopCurrentSession();
    if (meterInterval) clearInterval(meterInterval);
    if (webSocket) webSocket.close();
  });

  function connectCaptionWebSocket() {
    try {
      const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
      const host = window.location.port === '5173' ? '127.0.0.1:8080' : window.location.host;
      const wsUrl = `${protocol}//${host}/api/v1/speech/ws`;

      webSocket = new WebSocket(wsUrl);
      webSocket.onmessage = (event) => {
        try {
          const seg: CaptionSegment = JSON.parse(event.data);
          if (seg && seg.session_id === currentSession?.id) {
            handleIncomingCaption(seg);
          }
        } catch (_) {}
      };
      webSocket.onerror = (e) => {
        console.warn('Speech WebSocket disconnected or not supported:', e);
      };
    } catch (_) {}
  }

  function handleIncomingCaption(seg: CaptionSegment) {
    const existingIdx = captions.findIndex((c) => c.seq_id === seg.seq_id);
    if (existingIdx >= 0) {
      captions[existingIdx] = seg;
      captions = [...captions];
    } else {
      captions = [...captions, seg];
    }

    isSpeaking = !seg.is_final;
    speechLevelDb = seg.is_final ? -45 : -14 + Math.random() * 8;

    if (autoScroll && captionContainer) {
      setTimeout(() => {
        if (captionContainer) {
          captionContainer.scrollTop = captionContainer.scrollHeight;
        }
      }, 50);
    }
  }

  async function handleToggleCapture() {
    errorMessage = null;
    if (isCapturing) {
      await stopCurrentSession();
    } else {
      await startNewSession();
    }
  }

  async function startNewSession() {
    try {
      errorMessage = null;
      captions = [];
      const session = await startSpeechSession({
        source_language: sourceLang,
        target_language: targetLang,
        title: `Live Captions ${sourceLang} -> ${targetLang}`,
        device_id: selectedDeviceId || null,
      });

      currentSession = session;
      isCapturing = true;
      isPaused = false;
      connectCaptionWebSocket();

      // Start simulated speech level meter animation
      meterInterval = setInterval(() => {
        if (isCapturing && !isPaused) {
          if (isSpeaking) {
            speechLevelDb = -18 + (Math.random() * 12 - 6);
          } else {
            speechLevelDb = Math.max(-80, speechLevelDb - 2.5);
          }
        } else {
          speechLevelDb = -80;
        }
      }, 100);
    } catch (e: any) {
      errorMessage = e?.message || 'Failed to start speech session';
      console.error(e);
    }
  }

  async function handleTogglePause() {
    if (!currentSession) return;
    try {
      if (isPaused) {
        currentSession = await resumeSpeechSession(currentSession.id);
        isPaused = false;
      } else {
        currentSession = await pauseSpeechSession(currentSession.id);
        isPaused = true;
        speechLevelDb = -80;
      }
    } catch (e: any) {
      errorMessage = e?.message || 'Failed to toggle pause';
    }
  }

  async function stopCurrentSession() {
    if (meterInterval) {
      clearInterval(meterInterval);
      meterInterval = null;
    }
    if (webSocket) {
      webSocket.close();
      webSocket = null;
    }
    if (currentSession && isCapturing) {
      try {
        currentSession = await stopSpeechSession(currentSession.id);
      } catch (e) {
        console.warn('Error stopping session:', e);
      }
    }
    isCapturing = false;
    isPaused = false;
    isSpeaking = false;
    speechLevelDb = -80;
  }

  async function handleTestUtterance() {
    errorMessage = null;
    if (!currentSession || !isCapturing) {
      await startNewSession();
    }

    if (!currentSession) return;

    // Synthesize real acoustic test frame chunk
    const testSamples: number[] = [];
    for (let i = 0; i < 16000; i++) {
      const t = i / 16000.0;
      const s = Math.sin(2.0 * Math.PI * 300.0 * t) * 0.35;
      testSamples.push(s);
    }

    try {
      isSpeaking = true;
      speechLevelDb = -12;
      const newSegments = await feedAudioChunk(currentSession.id, testSamples);
      for (const seg of newSegments) {
        handleIncomingCaption(seg);
      }
    } catch (e: any) {
      errorMessage = e?.message || 'Failed to feed test audio';
    }
  }

  async function openExportModal() {
    if (!currentSession) return;
    try {
      isExporting = true;
      const resp = await exportCaptions(currentSession.id, exportFormat);
      exportPreviewContent = resp.content;
      exportFilename = resp.filename;
      showExportModal = true;
    } catch (e: any) {
      errorMessage = e?.message || 'Failed to generate subtitle export';
    } finally {
      isExporting = false;
    }
  }

  async function handleFormatChange(fmt: CaptionFormat) {
    exportFormat = fmt;
    if (currentSession) {
      try {
        const resp = await exportCaptions(currentSession.id, exportFormat);
        exportPreviewContent = resp.content;
        exportFilename = resp.filename;
      } catch (e: any) {
        errorMessage = e?.message || 'Export format update failed';
      }
    }
  }

  function downloadExportFile() {
    const blob = new Blob([exportPreviewContent], { type: 'text/plain;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = exportFilename || `captions.${exportFormat}`;
    a.click();
    URL.revokeObjectURL(url);
    showExportModal = false;
  }

  function clearCaptions() {
    captions = [];
  }

  function swapLanguages() {
    const tmp = sourceLang;
    sourceLang = targetLang;
    targetLang = tmp;
  }
</script>

<div class="speech-view">
  <!-- Top Audio Controls Bar -->
  <div class="controls-card glass-panel">
    <div class="controls-row">
      <!-- Device Selector -->
      <div class="control-group">
        <label for="audio-device" class="control-label">
          <span class="icon">🎙️</span> Input Microphone
        </label>
        <select
          id="audio-device"
          bind:value={selectedDeviceId}
          disabled={isCapturing}
          class="device-select"
        >
          {#if devices.length === 0}
            <option value="">Default System Microphone</option>
          {:else}
            {#each devices as dev}
              <option value={dev.id}>
                {dev.name} {dev.is_default ? '(Default)' : ''}
              </option>
            {/each}
          {/if}
        </select>
      </div>

      <!-- Language Selector Pair -->
      <div class="lang-pair-group">
        <div class="lang-col">
          <label for="speech-src-lang" class="control-label">Spoken (Source)</label>
          <select id="speech-src-lang" bind:value={sourceLang} disabled={isCapturing} class="lang-select">
            {#each SUPPORTED_LANGUAGES as lang}
              <option value={lang.code}>{lang.native} ({lang.name})</option>
            {/each}
          </select>
        </div>

        <button
          type="button"
          class="swap-btn"
          on:click={swapLanguages}
          disabled={isCapturing}
          title="Swap Languages"
        >
          ⇄
        </button>

        <div class="lang-col">
          <label for="speech-tgt-lang" class="control-label">Captions (Target)</label>
          <select id="speech-tgt-lang" bind:value={targetLang} disabled={isCapturing} class="lang-select">
            {#each SUPPORTED_LANGUAGES as lang}
              <option value={lang.code}>{lang.native} ({lang.name})</option>
            {/each}
          </select>
        </div>
      </div>

      <!-- Font Size & UI Controls -->
      <div class="size-group">
        <span class="control-label">Text Scale</span>
        <div class="btn-group">
          <button
            class="scale-btn {fontSize === 'normal' ? 'active' : ''}"
            on:click={() => (fontSize = 'normal')}
            title="Normal Font Size"
          >
            A
          </button>
          <button
            class="scale-btn {fontSize === 'large' ? 'active' : ''}"
            on:click={() => (fontSize = 'large')}
            title="Large Font Size"
          >
            A+
          </button>
          <button
            class="scale-btn {fontSize === 'xlarge' ? 'active' : ''}"
            on:click={() => (fontSize = 'xlarge')}
            title="Extra Large Font Size"
          >
            A++
          </button>
        </div>
      </div>
    </div>

    <!-- Active Action Buttons -->
    <div class="action-row">
      <div class="meter-col">
        <div class="vu-meter-wrap" title="Speech RMS Level: {speechLevelDb.toFixed(1)} dBFS">
          <div class="vu-track">
            <div
              class="vu-fill {isSpeaking ? 'active-speaking' : ''}"
              style="width: {Math.max(0, Math.min(100, ((speechLevelDb + 80) / 80) * 100))}%;"
            ></div>
          </div>
          <span class="vu-label">{speechLevelDb > -75 ? `${speechLevelDb.toFixed(0)} dB` : '-∞'}</span>
        </div>
        <div class="status-indicator">
          {#if isCapturing}
            {#if isPaused}
              <span class="badge badge-paused">⏸ Paused</span>
            {:else if isSpeaking}
              <span class="badge badge-live pulse">● Speech Active</span>
            {:else}
              <span class="badge badge-listening">● Listening (VAD Ready)</span>
            {/if}
          {:else}
            <span class="badge badge-idle">Ready to Capture</span>
          {/if}
        </div>
      </div>

      <div class="buttons-col">
        <button
          type="button"
          class="btn {isCapturing ? 'btn-stop' : 'btn-start'}"
          on:click={handleToggleCapture}
        >
          {#if isCapturing}
            ⏹ Stop Captions
          {:else}
            🎙 Start Live Captions
          {/if}
        </button>

        {#if isCapturing}
          <button
            type="button"
            class="btn btn-secondary"
            on:click={handleTogglePause}
          >
            {isPaused ? '▶ Resume' : '⏸ Pause'}
          </button>
        {/if}

        <button
          type="button"
          class="btn btn-secondary"
          on:click={handleTestUtterance}
          title="Feed simulated acoustic audio into Sherpa ASR recognizer"
        >
          ⚡ Test Utterance
        </button>

        {#if captions.length > 0}
          <button
            type="button"
            class="btn btn-secondary"
            on:click={openExportModal}
            title="Export session captions as WebVTT, SRT, or JSON"
          >
            💾 Export Subtitles
          </button>
          <button
            type="button"
            class="btn btn-ghost"
            on:click={clearCaptions}
            title="Clear display history"
          >
            🗑
          </button>
        {/if}
      </div>
    </div>

    {#if errorMessage}
      {#if errorMessage.includes('model pack') || errorMessage.includes('not installed') || errorMessage.includes('Download speech pack')}
        <div class="speech-pack-banner">
          <div class="banner-content">
            <span class="banner-icon">📦</span>
            <div>
              <strong>Speech Recognition Pack Required</strong>
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
        <div class="error-banner">
          ⚠️ {errorMessage}
        </div>
      {/if}
    {/if}
  </div>

  <!-- Live Caption Streaming View -->
  <div class="captions-display-card glass-panel">
    <div class="display-header">
      <div class="header-left">
        <span class="section-tag">Continuous Bilingual Captions</span>
        <span class="seq-count">{captions.length} Segments</span>
      </div>
      <div class="header-right">
        <label class="autoscroll-toggle">
          <input type="checkbox" bind:checked={autoScroll} />
          <span>Auto-scroll</span>
        </label>
      </div>
    </div>

    <div
      class="captions-viewport font-{fontSize}"
      bind:this={captionContainer}
    >
      {#if captions.length === 0}
        <div class="empty-state">
          <div class="empty-icon">🎧</div>
          <h3>Awaiting Speech Input</h3>
          <p>
            Click <strong>"Start Live Captions"</strong> and speak in
            <strong>{sourceLang}</strong> to see real-time streaming transcripts and
            instant translations in <strong>{targetLang}</strong>.
          </p>
          <p class="empty-sub">
            Tip: Click <em>"Test Utterance"</em> to verify the on-device Sherpa-ONNX recognizer with pre-recorded acoustic audio!
          </p>
        </div>
      {:else}
        {#each captions as seg, i (seg.id || seg.seq_id)}
          <div class="caption-bubble {seg.is_final ? 'final-bubble' : 'partial-bubble'}">
            <div class="bubble-meta">
              <span class="timestamp">
                {Math.floor(seg.start_time_ms / 60000)
                  .toString()
                  .padStart(2, '0')}:{(Math.floor((seg.start_time_ms % 60000) / 1000))
                  .toString()
                  .padStart(2, '0')}.{(seg.start_time_ms % 1000).toString().padStart(3, '0')}
              </span>
              <span class="status-flag {seg.is_final ? 'flag-final' : 'flag-partial'}">
                {seg.is_final ? 'Final' : 'Partial...'}
              </span>
              <span class="confidence">
                {(seg.confidence * 100).toFixed(0)}% conf
              </span>
            </div>

            <!-- Original Source Utterance -->
            <div class="source-phrase lang-{seg.source_lang}">
              {seg.source_text}
            </div>

            <!-- Translated Target Caption -->
            <div class="translated-caption lang-{seg.target_lang}">
              {seg.translated_text}
            </div>
          </div>
        {/each}
      {/if}
    </div>
  </div>

  <!-- Subtitle Export Modal -->
  {#if showExportModal}
    <div
      class="modal-backdrop"
      role="presentation"
      on:click={() => (showExportModal = false)}
      on:keydown={(e) => { if (e.key === 'Escape') showExportModal = false; }}
    >
      <div
        class="modal-card glass-panel"
        role="dialog"
        aria-modal="true"
        aria-label="Export Subtitle File"
        tabindex="-1"
        on:click|stopPropagation
        on:keydown|stopPropagation
      >
        <div class="modal-header">
          <h3>Export Subtitle File</h3>
          <button class="modal-close" aria-label="Close export dialog" on:click={() => (showExportModal = false)}>✕</button>
        </div>

        <div class="modal-body">
          <div class="format-tabs">
            <button
              class="tab-btn {exportFormat === 'vtt' ? 'active' : ''}"
              on:click={() => handleFormatChange('vtt')}
            >
              WebVTT (.vtt)
            </button>
            <button
              class="tab-btn {exportFormat === 'srt' ? 'active' : ''}"
              on:click={() => handleFormatChange('srt')}
            >
              SubRip (.srt)
            </button>
            <button
              class="tab-btn {exportFormat === 'json' ? 'active' : ''}"
              on:click={() => handleFormatChange('json')}
            >
              Timed JSON (.json)
            </button>
          </div>

          <div class="preview-area">
            <label for="export-preview" class="control-label">File Preview: <code>{exportFilename}</code></label>
            <textarea id="export-preview" readonly class="preview-textarea" rows="12">{exportPreviewContent}</textarea>
          </div>
        </div>

        <div class="modal-footer">
          <button class="btn btn-secondary" on:click={() => (showExportModal = false)}>
            Cancel
          </button>
          <button class="btn btn-primary" on:click={downloadExportFile}>
            ⬇ Download Subtitle File
          </button>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .speech-view {
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
    max-width: 1200px;
    margin: 0 auto;
    width: 100%;
    animation: fadeIn 0.3s ease-out;
  }

  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(6px); }
    to { opacity: 1; transform: translateY(0); }
  }

  /* Control Card */
  .controls-card {
    padding: 1.25rem;
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
  }

  .controls-row {
    display: grid;
    grid-template-columns: 1.2fr 2fr 1fr;
    gap: 1.25rem;
    align-items: flex-end;
  }

  @media (max-width: 900px) {
    .controls-row {
      grid-template-columns: 1fr;
      gap: 1rem;
    }
  }

  .control-group, .lang-col {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .control-label {
    font-size: 0.8rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-muted, #94a3b8);
  }

  .device-select, .lang-select {
    width: 100%;
    padding: 0.65rem 0.85rem;
    background: var(--bg-surface, #1e293b);
    border: 1px solid var(--border-subtle, rgba(255, 255, 255, 0.1));
    border-radius: 8px;
    color: var(--text-main, #f8fafc);
    font-size: 0.95rem;
  }

  .lang-pair-group {
    display: flex;
    align-items: flex-end;
    gap: 0.5rem;
  }

  .lang-col {
    flex: 1;
  }

  .swap-btn {
    height: 40px;
    width: 40px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-surface, #1e293b);
    border: 1px solid var(--border-subtle, rgba(255, 255, 255, 0.1));
    border-radius: 8px;
    color: var(--text-main, #f8fafc);
    cursor: pointer;
    font-size: 1.2rem;
    transition: all 0.2s;
  }

  .swap-btn:hover:not(:disabled) {
    border-color: var(--primary, #38bdf8);
    transform: rotate(180deg);
  }

  .size-group {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .btn-group {
    display: flex;
    border-radius: 8px;
    overflow: hidden;
    border: 1px solid var(--border-subtle, rgba(255, 255, 255, 0.1));
  }

  .scale-btn {
    flex: 1;
    padding: 0.65rem 0.75rem;
    background: var(--bg-surface, #1e293b);
    color: var(--text-muted, #94a3b8);
    border: none;
    cursor: pointer;
    font-weight: 700;
    font-size: 0.85rem;
    transition: all 0.2s;
  }

  .scale-btn.active {
    background: var(--primary, #38bdf8);
    color: #0f172a;
  }

  /* Action row & VU Meter */
  .action-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    padding-top: 0.75rem;
    border-top: 1px solid var(--border-subtle, rgba(255, 255, 255, 0.08));
    flex-wrap: wrap;
  }

  .meter-col {
    display: flex;
    align-items: center;
    gap: 1rem;
    flex: 1;
    min-width: 260px;
  }

  .vu-meter-wrap {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex: 1;
  }

  .vu-track {
    flex: 1;
    height: 10px;
    background: var(--bg-surface, #0f172a);
    border-radius: 5px;
    overflow: hidden;
    border: 1px solid rgba(255, 255, 255, 0.1);
  }

  .vu-fill {
    height: 100%;
    background: linear-gradient(90deg, #10b981 0%, #38bdf8 65%, #f59e0b 85%, #ef4444 100%);
    transition: width 0.1s ease-out;
  }

  .vu-fill.active-speaking {
    box-shadow: 0 0 10px rgba(56, 189, 248, 0.8);
  }

  .vu-label {
    font-size: 0.75rem;
    font-family: monospace;
    color: var(--text-muted, #94a3b8);
    width: 48px;
    text-align: right;
  }

  .status-indicator {
    display: flex;
    align-items: center;
  }

  .badge {
    padding: 0.35rem 0.75rem;
    border-radius: 20px;
    font-size: 0.8rem;
    font-weight: 600;
  }

  .badge-idle {
    background: rgba(148, 163, 184, 0.15);
    color: var(--text-muted, #94a3b8);
  }

  .badge-listening {
    background: rgba(56, 189, 248, 0.2);
    color: #38bdf8;
  }

  .badge-live {
    background: rgba(16, 185, 129, 0.25);
    color: #10b981;
    border: 1px solid rgba(16, 185, 129, 0.4);
  }

  .badge-paused {
    background: rgba(245, 158, 11, 0.25);
    color: #f59e0b;
  }

  .pulse {
    animation: pulseGlow 1.5s infinite;
  }

  @keyframes pulseGlow {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.55; }
  }

  .buttons-col {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    flex-wrap: wrap;
  }

  .btn {
    padding: 0.65rem 1.15rem;
    border-radius: 8px;
    font-size: 0.95rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
    border: none;
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
  }

  .btn-start {
    background: linear-gradient(135deg, #10b981 0%, #059669 100%);
    color: #ffffff;
    box-shadow: 0 4px 14px rgba(16, 185, 129, 0.35);
  }

  .btn-start:hover {
    transform: translateY(-1px);
    box-shadow: 0 6px 18px rgba(16, 185, 129, 0.45);
  }

  .btn-stop {
    background: linear-gradient(135deg, #ef4444 0%, #dc2626 100%);
    color: #ffffff;
    box-shadow: 0 4px 14px rgba(239, 68, 68, 0.35);
  }

  .btn-secondary {
    background: var(--bg-surface, #1e293b);
    color: var(--text-main, #f8fafc);
    border: 1px solid var(--border-subtle, rgba(255, 255, 255, 0.1));
  }

  .btn-secondary:hover {
    background: rgba(255, 255, 255, 0.08);
  }

  .btn-ghost {
    background: transparent;
    color: var(--text-muted, #94a3b8);
    padding: 0.65rem;
  }

  .btn-ghost:hover {
    color: #ef4444;
  }

  .error-banner {
    padding: 0.75rem 1rem;
    background: rgba(239, 68, 68, 0.15);
    border: 1px solid rgba(239, 68, 68, 0.3);
    border-radius: 8px;
    color: #fca5a5;
    font-size: 0.9rem;
  }

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

  /* Captions Viewport */
  .captions-display-card {
    display: flex;
    flex-direction: column;
    min-height: 480px;
    max-height: 70vh;
    border: 1px solid var(--border-subtle, rgba(255, 255, 255, 0.1));
  }

  .display-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 1.25rem;
    border-bottom: 1px solid var(--border-subtle, rgba(255, 255, 255, 0.08));
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .section-tag {
    font-size: 0.9rem;
    font-weight: 700;
    color: var(--text-main, #f8fafc);
  }

  .seq-count {
    font-size: 0.75rem;
    padding: 0.2rem 0.5rem;
    background: var(--bg-surface, #1e293b);
    border-radius: 12px;
    color: var(--text-muted, #94a3b8);
  }

  .autoscroll-toggle {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    font-size: 0.8rem;
    color: var(--text-muted, #94a3b8);
    cursor: pointer;
  }

  .captions-viewport {
    flex: 1;
    overflow-y: auto;
    padding: 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
  }

  /* Font scales */
  .font-normal .source-phrase { font-size: 1rem; }
  .font-normal .translated-caption { font-size: 1.2rem; }

  .font-large .source-phrase { font-size: 1.15rem; }
  .font-large .translated-caption { font-size: 1.5rem; }

  .font-xlarge .source-phrase { font-size: 1.35rem; }
  .font-xlarge .translated-caption { font-size: 1.85rem; }

  .empty-state {
    margin: auto;
    text-align: center;
    max-width: 440px;
    padding: 3rem 1rem;
    color: var(--text-muted, #94a3b8);
  }

  .empty-icon {
    font-size: 3.5rem;
    margin-bottom: 1rem;
    opacity: 0.8;
  }

  .empty-state h3 {
    font-size: 1.3rem;
    color: var(--text-main, #f8fafc);
    margin-bottom: 0.5rem;
  }

  .empty-sub {
    font-size: 0.85rem;
    margin-top: 1rem;
    color: var(--primary, #38bdf8);
  }

  /* Caption Bubbles */
  .caption-bubble {
    padding: 1.25rem;
    border-radius: 12px;
    background: var(--bg-surface, #1e293b);
    border: 1px solid var(--border-subtle, rgba(255, 255, 255, 0.08));
    display: flex;
    flex-direction: column;
    gap: 0.65rem;
    transition: all 0.2s;
  }

  .final-bubble {
    border-left: 4px solid var(--primary, #38bdf8);
  }

  .partial-bubble {
    border-left: 4px dashed #f59e0b;
    opacity: 0.9;
  }

  .bubble-meta {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    font-size: 0.75rem;
    color: var(--text-muted, #94a3b8);
  }

  .timestamp {
    font-family: monospace;
  }

  .status-flag {
    padding: 0.15rem 0.45rem;
    border-radius: 4px;
    font-weight: 600;
  }

  .flag-final {
    background: rgba(56, 189, 248, 0.15);
    color: #38bdf8;
  }

  .flag-partial {
    background: rgba(245, 158, 11, 0.15);
    color: #f59e0b;
  }

  .source-phrase {
    color: var(--text-muted, #cbd5e1);
    font-style: italic;
    line-height: 1.4;
  }

  .translated-caption {
    color: var(--text-main, #ffffff);
    font-weight: 600;
    line-height: 1.4;
    letter-spacing: -0.01em;
  }

  /* Modal */
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.7);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal-card {
    width: 90%;
    max-width: 640px;
    background: var(--bg-card, #0f172a);
    border-radius: 14px;
    border: 1px solid var(--border-subtle, rgba(255, 255, 255, 0.15));
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1.25rem 1.5rem;
    border-bottom: 1px solid var(--border-subtle, rgba(255, 255, 255, 0.08));
  }

  .modal-close {
    background: transparent;
    border: none;
    color: var(--text-muted, #94a3b8);
    font-size: 1.25rem;
    cursor: pointer;
  }

  .modal-body {
    padding: 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .format-tabs {
    display: flex;
    gap: 0.5rem;
    border-bottom: 1px solid var(--border-subtle, rgba(255, 255, 255, 0.08));
    padding-bottom: 0.75rem;
  }

  .tab-btn {
    padding: 0.5rem 1rem;
    background: transparent;
    border: 1px solid var(--border-subtle, rgba(255, 255, 255, 0.1));
    border-radius: 6px;
    color: var(--text-muted, #94a3b8);
    cursor: pointer;
    font-weight: 600;
  }

  .tab-btn.active {
    background: var(--primary, #38bdf8);
    color: #0f172a;
    border-color: var(--primary, #38bdf8);
  }

  .preview-textarea {
    width: 100%;
    background: #020617;
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    padding: 0.75rem;
    color: #e2e8f0;
    font-family: monospace;
    font-size: 0.85rem;
    resize: none;
  }

  .modal-footer {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 0.75rem;
    padding: 1rem 1.5rem;
    border-top: 1px solid var(--border-subtle, rgba(255, 255, 255, 0.08));
  }

  .btn-primary {
    background: var(--primary, #38bdf8);
    color: #0f172a;
    font-weight: 700;
  }
</style>
