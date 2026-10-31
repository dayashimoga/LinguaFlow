# LinguaFlow — Stage 12: Authentic Media Transcription, Unicode Indic OCR, Document Parsing & Native Overlay

Execute the Global Engineering Contract (`.antigravity/contract.md`). Replace the 0.52 ms Otsu bounding-box stub and 5.0 µs Markdown splitter with real multimedia demuxing, pure-Rust/ONNX Unicode Indic OCR, and multi-format document translation across desktop and mobile.

---

## Phase 12.1: Pure-Rust Media Demuxer, Hardware Resampler & Subtitle Editor (`media_transcription.rs`)

### Subphase 12.1.1 — `symphonia` Bounded 30-Second Sliding-Window Demuxer
- **Minute Details**:
  - Decode `.wav`, `.mp3`, `.ogg`, `.flac`, `.mp4`, and `.m4a` containers in `crates/lingua-engine/src/media_transcription.rs`.
  - Strictly cap the PCM sliding window buffer at 30 seconds (480,000 `f32` samples at 16 kHz = 1.92 MB, well below the $< 3.2$ MB heap budget).
  - Reuse a single pre-allocated buffer across chunks to guarantee zero heap fragmentation on mobile devices.

### Subphase 12.1.2 — Lock-Free Polyphase Audio Resampler (44.1/48 kHz -> 16 kHz Mono)
- **Minute Details**:
  - Implement stereo-to-mono downmixing and polyphase sinc resampling so both media files and mobile/desktop `cpal` microphones running at 44.1 kHz or 48 kHz are cleanly converted to 16 kHz mono `f32` before hitting Silero VAD and streaming ASR.

### Subphase 12.1.3 — Millisecond Subtitle Pipeline & `MediaView.svelte` Timeline Editor
- **Minute Details**:
  - Route VAD-segmented frames through ASR + 3-tier Translation to produce exact timestamps (`00:00:01,240 --> 00:00:04,180`).
  - In `src/views/MediaView.svelte`, build an interactive audio scrubber using `$state.raw` for subtitle cues, inline source/target text editing, and one-click native file export (`.srt`, `.vtt`, `.txt`, `.json`).

---

## Phase 12.2: Cross-Platform Unicode Indic & English OCR Engine (`ocr.rs`)

### Subphase 12.2.1 — Pure-Rust Image Preprocessing & Connected-Component Line Segmentation
- **Minute Details**:
  - In `crates/lingua-engine/src/ocr.rs`, implement EXIF orientation normalization, grayscale conversion, integral-image Sauvola/Otsu adaptive binarization, skew correction, and horizontal projection profile line/word bounding-box detection.

### Subphase 12.2.2 — Memory-Mapped ONNX Trilingual OCR (`kn-IN`, `ta-IN`, `en-US`)
- **Minute Details**:
  - Connect segmented text lines to quantized INT8 ONNX OCR recognition models (`kan`, `tam`, `eng` via `ort` / `tract-onnx`, avoiding `leptess` C++ mobile linker failures).
  - Implement CTC beam-search decoding with Unicode NFC normalization so Kannada *ottaksharas* (ಕ್ಷ, ಜ್ಞ, ಷ್ಟ್ರ) and Tamil *uyirmei* characters assemble without broken viramas/pullis.
  - Load OCR weights via `mmap` and register them with the 90-second idle eviction pool.

### Subphase 12.2.3 — Interactive Region Crop, Clipboard Paste (`Ctrl+V`) & Camera Capture
- **Minute Details**:
  - In `src/views/MediaView.svelte`, build a drag-to-crop bounding box canvas overlay, desktop `Ctrl+V` / `Cmd+V` clipboard screenshot paste, and mobile/desktop camera & gallery image ingestion.

---

## Phase 12.3: Structured Document Parser & Platform-Adaptive Live Caption Overlay

### Subphase 12.3.1 — Layout-Preserving Multi-Format Document Parser (`doc_parser.rs`)
- **Minute Details**:
  - Implement structured AST parsing for `.txt`, `.md` (`pulldown-cmark`), `.csv` (`csv`), `.pdf` (`pdf-extract`), and `.docx` (`docx-rs` / XML zip reader).
  - Preserve headings, bullet hierarchies, and table cell grids across batch translation, and reconstruct translated output in the original format with cancellation token support.

### Subphase 12.3.2 — Desktop Borderless Overlay Window & Mobile PiP/HUD Bridge
- **Minute Details**:
  - Implement `CaptionOverlayManager` in `src-tauri`:
    - On `#[cfg(desktop)]` (Windows, Linux, macOS): Spawn secondary Tauri `WebviewWindow` (`label: "caption-overlay"`, `decorations: false`, `transparent: true`, `alwaysOnTop: true`) with draggable handle, opacity control, and font-size scaler.
    - On `#[cfg(mobile)]` (Android, iOS): Render a Picture-in-Picture (PiP) / persistent floating HUD overlay inside the mobile viewport + native foreground notification so mobile targets never crash attempting desktop multi-window creation.
  - Scope `src-tauri/capabilities/` so `caption-overlay` has minimal read-only event privileges.

---

## Acceptance Criteria
1. Decodes real audio and video files (`.mp3`, `.wav`, `.m4a`, `.mp4`, `.flac`, `.ogg`) via pure-Rust `symphonia` with bounded sliding-window memory ($< 3.2$ MB).
2. Generates valid frame-accurate `.srt` and `.vtt` bilingual subtitle files with interactive timing edits in `MediaView.svelte`.
3. OCR extracts Unicode Indic text from images and clipboard screenshots for Kannada, Tamil, and English with zero C++ `leptess` dependencies.
4. Structured documents (PDF, DOCX, CSV, MD) preserve layout, tables, and markup across batch translation.
5. Desktop caption overlay creates a borderless, transparent, draggable native multi-window overlay with mobile PiP/HUD fallback.
