# LinguaFlow — Stage 12: Authentic Media Transcription, Unicode Indic OCR, Document Parsing & Native Overlay

Execute the Global Engineering Contract. Replace the 0.52 ms Otsu bounding-box stub and 5.0 µs Markdown splitter with real multimedia demuxing, Unicode Indic OCR, and multi-format document translation.

---

## Phase 12.1 — Real Media Decoding & Subtitle Generation (`media_transcription.rs`)

### Subphase 12.1.1 — Streaming Audio Demuxer (`symphonia`)
- Integrate pure-Rust `symphonia` in `crates/lingua-engine/src/media_transcription.rs` to decode real `.wav`, `.mp3`, `.ogg`, `.flac`, and `.mp4`/`.m4a` audio streams in bounded 30-second sliding windows ($< 3.2$ MB heap allocation).

### Subphase 12.1.2 — Timestamped Bilingual SRT/VTT/JSON Export & Interactive Subtitle Editor
- Feed decoded PCM chunks through the Stage 11 VAD + ASR + Translation pipeline to produce frame-accurate millisecond timestamps (`00:00:01,240 --> 00:00:04,180`).
- In `MediaView.svelte`, provide an interactive timeline where users can play the media file, inspect/edit both the source transcript and translated subtitle per segment, and export valid `.srt`, `.vtt`, `.txt`, and `.json` files via native Tauri file-save dialogs.

---

## Phase 12.2 — Real Unicode Indic & English OCR Engine (`ocr.rs`)

### Subphase 12.2.1 — Optical Character Recognition for `kn-IN`, `ta-IN`, and `en-US`
- Maintain the Otsu thresholding and connected-component layout analyzer as the preprocessing step, and connect it to a real optical character recognition engine:
  - Embedded ONNX PaddleOCR v4 / Surya-OCR, or Tesseract 5 (`leptess` / `tesseract-rs`) with `kan.traineddata`, `tam.traineddata`, and `eng.traineddata` managed via `ModelsView.svelte`.

### Subphase 12.2.2 — Interactive Image Region Selector & Camera Capture in `MediaView.svelte`
- Allow users to drag-and-drop PNG/JPG/WebP images, paste screenshots from clipboard (`Ctrl+V`), or capture from webcam/mobile camera.
- Render interactive bounding boxes over detected Kannada/Tamil/English text lines, allow region selection, display extracted source Unicode text alongside its translation, and support one-click copy.

---

## Phase 12.3 — Structured Document Parser (`document.rs`) & Desktop Overlay Window

### Subphase 12.3.1 — Multi-Format Document Import/Export (`.txt`, `.md`, `.csv`, `.pdf`, `.docx`)
- Upgrade `crates/lingua-engine/src/document.rs` to extract and reconstruct structured text blocks from PDF (`pdf-extract` / `lopdf`), DOCX (`docx-rs` / XML zip parser), CSV (`csv` crate), and Markdown (`pulldown-cmark`).
- Preserve headings, tables, and paragraph boundaries during batch translation with cancellation token support.

### Subphase 12.3.2 — True Tauri Multi-Window Borderless Live Caption Overlay
- Configure `src-tauri/tauri.conf.json` and `platform_integrations.rs` to spawn an actual secondary native Tauri WebviewWindow (`label: "caption-overlay"`) with `decorations: false`, `transparent: true`, `alwaysOnTop: true`, draggable handle, and customizable font size/opacity for live system/microphone subtitles.

---

## Acceptance Criteria
1. Decodes real audio and video files (`.mp3`, `.wav`, `.m4a`, `.mp4`) via pure-Rust `symphonia` with bounded sliding-window memory ($< 3.2$ MB).
2. Generates valid frame-accurate `.srt` and `.vtt` bilingual subtitle files with interactive timing edits in `MediaView.svelte`.
3. OCR extracts Unicode Indic text from images and clipboard screenshots for Kannada, Tamil, and English.
4. Structured documents (PDF, DOCX, CSV, MD) preserve layout, tables, and markup across batch translation.
5. Desktop caption overlay creates a borderless, transparent, draggable native multi-window overlay.
