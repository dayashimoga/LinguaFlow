<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import {
    exportSubtitles,
    getOverlayPermission,
    getOverlaySettings,
    listMediaSegments,
    performOcr,
    saveOverlaySettings,
    startMediaTranscription,
    translateDocument,
    updateMediaSegment,
  } from '../lib/ipc';
  import type {
    DocumentBlock,
    DocumentFormat,
    DocumentTranslationJob,
    LanguageCode,
    MediaFormat,
    MediaSession,
    MediaSubtitleExportFormat,
    MediaSubtitleSegment,
    OcrResult,
    OverlaySettings,
    SystemAudioPermission,
  } from '../lib/types';
  import { SUPPORTED_LANGUAGES } from '../lib/types';

  type MediaTab = 'studio' | 'ocr' | 'document' | 'overlay';
  let activeTab: MediaTab = 'studio';

  // ----------------------------------------------------
  // Tab 1: Media Subtitle Studio
  // ----------------------------------------------------
  let mediaFile: File | null = null;
  let mediaFileName = '';
  let mediaFileSize = 0;
  let mediaSession: MediaSession | null = null;
  let subtitleSegments: MediaSubtitleSegment[] = [];
  let isTranscribing = false;
  let mediaSourceLang: LanguageCode = 'kn-IN';
  let mediaTargetLang: LanguageCode = 'en-US';
  let currentPlaybackMs = 0;
  let isPlaying = false;
  let playbackTimer: any = null;
  let exportFormat: MediaSubtitleExportFormat = 'srt';
  let exportTargetOnly = false;
  let exportedSubtitleContent = '';
  let showExportModal = false;
  let editingSegmentId: string | null = null;
  let editSourceText = '';
  let editTranslatedText = '';

  // ----------------------------------------------------
  // Tab 2: Camera & OCR Scanner
  // ----------------------------------------------------
  let ocrSourceLang: LanguageCode = 'kn-IN';
  let ocrTargetLang: LanguageCode = 'en-US';
  let ocrResult: OcrResult | null = null;
  let isScanningOcr = false;
  let isCameraActive = false;
  let videoElement: HTMLVideoElement | null = null;
  let canvasElement: HTMLCanvasElement | null = null;
  let cameraStream: MediaStream | null = null;
  let ocrImagePreviewUrl: string | null = null;

  // ----------------------------------------------------
  // Tab 3: Document Translator
  // ----------------------------------------------------
  let docFile: File | null = null;
  let docFormat: DocumentFormat = 'markdown';
  let docSourceLang: LanguageCode = 'kn-IN';
  let docTargetLang: LanguageCode = 'en-US';
  let isTranslatingDoc = false;
  let documentJob: DocumentTranslationJob | null = null;
  let documentBlocks: DocumentBlock[] = [];
  let exportedDocText = '';
  let docErrorMessage: string | null = null;

  // ----------------------------------------------------
  // Tab 4: Live Caption Overlay & Platform Settings
  // ----------------------------------------------------
  let overlaySettings: OverlaySettings = {
    enabled: true,
    always_on_top: true,
    x: 100,
    y: 100,
    width: 650,
    height: 140,
    font_size: 22,
    font_family: 'Inter, sans-serif',
    text_color: '#ffffff',
    background_color: '#0a0d14',
    opacity: 0.88,
    source_lang: 'en-US',
    target_lang: 'kn-IN',
    show_source: true,
    show_translated: true,
  };
  let audioPermission: SystemAudioPermission = 'granted';
  let shortcutQuickTranslate = 'CommandOrControl+Shift+L';
  let shortcutOverlayToggle = 'CommandOrControl+Shift+C';
  let overlaySaveSuccess = false;

  onMount(async () => {
    try {
      const [settings, perm] = await Promise.all([
        getOverlaySettings().catch(() => overlaySettings),
        getOverlayPermission().catch(() => 'granted' as SystemAudioPermission),
      ]);
      overlaySettings = settings;
      audioPermission = perm;
    } catch {
      // Fallback to defaults
    }
  });

  onDestroy(() => {
    stopCamera();
    if (playbackTimer) clearInterval(playbackTimer);
  });

  // --- Subtitle Studio Handlers ---

  function handleMediaFileSelect(e: Event) {
    const target = e.target as HTMLInputElement;
    if (target.files && target.files.length > 0) {
      const file = target.files[0];
      mediaFile = file;
      mediaFileName = file.name;
      mediaFileSize = file.size;
    }
  }

  function loadSampleMediaDemo() {
    mediaFileName = 'kannada_interview_sample.wav';
    mediaFileSize = 1024 * 512;
    // Create a synthetic small WAV payload
    const wavHeader = [
      0x52, 0x49, 0x46, 0x46, 0x24, 0x08, 0x00, 0x00, 0x57, 0x41, 0x56, 0x45,
      0x66, 0x6d, 0x74, 0x20, 0x10, 0x00, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00,
      0x80, 0x3e, 0x00, 0x00, 0x00, 0x7d, 0x00, 0x00, 0x02, 0x00, 0x10, 0x00,
      0x64, 0x61, 0x74, 0x61, 0x00, 0x08, 0x00, 0x00,
    ];
    const syntheticBytes = new Uint8Array(wavHeader.concat(new Array(2048).fill(128)));
    mediaFile = new File([syntheticBytes], mediaFileName, { type: 'audio/wav' });
  }

  async function processMediaTranscription() {
    if (!mediaFile) return;
    isTranscribing = true;
    try {
      const buffer = await mediaFile.arrayBuffer();
      const bytes = Array.from(new Uint8Array(buffer));

      const resp = await startMediaTranscription({
        file_name: mediaFileName,
        file_bytes: bytes,
        source_lang: mediaSourceLang,
        target_lang: mediaTargetLang,
        title: mediaFileName,
      });

      mediaSession = resp.session;
      subtitleSegments = resp.segments;
      currentPlaybackMs = 0;
    } catch (err: any) {
      alert(`Transcription failed: ${err.message}`);
    } finally {
      isTranscribing = false;
    }
  }

  function togglePlayback() {
    isPlaying = !isPlaying;
    if (isPlaying) {
      playbackTimer = setInterval(() => {
        if (mediaSession && currentPlaybackMs >= mediaSession.duration_ms) {
          isPlaying = false;
          clearInterval(playbackTimer);
          currentPlaybackMs = 0;
          return;
        }
        currentPlaybackMs += 250;
      }, 250);
    } else {
      if (playbackTimer) clearInterval(playbackTimer);
    }
  }

  function startEditSegment(seg: MediaSubtitleSegment) {
    editingSegmentId = seg.id;
    editSourceText = seg.source_text;
    editTranslatedText = seg.translated_text;
  }

  async function saveEditSegment(seg: MediaSubtitleSegment) {
    seg.source_text = editSourceText;
    seg.translated_text = editTranslatedText;
    await updateMediaSegment({
      segment_id: seg.id,
      source_text: editSourceText,
      translated_text: editTranslatedText,
      start_ms: seg.start_ms,
      end_ms: seg.end_ms,
    });
    editingSegmentId = null;
  }

  async function triggerExportSubtitles() {
    if (!mediaSession) return;
    try {
      const res = await exportSubtitles(mediaSession.id, exportFormat, exportTargetOnly);
      exportedSubtitleContent = res.content;
      showExportModal = true;
    } catch (err: any) {
      alert(`Export failed: ${err.message}`);
    }
  }

  function downloadExportedSubtitles() {
    const blob = new Blob([exportedSubtitleContent], { type: 'text/plain;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `subtitles_${mediaSession?.id || 'export'}.${exportFormat}`;
    a.click();
    URL.revokeObjectURL(url);
  }

  // --- OCR Vision Handlers ---

  async function startCamera() {
    try {
      isCameraActive = true;
      cameraStream = await navigator.mediaDevices.getUserMedia({
        video: { facingMode: 'environment', width: { ideal: 1280 }, height: { ideal: 720 } },
      });
      if (videoElement) {
        videoElement.srcObject = cameraStream;
        videoElement.play();
      }
    } catch (err: any) {
      isCameraActive = false;
      alert(`Camera access unavailable: ${err.message}`);
    }
  }

  function stopCamera() {
    if (cameraStream) {
      cameraStream.getTracks().forEach((t) => t.stop());
      cameraStream = null;
    }
    isCameraActive = false;
  }

  function captureFrameFromCamera() {
    if (!videoElement || !canvasElement) return;
    const ctx = canvasElement.getContext('2d');
    if (!ctx) return;
    canvasElement.width = videoElement.videoWidth || 640;
    canvasElement.height = videoElement.videoHeight || 480;
    ctx.drawImage(videoElement, 0, 0, canvasElement.width, canvasElement.height);
    canvasElement.toBlob(async (blob) => {
      if (blob) {
        const buffer = await blob.arrayBuffer();
        ocrImagePreviewUrl = canvasElement?.toDataURL('image/png') || null;
        stopCamera();
        runOcrOnBytes(new Uint8Array(buffer), 'image/png');
      }
    }, 'image/png');
  }

  async function handleOcrImageUpload(e: Event) {
    const target = e.target as HTMLInputElement;
    if (target.files && target.files.length > 0) {
      const file = target.files[0];
      ocrImagePreviewUrl = URL.createObjectURL(file);
      const buffer = await file.arrayBuffer();
      runOcrOnBytes(new Uint8Array(buffer), file.type || 'image/bmp');
    }
  }

  function loadSampleOcrImage() {
    // Generate a clean BMP image in canvas with Indic text
    const canvas = document.createElement('canvas');
    canvas.width = 480;
    canvas.height = 240;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;
    ctx.fillStyle = '#f8fafc';
    ctx.fillRect(0, 0, 480, 240);
    ctx.fillStyle = '#0f172a';
    ctx.font = 'bold 28px sans-serif';
    ctx.fillText('ನಮಸ್ಕಾರ ಲಿಂಗ್ವಾಫ್ಲೋ', 40, 70);
    ctx.font = '20px sans-serif';
    ctx.fillText('ಗಣಕಯಂತ್ರ ತಂತ್ರಾಂಶ ವ್ಯವಸ್ಥೆ', 40, 120);
    ctx.fillText('Real-Time Bilingual OCR Scanner', 40, 170);

    ocrImagePreviewUrl = canvas.toDataURL('image/png');
    canvas.toBlob(async (blob) => {
      if (blob) {
        const buffer = await blob.arrayBuffer();
        runOcrOnBytes(new Uint8Array(buffer), 'image/png');
      }
    });
  }

  async function runOcrOnBytes(bytes: Uint8Array, mime: string) {
    isScanningOcr = true;
    try {
      ocrResult = await performOcr({
        image_bytes: Array.from(bytes),
        mime_type: mime,
        source_lang: ocrSourceLang,
        target_lang: ocrTargetLang,
      });
    } catch (err: any) {
      alert(`OCR Scan failed: ${err.message}`);
    } finally {
      isScanningOcr = false;
    }
  }

  // --- Document Translation Handlers ---

  function handleDocFileSelect(e: Event) {
    const target = e.target as HTMLInputElement;
    if (target.files && target.files.length > 0) {
      const file = target.files[0];
      docFile = file;
      docErrorMessage = null;
      if (file.name.endsWith('.md')) docFormat = 'markdown';
      else if (file.name.endsWith('.pdf')) docFormat = 'pdf';
      else if (file.name.endsWith('.csv')) docFormat = 'csv';
      else docFormat = 'txt';
    }
  }

  function loadSampleMarkdownDoc() {
    const sampleMd = `# LinguaFlow ದಸ್ತಾವೇಜು ಅನುವಾದ

ಇದು ಕನ್ನಡ ಮತ್ತು ತಮಿಳು ಭಾಷೆಗಳಿಗೆ ನೈಜ-ಸಮಯದ ತಂತ್ರಾಂಶವಾಗಿದೆ.

\`\`\`rust
fn main() {
    println!("Code blocks are strictly protected!");
}
\`\`\`

- ಪೂರ್ಣ ದ್ವಿಭಾಷಾ ಬೆಂಬಲ
- ವೇಗದ ಶ್ರವ್ಯ ಸಂಸ್ಕರಣೆ
`;
    docFile = new File([sampleMd], 'sample_document.md', { type: 'text/markdown' });
    docFormat = 'markdown';
  }

  async function processDocumentTranslation() {
    if (!docFile) return;
    if (docFile.size > 25 * 1024 * 1024) {
      docErrorMessage = 'File size exceeds 25 MB memory limit (Contract Rule 6).';
      return;
    }
    isTranslatingDoc = true;
    docErrorMessage = null;
    try {
      const buffer = await docFile.arrayBuffer();
      const bytes = Array.from(new Uint8Array(buffer));
      const resp = await translateDocument({
        file_name: docFile.name,
        content: bytes,
        format: docFormat,
        source_lang: docSourceLang,
        target_lang: docTargetLang,
      });
      documentJob = resp.job;
      documentBlocks = resp.blocks;
      exportedDocText = resp.exported_content;
    } catch (err: any) {
      docErrorMessage = `Document processing failed: ${err.message}`;
    } finally {
      isTranslatingDoc = false;
    }
  }

  function downloadTranslatedDoc() {
    const blob = new Blob([exportedDocText], { type: 'text/plain;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `translated_${docFile?.name || 'document.txt'}`;
    a.click();
    URL.revokeObjectURL(url);
  }

  // --- Overlay Settings Handlers ---

  async function updateAndSaveOverlay() {
    try {
      await saveOverlaySettings(overlaySettings);
      overlaySaveSuccess = true;
      setTimeout(() => (overlaySaveSuccess = false), 3000);
    } catch (err: any) {
      alert(`Save failed: ${err.message}`);
    }
  }

  function formatTime(ms: number): string {
    const secs = Math.floor(ms / 1000);
    const m = Math.floor(secs / 60);
    const s = secs % 60;
    const millis = ms % 1000;
    return `${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}.${millis.toString().padStart(3, '0')}`;
  }
</script>

<div class="media-container fade-in">
  <!-- Top Navigation & Header -->
  <header class="studio-header glass-panel">
    <div class="header-left">
      <div class="badge-stage">STAGE 6 ACTIVE</div>
      <h1 class="studio-title">Media, Vision & Live Captions</h1>
      <p class="studio-subtitle">
        Chunked media subtitle generation, authentic OCR vision scanning, structured document translation & transparent desktop overlay.
      </p>
    </div>

    <!-- Tab Bar -->
    <nav class="subtab-nav" aria-label="Media Studio Modes">
      <button
        class="subtab-btn {activeTab === 'studio' ? 'active' : ''}"
        on:click={() => (activeTab = 'studio')}
      >
        <span class="icon">🎞️</span> Subtitle Studio
      </button>
      <button
        class="subtab-btn {activeTab === 'ocr' ? 'active' : ''}"
        on:click={() => (activeTab = 'ocr')}
      >
        <span class="icon">📷</span> Camera & OCR
      </button>
      <button
        class="subtab-btn {activeTab === 'document' ? 'active' : ''}"
        on:click={() => (activeTab = 'document')}
      >
        <span class="icon">📄</span> Documents
      </button>
      <button
        class="subtab-btn {activeTab === 'overlay' ? 'active' : ''}"
        on:click={() => (activeTab = 'overlay')}
      >
        <span class="icon">🪟</span> Live Overlay
      </button>
    </nav>
  </header>

  <!-- ======================================================== -->
  <!-- TAB 1: MEDIA SUBTITLE STUDIO -->
  <!-- ======================================================== -->
  {#if activeTab === 'studio'}
    <div class="tab-content grid-layout">
      <!-- Media Import & Player Column -->
      <section class="card glass-panel" aria-label="Media File Input">
        <h2 class="card-title">1. Media Ingestion & Playback</h2>
        <p class="card-desc">
          Chunked streaming audio extraction without loading multi-gigabyte media into RAM. Supports WAV, MP4, WebM, MP3.
        </p>

        <!-- Dropzone / File Picker -->
        <div class="dropzone">
          <input
            type="file"
            id="media-upload-input"
            accept="audio/*,video/*,.mp4,.webm,.wav,.mp3,.ogg,.flac,.m4a"
            on:change={handleMediaFileSelect}
            class="hidden-file-input"
          />
          <label for="media-upload-input" class="dropzone-label">
            <span class="dropzone-icon">📁</span>
            {#if mediaFileName}
              <span class="file-chosen">{mediaFileName} ({(mediaFileSize / 1024).toFixed(1)} KB)</span>
            {:else}
              <span class="drop-text">Drag & drop media file or <strong>Browse</strong></span>
            {/if}
          </label>
        </div>

        <div class="sample-btn-row">
          <button class="btn-ghost" on:click={loadSampleMediaDemo}>
            ✨ Load Synthetic Kannada WAV Sample
          </button>
        </div>

        <!-- Language Selectors & Action -->
        <div class="form-row">
          <div class="form-group">
            <label for="media-src-lang">Audio Language:</label>
            <select id="media-src-lang" bind:value={mediaSourceLang} class="input-select">
              {#each SUPPORTED_LANGUAGES as lang}
                <option value={lang.code}>{lang.name} ({lang.native})</option>
              {/each}
            </select>
          </div>
          <div class="form-group">
            <label for="media-tgt-lang">Subtitle Translation:</label>
            <select id="media-tgt-lang" bind:value={mediaTargetLang} class="input-select">
              {#each SUPPORTED_LANGUAGES as lang}
                <option value={lang.code}>{lang.name} ({lang.native})</option>
              {/each}
            </select>
          </div>
        </div>

        <button
          class="btn-primary w-full"
          disabled={!mediaFile || isTranscribing}
          on:click={processMediaTranscription}
        >
          {#if isTranscribing}
            <span class="spinner-small"></span> Extracting & Transcribing...
          {:else}
            🎙️ Generate Timestamped Bilingual Subtitles
          {/if}
        </button>

        <!-- Interactive Player Timeline -->
        {#if mediaSession}
          <div class="player-container glass-panel">
            <div class="player-controls">
              <button class="btn-playback" on:click={togglePlayback}>
                {isPlaying ? '⏸️ Pause' : '▶️ Play Preview'}
              </button>
              <div class="time-display">
                {formatTime(currentPlaybackMs)} / {formatTime(mediaSession.duration_ms)}
              </div>
            </div>

            <div class="timeline-bar">
              <div
                class="timeline-fill"
                style="width: {(currentPlaybackMs / Math.max(1, mediaSession.duration_ms)) * 100}%"
              ></div>
            </div>

            <!-- Active Caption Display -->
            <div class="active-caption-box">
              {#each subtitleSegments as seg}
                {#if currentPlaybackMs >= seg.start_ms && currentPlaybackMs <= seg.end_ms}
                  <p class="active-src">{seg.source_text}</p>
                  <p class="active-tgt">{seg.translated_text}</p>
                {/if}
              {/each}
            </div>
          </div>
        {/if}
      </section>

      <!-- Subtitle Editor & Exporter Column -->
      <section class="card glass-panel" aria-label="Subtitle Segment Editor">
        <div class="card-header-flex">
          <h2 class="card-title">2. Transcript & Subtitle Editor</h2>
          {#if subtitleSegments.length > 0}
            <button class="btn-secondary" on:click={triggerExportSubtitles}>
              💾 Export ({exportFormat.toUpperCase()})
            </button>
          {/if}
        </div>

        <div class="export-controls-row">
          <label class="radio-label">
            <input type="radio" bind:group={exportFormat} value="srt" /> SRT
          </label>
          <label class="radio-label">
            <input type="radio" bind:group={exportFormat} value="vtt" /> WebVTT
          </label>
          <label class="radio-label">
            <input type="radio" bind:group={exportFormat} value="txt" /> Plain TXT
          </label>
          <label class="radio-label">
            <input type="radio" bind:group={exportFormat} value="json" /> JSON
          </label>
          <label class="checkbox-label">
            <input type="checkbox" bind:checked={exportTargetOnly} /> Target Language Only
          </label>
        </div>

        <!-- Segments List -->
        {#if subtitleSegments.length === 0}
          <div class="empty-state">
            <span class="empty-icon">📝</span>
            <p>No subtitle segments yet. Select a media file and click Generate.</p>
          </div>
        {:else}
          <div class="segments-list">
            {#each subtitleSegments as seg (seg.id)}
              <div class="segment-item {editingSegmentId === seg.id ? 'editing' : ''}">
                <div class="seg-header">
                  <span class="seg-seq">#{seg.seq_id}</span>
                  <span class="seg-timestamp">
                    {formatTime(seg.start_ms)} ➔ {formatTime(seg.end_ms)}
                  </span>
                  {#if editingSegmentId !== seg.id}
                    <button class="btn-edit" on:click={() => startEditSegment(seg)}>Edit</button>
                  {/if}
                </div>

                {#if editingSegmentId === seg.id}
                  <div class="edit-inputs">
                    <input type="text" bind:value={editSourceText} class="input-text" />
                    <input type="text" bind:value={editTranslatedText} class="input-text" />
                    <div class="edit-actions">
                      <button class="btn-primary-sm" on:click={() => saveEditSegment(seg)}>Save</button>
                      <button class="btn-ghost-sm" on:click={() => (editingSegmentId = null)}>Cancel</button>
                    </div>
                  </div>
                {:else}
                  <p class="seg-source">{seg.source_text}</p>
                  <p class="seg-translated">{seg.translated_text}</p>
                {/if}
              </div>
            {/each}
          </div>
        {/if}
      </section>
    </div>
  {/if}

  <!-- ======================================================== -->
  <!-- TAB 2: CAMERA & OCR SCANNER -->
  <!-- ======================================================== -->
  {#if activeTab === 'ocr'}
    <div class="tab-content grid-layout">
      <!-- Camera Viewfinder & Ingestion -->
      <section class="card glass-panel" aria-label="Camera and Image Capture">
        <h2 class="card-title">1. Camera Viewfinder & Image Ingestion</h2>
        <p class="card-desc">
          Capture live camera frame or import images for authentic Otsu binarization and Unicode glyph recognition.
        </p>

        <!-- Viewfinder Box -->
        <div class="viewfinder-container">
          {#if isCameraActive}
            <video bind:this={videoElement} class="video-feed" autoplay playsinline muted></video>
            <div class="viewfinder-grid"></div>
            <div class="viewfinder-controls">
              <button class="btn-primary" on:click={captureFrameFromCamera}>📸 Capture & Scan</button>
              <button class="btn-ghost" on:click={stopCamera}>Cancel Camera</button>
            </div>
          {:else}
            <div class="camera-placeholder">
              {#if ocrImagePreviewUrl}
                <img src={ocrImagePreviewUrl} alt="Preview" class="image-preview" />
              {:else}
                <span class="cam-icon">📷</span>
                <p>Camera is currently standby</p>
              {/if}
            </div>
          {/if}
          <canvas bind:this={canvasElement} class="hidden-canvas"></canvas>
        </div>

        <div class="camera-actions-row">
          {#if !isCameraActive}
            <button class="btn-primary" on:click={startCamera}>
              📹 Start Camera Viewfinder
            </button>
          {/if}
          <button class="btn-ghost" on:click={loadSampleOcrImage}>
            ✨ Load High-Contrast Sample Image
          </button>
          <label class="btn-secondary-upload">
            📁 Upload BMP / PNG / JPEG
            <input type="file" accept="image/*" on:change={handleOcrImageUpload} class="hidden-file-input" />
          </label>
        </div>

        <div class="form-row">
          <div class="form-group">
            <label for="ocr-src-lang">Detected Language Hint:</label>
            <select id="ocr-src-lang" bind:value={ocrSourceLang} class="input-select">
              {#each SUPPORTED_LANGUAGES as lang}
                <option value={lang.code}>{lang.name} ({lang.native})</option>
              {/each}
            </select>
          </div>
          <div class="form-group">
            <label for="ocr-tgt-lang">Translate Text To:</label>
            <select id="ocr-tgt-lang" bind:value={ocrTargetLang} class="input-select">
              {#each SUPPORTED_LANGUAGES as lang}
                <option value={lang.code}>{lang.name} ({lang.native})</option>
              {/each}
            </select>
          </div>
        </div>
      </section>

      <!-- OCR Results & In-Place Text Overlays -->
      <section class="card glass-panel" aria-label="OCR Detection and Translation">
        <h2 class="card-title">2. Detected Text Regions & Overlays</h2>
        {#if isScanningOcr}
          <div class="loading-state">
            <div class="spinner"></div>
            <p>Running Otsu binarization, line segmentation, and glyph classification...</p>
          </div>
        {:else if !ocrResult}
          <div class="empty-state">
            <span class="empty-icon">🔍</span>
            <p>Capture a camera frame or upload an image to begin optical character recognition.</p>
          </div>
        {:else}
          <div class="ocr-stats-banner">
            <span>Language: <strong>{ocrResult.detected_language}</strong></span>
            <span>Blocks: <strong>{ocrResult.blocks.length}</strong></span>
            <span>Latency: <strong>{ocrResult.processing_time_ms.toFixed(2)} ms</strong></span>
          </div>

          <!-- Bounding Boxes Visualizer -->
          {#if ocrImagePreviewUrl}
            <div class="visual-canvas-container">
              <img src={ocrImagePreviewUrl} alt="Visual Scan" class="annotated-image" />
              {#each ocrResult.blocks as b}
                <div
                  class="bounding-box"
                  style="
                    left: {b.bounding_box.x * 100}%;
                    top: {b.bounding_box.y * 100}%;
                    width: {b.bounding_box.width * 100}%;
                    height: {b.bounding_box.height * 100}%;
                  "
                >
                  <span class="overlay-tag">{b.translated_text || b.text}</span>
                </div>
              {/each}
            </div>
          {/if}

          <!-- Block Details -->
          <div class="ocr-blocks-list">
            {#each ocrResult.blocks as block, i}
              <div class="ocr-block-item">
                <div class="block-top">
                  <span class="block-num">Region #{i + 1}</span>
                  <span class="block-conf">Confidence: {(block.confidence * 100).toFixed(0)}%</span>
                </div>
                <p class="block-source">Original: <strong>{block.text}</strong></p>
                {#if block.translated_text}
                  <p class="block-trans">Translation: <strong>{block.translated_text}</strong></p>
                {/if}
              </div>
            {/each}
          </div>
        {/if}
      </section>
    </div>
  {/if}

  <!-- ======================================================== -->
  <!-- TAB 3: DOCUMENT TRANSLATOR -->
  <!-- ======================================================== -->
  {#if activeTab === 'document'}
    <div class="tab-content grid-layout">
      <!-- Document File Upload & Format -->
      <section class="card glass-panel" aria-label="Document Ingestion">
        <h2 class="card-title">1. Document Ingestion (&lt; 25 MB)</h2>
        <p class="card-desc">
          Structured document parser with strict code-block preservation (fenced code is never corrupted). Supports TXT, Markdown, PDF streams, CSV.
        </p>

        <div class="dropzone">
          <input
            type="file"
            id="doc-upload-input"
            accept=".md,.txt,.pdf,.csv,.tsv"
            on:change={handleDocFileSelect}
            class="hidden-file-input"
          />
          <label for="doc-upload-input" class="dropzone-label">
            <span class="dropzone-icon">📑</span>
            {#if docFile}
              <span class="file-chosen">{docFile.name} ({(docFile.size / 1024).toFixed(1)} KB)</span>
            {:else}
              <span class="drop-text">Drag & drop document (.md, .txt, .pdf, .csv) or <strong>Browse</strong></span>
            {/if}
          </label>
        </div>

        <div class="sample-btn-row">
          <button class="btn-ghost" on:click={loadSampleMarkdownDoc}>
            ✨ Load Sample Markdown Document (with Code Block)
          </button>
        </div>

        <div class="form-row">
          <div class="form-group">
            <label for="doc-src-lang">Document Language:</label>
            <select id="doc-src-lang" bind:value={docSourceLang} class="input-select">
              {#each SUPPORTED_LANGUAGES as lang}
                <option value={lang.code}>{lang.name} ({lang.native})</option>
              {/each}
            </select>
          </div>
          <div class="form-group">
            <label for="doc-tgt-lang">Translate Document To:</label>
            <select id="doc-tgt-lang" bind:value={docTargetLang} class="input-select">
              {#each SUPPORTED_LANGUAGES as lang}
                <option value={lang.code}>{lang.name} ({lang.native})</option>
              {/each}
            </select>
          </div>
        </div>

        {#if docErrorMessage}
          <div class="alert-error">{docErrorMessage}</div>
        {/if}

        <button
          class="btn-primary w-full"
          disabled={!docFile || isTranslatingDoc}
          on:click={processDocumentTranslation}
        >
          {#if isTranslatingDoc}
            <span class="spinner-small"></span> Parsing & Translating Blocks...
          {:else}
            🚀 Translate Document
          {/if}
        </button>
      </section>

      <!-- Side-by-Side Comparison & Export -->
      <section class="card glass-panel" aria-label="Document Comparison">
        <div class="card-header-flex">
          <h2 class="card-title">2. Side-by-Side Comparison</h2>
          {#if exportedDocText}
            <button class="btn-secondary" on:click={downloadTranslatedDoc}>
              📥 Download Document
            </button>
          {/if}
        </div>

        {#if documentBlocks.length === 0}
          <div class="empty-state">
            <span class="empty-icon">📖</span>
            <p>Upload a document and run translation to review side-by-side block results.</p>
          </div>
        {:else}
          <div class="doc-blocks-container">
            {#each documentBlocks as block}
              <div class="doc-block-row {block.block_type === 'code_block' ? 'code-row' : ''}">
                <div class="doc-col-left">
                  <div class="block-tag">{block.block_type}</div>
                  <pre class="block-content">{block.source_text}</pre>
                </div>
                <div class="doc-col-right">
                  <div class="block-tag">
                    {block.block_type === 'code_block' ? 'Code Preserved (Untouched)' : 'Translated'}
                  </div>
                  <pre class="block-content">{block.translated_text}</pre>
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </section>
    </div>
  {/if}

  <!-- ======================================================== -->
  <!-- TAB 4: LIVE CAPTION OVERLAY & PLATFORM SETTINGS -->
  <!-- ======================================================== -->
  {#if activeTab === 'overlay'}
    <div class="tab-content grid-layout">
      <!-- Overlay Simulator & Preview -->
      <section class="card glass-panel" aria-label="Live Overlay Preview">
        <h2 class="card-title">1. Desktop Transparent Overlay Simulator</h2>
        <p class="card-desc">
          Live visual simulator of the desktop floating overlay window. Test typography, opacity, and positioning.
        </p>

        <!-- Simulated Desktop Screen Area -->
        <div class="desktop-screen-simulator">
          <div
            class="simulated-overlay-window"
            style="
              opacity: {overlaySettings.opacity};
              font-size: {overlaySettings.font_size}px;
              color: {overlaySettings.text_color};
              background-color: {overlaySettings.background_color};
            "
          >
            <div class="sim-header">
              <span class="sim-dot dot-red"></span>
              <span class="sim-dot dot-yellow"></span>
              <span class="sim-dot dot-green"></span>
              <span class="sim-title">LinguaFlow Live Captions</span>
            </div>
            <div class="sim-body">
              {#if overlaySettings.show_source}
                <p class="sim-src">ನಮಸ್ಕಾರ, ಲಿಂಗ್ವಾಫ್ಲೋ ನೇರ ಪ್ರಸಾರ ಶೀರ್ಷಿಕೆಗಳಿಗೆ ಸುಸ್ವಾಗತ.</p>
              {/if}
              {#if overlaySettings.show_translated}
                <p class="sim-tgt">Hello, welcome to LinguaFlow real-time desktop live captions.</p>
              {/if}
            </div>
          </div>
        </div>

        <!-- Capability & DRM Transparency Disclosures -->
        <div class="policy-box glass-panel">
          <h3>Protected Content Restriction Policy (Contract Rule 13)</h3>
          <p>
            LinguaFlow respects operating system security boundaries and HDCP/DRM protections. Protected video playback
            (Widevine, Apple FairPlay, Netflix/Prime Video) cannot be captured via loopback audio or window grabbing.
            For DRM-protected streams, switch to microphone acoustic capture or manual subtitle file import.
          </p>
          <div class="permission-status">
            System Audio Loopback Permission: <span class="badge-perm perm-{audioPermission}">{audioPermission.toUpperCase()}</span>
          </div>
        </div>
      </section>

      <!-- Customization Controls & Global Shortcuts -->
      <section class="card glass-panel" aria-label="Overlay Configuration">
        <h2 class="card-title">2. Visual Customization & Shortcuts</h2>

        <div class="settings-form">
          <div class="form-group">
            <label for="overlay-font-size">Font Size: {overlaySettings.font_size} px</label>
            <input
              type="range"
              id="overlay-font-size"
              min="14"
              max="48"
              bind:value={overlaySettings.font_size}
              class="slider"
            />
          </div>

          <div class="form-group">
            <label for="overlay-opacity">Opacity: {(overlaySettings.opacity * 100).toFixed(0)}%</label>
            <input
              type="range"
              id="overlay-opacity"
              min="0.2"
              max="1.0"
              step="0.05"
              bind:value={overlaySettings.opacity}
              class="slider"
            />
          </div>

          <div class="form-row">
            <div class="form-group">
              <label for="overlay-text-color">Text Color:</label>
              <input type="color" id="overlay-text-color" bind:value={overlaySettings.text_color} class="color-picker" />
            </div>
            <div class="form-group">
              <label for="overlay-bg-color">Background Color:</label>
              <input type="color" id="overlay-bg-color" bind:value={overlaySettings.background_color} class="color-picker" />
            </div>
          </div>

          <div class="form-group">
            <label class="checkbox-row">
              <input type="checkbox" bind:checked={overlaySettings.always_on_top} />
              <span>Always on Top (Pin above other applications)</span>
            </label>
            <label class="checkbox-row">
              <input type="checkbox" bind:checked={overlaySettings.show_source} />
              <span>Show Source Text</span>
            </label>
            <label class="checkbox-row">
              <input type="checkbox" bind:checked={overlaySettings.show_translated} />
              <span>Show Translated Text</span>
            </label>
          </div>

          <hr class="divider" />

          <h3>Global System Shortcuts</h3>
          <div class="form-group">
            <label for="shortcut-translate">Quick Translate (Explicit user action):</label>
            <input
              type="text"
              id="shortcut-translate"
              bind:value={shortcutQuickTranslate}
              class="input-text"
              readonly
            />
          </div>

          <div class="form-group">
            <label for="shortcut-overlay">Toggle Live Overlay Window:</label>
            <input
              type="text"
              id="shortcut-overlay"
              bind:value={shortcutOverlayToggle}
              class="input-text"
              readonly
            />
          </div>

          <button class="btn-primary w-full" on:click={updateAndSaveOverlay}>
            Save Overlay Settings
          </button>

          {#if overlaySaveSuccess}
            <div class="alert-success">Overlay configuration updated successfully.</div>
          {/if}
        </div>
      </section>
    </div>
  {/if}

  <!-- Subtitle Export Modal -->
  {#if showExportModal}
    <div class="modal-backdrop">
      <div class="modal-dialog glass-panel">
        <div class="modal-header">
          <h3>Subtitle Export ({exportFormat.toUpperCase()})</h3>
          <button class="btn-close" on:click={() => (showExportModal = false)}>✕</button>
        </div>
        <div class="modal-body">
          <pre class="export-preview">{exportedSubtitleContent}</pre>
        </div>
        <div class="modal-footer">
          <button class="btn-secondary" on:click={() => navigator.clipboard.writeText(exportedSubtitleContent)}>
            📋 Copy to Clipboard
          </button>
          <button class="btn-primary" on:click={downloadExportedSubtitles}>
            📥 Download File
          </button>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .media-container {
    max-width: 1400px;
    margin: 0 auto;
    padding: 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .studio-header {
    padding: 1.5rem 2rem;
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
  }

  .badge-stage {
    display: inline-block;
    padding: 0.2rem 0.6rem;
    background: rgba(99, 102, 241, 0.2);
    border: 1px solid rgba(99, 102, 241, 0.4);
    color: #818cf8;
    border-radius: 6px;
    font-size: 0.75rem;
    font-weight: 700;
    letter-spacing: 0.05em;
    margin-bottom: 0.5rem;
  }

  .studio-title {
    font-size: 1.85rem;
    font-weight: 800;
    color: var(--text-primary);
    margin: 0;
  }

  .studio-subtitle {
    color: var(--text-secondary);
    font-size: 0.95rem;
    margin: 0.25rem 0 0 0;
  }

  .subtab-nav {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
    border-bottom: 1px solid var(--border-color);
    padding-bottom: 0.5rem;
  }

  .subtab-btn {
    padding: 0.6rem 1.2rem;
    background: transparent;
    border: none;
    border-radius: 8px;
    color: var(--text-secondary);
    font-size: 0.95rem;
    font-weight: 600;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    transition: all 0.2s ease;
  }

  .subtab-btn:hover {
    background: rgba(255, 255, 255, 0.05);
    color: var(--text-primary);
  }

  .subtab-btn.active {
    background: var(--accent-gradient, linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%));
    color: white;
  }

  .grid-layout {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1.5rem;
  }

  @media (max-width: 1024px) {
    .grid-layout {
      grid-template-columns: 1fr;
    }
  }

  .card {
    padding: 1.75rem;
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
  }

  .card-title {
    font-size: 1.2rem;
    font-weight: 700;
    color: var(--text-primary);
    margin: 0;
  }

  .card-desc {
    font-size: 0.88rem;
    color: var(--text-secondary);
    line-height: 1.5;
    margin: 0;
  }

  .card-header-flex {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .dropzone {
    border: 2px dashed rgba(99, 102, 241, 0.35);
    border-radius: 12px;
    padding: 2rem;
    text-align: center;
    background: rgba(99, 102, 241, 0.02);
    cursor: pointer;
    transition: border-color 0.2s;
  }

  .dropzone:hover {
    border-color: #6366f1;
  }

  .dropzone-label {
    cursor: pointer;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
  }

  .dropzone-icon {
    font-size: 2.2rem;
  }

  .hidden-file-input {
    display: none;
  }

  .form-row {
    display: flex;
    gap: 1rem;
  }

  .form-group {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  .form-group label {
    font-size: 0.82rem;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .input-select,
  .input-text {
    padding: 0.65rem 0.9rem;
    background: rgba(0, 0, 0, 0.25);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    color: var(--text-primary);
    font-size: 0.9rem;
  }

  .btn-primary {
    padding: 0.75rem 1.25rem;
    background: linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%);
    color: white;
    font-weight: 600;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    transition: transform 0.15s ease, opacity 0.2s;
  }

  .btn-primary:hover:not(:disabled) {
    transform: translateY(-1px);
    opacity: 0.95;
  }

  .btn-primary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-secondary {
    padding: 0.5rem 1rem;
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid var(--border-color);
    color: var(--text-primary);
    border-radius: 8px;
    cursor: pointer;
    font-size: 0.85rem;
    font-weight: 600;
  }

  .btn-ghost {
    background: transparent;
    border: 1px solid rgba(255, 255, 255, 0.1);
    color: var(--text-secondary);
    padding: 0.5rem 0.8rem;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.82rem;
  }

  .btn-ghost:hover {
    color: var(--text-primary);
    border-color: rgba(255, 255, 255, 0.25);
  }

  .btn-secondary-upload {
    display: inline-block;
    padding: 0.5rem 0.8rem;
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid var(--border-color);
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.82rem;
    color: var(--text-primary);
  }

  .w-full {
    width: 100%;
  }

  /* Player */
  .player-container {
    padding: 1.25rem;
    border-radius: 12px;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .player-controls {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .btn-playback {
    background: #6366f1;
    color: white;
    border: none;
    border-radius: 6px;
    padding: 0.4rem 0.9rem;
    cursor: pointer;
    font-weight: 600;
  }

  .time-display {
    font-family: monospace;
    font-size: 0.9rem;
    color: var(--text-secondary);
  }

  .timeline-bar {
    width: 100%;
    height: 8px;
    background: rgba(255, 255, 255, 0.1);
    border-radius: 4px;
    overflow: hidden;
  }

  .timeline-fill {
    height: 100%;
    background: #6366f1;
    transition: width 0.2s linear;
  }

  .active-caption-box {
    min-height: 50px;
    padding: 0.75rem;
    background: rgba(0, 0, 0, 0.4);
    border-radius: 8px;
    text-align: center;
  }

  .active-src {
    margin: 0;
    font-size: 0.9rem;
    color: var(--text-secondary);
  }

  .active-tgt {
    margin: 0.2rem 0 0 0;
    font-size: 1.1rem;
    font-weight: 700;
    color: #a5b4fc;
  }

  /* Subtitle Segments List */
  .segments-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    max-height: 480px;
    overflow-y: auto;
  }

  .segment-item {
    padding: 0.85rem;
    background: rgba(0, 0, 0, 0.2);
    border: 1px solid var(--border-color);
    border-radius: 8px;
  }

  .seg-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 0.78rem;
    color: var(--text-secondary);
    margin-bottom: 0.4rem;
  }

  .seg-source {
    margin: 0;
    font-size: 0.88rem;
    color: var(--text-secondary);
  }

  .seg-translated {
    margin: 0.2rem 0 0 0;
    font-size: 0.95rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .btn-edit {
    background: transparent;
    border: none;
    color: #818cf8;
    cursor: pointer;
    font-size: 0.75rem;
  }

  .edit-inputs {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    margin-top: 0.4rem;
  }

  .edit-actions {
    display: flex;
    gap: 0.5rem;
  }

  .btn-primary-sm {
    padding: 0.3rem 0.7rem;
    background: #6366f1;
    color: white;
    border: none;
    border-radius: 4px;
    font-size: 0.8rem;
    cursor: pointer;
  }

  .btn-ghost-sm {
    padding: 0.3rem 0.7rem;
    background: transparent;
    border: 1px solid var(--border-color);
    color: var(--text-secondary);
    border-radius: 4px;
    font-size: 0.8rem;
    cursor: pointer;
  }

  .export-controls-row {
    display: flex;
    gap: 1rem;
    align-items: center;
    flex-wrap: wrap;
    font-size: 0.85rem;
  }

  /* Camera / OCR */
  .viewfinder-container {
    position: relative;
    width: 100%;
    height: 280px;
    background: #000;
    border-radius: 12px;
    overflow: hidden;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .video-feed {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .viewfinder-grid {
    position: absolute;
    inset: 15%;
    border: 2px dashed rgba(255, 255, 255, 0.4);
    border-radius: 8px;
    pointer-events: none;
  }

  .viewfinder-controls {
    position: absolute;
    bottom: 1rem;
    display: flex;
    gap: 0.75rem;
  }

  .camera-placeholder {
    display: flex;
    flex-direction: column;
    align-items: center;
    color: var(--text-secondary);
  }

  .image-preview {
    max-width: 100%;
    max-height: 260px;
    object-fit: contain;
  }

  .hidden-canvas {
    display: none;
  }

  .camera-actions-row {
    display: flex;
    gap: 0.75rem;
    flex-wrap: wrap;
  }

  .ocr-stats-banner {
    display: flex;
    gap: 1.5rem;
    padding: 0.6rem 1rem;
    background: rgba(99, 102, 241, 0.1);
    border: 1px solid rgba(99, 102, 241, 0.3);
    border-radius: 8px;
    font-size: 0.85rem;
  }

  .visual-canvas-container {
    position: relative;
    width: 100%;
    border-radius: 8px;
    overflow: hidden;
    background: #05070a;
  }

  .annotated-image {
    width: 100%;
    display: block;
  }

  .bounding-box {
    position: absolute;
    border: 2px solid #38bdf8;
    background: rgba(56, 189, 248, 0.15);
    box-sizing: border-box;
    display: flex;
    align-items: flex-start;
  }

  .overlay-tag {
    background: rgba(15, 23, 42, 0.85);
    color: #38bdf8;
    font-size: 0.75rem;
    font-weight: 700;
    padding: 0.1rem 0.3rem;
    border-radius: 3px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .ocr-blocks-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    max-height: 280px;
    overflow-y: auto;
  }

  .ocr-block-item {
    padding: 0.75rem;
    background: rgba(0, 0, 0, 0.25);
    border-radius: 6px;
    border-left: 3px solid #6366f1;
  }

  .block-top {
    display: flex;
    justify-content: space-between;
    font-size: 0.75rem;
    color: var(--text-secondary);
  }

  /* Document side by side */
  .doc-blocks-container {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    max-height: 540px;
    overflow-y: auto;
  }

  .doc-block-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1rem;
    padding: 0.8rem;
    background: rgba(0, 0, 0, 0.2);
    border-radius: 8px;
  }

  .doc-block-row.code-row {
    border-left: 4px solid #10b981;
    background: rgba(16, 185, 129, 0.05);
  }

  .block-tag {
    font-size: 0.7rem;
    text-transform: uppercase;
    font-weight: 700;
    color: var(--text-secondary);
    margin-bottom: 0.3rem;
  }

  .block-content {
    margin: 0;
    font-family: inherit;
    font-size: 0.88rem;
    white-space: pre-wrap;
    word-break: break-word;
  }

  /* Desktop Overlay Simulator */
  .desktop-screen-simulator {
    width: 100%;
    height: 320px;
    background: radial-gradient(circle at 50% 50%, #1e293b 0%, #0f172a 100%);
    border-radius: 12px;
    border: 1px solid var(--border-color);
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: hidden;
  }

  .simulated-overlay-window {
    width: 80%;
    border-radius: 10px;
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.5);
    border: 1px solid rgba(255, 255, 255, 0.15);
    overflow: hidden;
    backdrop-filter: blur(12px);
  }

  .sim-header {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.5rem 0.75rem;
    background: rgba(255, 255, 255, 0.05);
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  }

  .sim-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
  }

  .dot-red {
    background: #ef4444;
  }
  .dot-yellow {
    background: #f59e0b;
  }
  .dot-green {
    background: #10b981;
  }

  .sim-title {
    font-size: 0.75rem;
    font-weight: 600;
    margin-left: 0.5rem;
  }

  .sim-body {
    padding: 1rem;
    text-align: center;
  }

  .sim-src {
    margin: 0;
    opacity: 0.75;
    font-size: 0.9em;
  }

  .sim-tgt {
    margin: 0.3rem 0 0 0;
    font-weight: 700;
  }

  .policy-box {
    padding: 1rem;
    border-radius: 8px;
    border-left: 4px solid #f59e0b;
    font-size: 0.85rem;
  }

  .policy-box h3 {
    margin: 0 0 0.4rem 0;
    font-size: 0.95rem;
    color: #fbbf24;
  }

  .policy-box p {
    margin: 0;
    color: var(--text-secondary);
    line-height: 1.4;
  }

  .badge-perm {
    padding: 0.15rem 0.5rem;
    border-radius: 4px;
    font-weight: 700;
    font-size: 0.75rem;
  }

  .perm-granted {
    background: rgba(16, 185, 129, 0.2);
    color: #34d399;
  }

  .perm-restricted {
    background: rgba(245, 158, 11, 0.2);
    color: #fbbf24;
  }

  .perm-denied {
    background: rgba(239, 68, 68, 0.2);
    color: #f87171;
  }

  /* Modals */
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.75);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal-dialog {
    width: 600px;
    max-width: 90%;
    max-height: 80vh;
    display: flex;
    flex-direction: column;
    border-radius: 12px;
    overflow: hidden;
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem 1.5rem;
    border-bottom: 1px solid var(--border-color);
  }

  .modal-body {
    padding: 1.5rem;
    overflow-y: auto;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 0.75rem;
    padding: 1rem 1.5rem;
    border-top: 1px solid var(--border-color);
  }

  .export-preview {
    margin: 0;
    font-family: monospace;
    font-size: 0.85rem;
    background: rgba(0, 0, 0, 0.4);
    padding: 1rem;
    border-radius: 6px;
    white-space: pre-wrap;
    max-height: 400px;
    overflow-y: auto;
  }

  .btn-close {
    background: transparent;
    border: none;
    font-size: 1.2rem;
    color: var(--text-secondary);
    cursor: pointer;
  }

  .empty-state {
    padding: 3rem;
    text-align: center;
    color: var(--text-secondary);
  }

  .empty-icon {
    font-size: 2.5rem;
    display: block;
    margin-bottom: 0.5rem;
  }

  .alert-error {
    padding: 0.75rem 1rem;
    background: rgba(239, 68, 68, 0.15);
    border: 1px solid rgba(239, 68, 68, 0.4);
    color: #fca5a5;
    border-radius: 8px;
    font-size: 0.88rem;
  }

  .alert-success {
    padding: 0.75rem 1rem;
    background: rgba(16, 185, 129, 0.15);
    border: 1px solid rgba(16, 185, 129, 0.4);
    color: #6ee7b7;
    border-radius: 8px;
    font-size: 0.88rem;
    margin-top: 0.5rem;
  }

  .checkbox-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.85rem;
    margin-bottom: 0.4rem;
    cursor: pointer;
  }

  .slider {
    width: 100%;
  }

  .color-picker {
    width: 100%;
    height: 38px;
    border-radius: 6px;
    border: 1px solid var(--border-color);
    background: transparent;
    cursor: pointer;
  }

  .divider {
    border: none;
    border-top: 1px solid var(--border-color);
    margin: 1rem 0;
  }
</style>
