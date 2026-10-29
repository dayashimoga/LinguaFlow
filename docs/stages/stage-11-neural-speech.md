# LinguaFlow — Stage 11: Real Audio Capture, Neural Speech Pipeline (ASR & TTS) & Conversation Engine

Execute the Global Engineering Contract. Replace mathematical dummy waveforms (4.8 ms multi-formant buzzers) and stubbed 29 µs ASR pass-throughs with real cross-platform audio I/O and offline neural speech models.

---

## Phase 11.1 — Native Cross-Platform Audio Capture & Playback (`cpal` & `rodio`)

### Subphase 11.1.1 — Non-Blocking Hardware Audio Input (`cpal`)
- Integrate `cpal` in `crates/lingua-engine/src/audio_buffer.rs` to enumerate real OS input microphones and output speakers, populating device selection dropdowns in `SettingsView.svelte` and `LiveSpeechView.svelte`.
- Implement hardware sample-rate negotiation (44.1 kHz / 48 kHz) with high-quality polyphase/sinc downsampling (`rubato`) to 16 kHz 16-bit mono PCM pushed into the lock-free $< 2$ MB ring buffer off the UI thread.

### Subphase 11.1.2 — Silero VAD ONNX + Hybrid Energy/ZCR Gating
- Pair the fast energy/ZCR pre-filter (`vad.rs`) with an embedded Silero VAD v5 ONNX (1.8 MB) model to accurately distinguish human speech from mechanical keyboard clicks, fan noise, and ambient traffic in rural clinics and logistics warehouses.

---

## Phase 11.2 — Real Offline Streaming ASR (`sherpa-onnx` / `Whisper` / `IndicConformer`)

### Subphase 11.2.1 — Streaming Transducer / Whisper Integration (`streaming_asr.rs`)
- Integrate real offline ASR inference using `sherpa-onnx` / `whisper-rs` / ONNX Runtime supporting:
  - **English (`en-US`)**: Zipformer / Whisper-tiny.en / Moonshine-tiny INT8.
  - **Kannada (`kn-IN`) & Tamil (`ta-IN`)**: AI4Bharat IndicWhisper / IndicConformer or MMS-1B-all quantized ONNX adapters.
- Emit genuine `PartialTranscript` events every 250 ms during speech and stabilized `FinalTranscript` events upon VAD silence detection ($\ge 450$ ms), feeding directly into the continuous translation coordinator.

### Subphase 11.2.2 — Honest Capability Gating for Missing ASR Models
- If the user has not downloaded the speech recognition model pack for `kn-IN` or `ta-IN`, `LiveSpeechView.svelte` and `ConversationView.svelte` must display an actionable *Download Speech Pack (kn-IN / ta-IN)* banner linking to `ModelsView.svelte` rather than simulating fake transcripts.

---

## Phase 11.3 — Natural Neural Text-to-Speech (`tts_engine.rs`) & UI Audio Controls

### Subphase 11.3.1 — Replace Formant Buzzer with Neural VITS / Piper / MMS-TTS & Native OS Fallback
- Retire the synthetic 4.8 ms formant sine-wave generator as the primary voice engine.
- Implement a dual-provider TTS architecture in `crates/lingua-engine/src/tts_engine.rs`:
  - **Offline Neural TTS (sherpa-onnx VITS / Piper / Facebook MMS-TTS INT8)**: Natural-sounding local models for `kn-IN`, `ta-IN`, and `en-US` with configurable speaking rate (0.5x to 2.0x).
  - **OS Native Speech Synthesizer Fallback (tts crate / Web Speech API / SAPI / AVSpeechSynthesizer / SpeechDispatcher)**: Zero-download fallback when neural voice packs are not yet installed.

### Subphase 11.3.2 — Wire Audio Controls Across TranslateView, DictionaryView, and ConversationView
- Add Listen / Speak (🔊) buttons with animated playing/stop states and speed controls (0.75x, 1.0x, 1.25x) to:
  - Source Text footer and Translated Text footer in `TranslateView.svelte`.
  - Headword and example sentences in `DictionaryView.svelte`.
  - Every conversation bubble in `ConversationView.svelte`.
- Add a Voice Dictation (🎙️) button inside `TranslateView.svelte`'s Source Text card so users can speak directly into the text translator.

---

## Acceptance Criteria
1. Audio capture enumerates real OS hardware devices and ingests live microphone audio via `cpal` with bounded buffer memory ($< 2$ MB).
2. Silero VAD correctly differentiates spoken voice from ambient silence and noise.
3. Streaming ASR produces authentic partial and final transcripts for supported languages when models are installed.
4. Missing speech models trigger honest download callouts in `ModelsView.svelte` without emitting dummy transcripts.
5. Neural TTS synthesizes intelligible, natural-sounding audio for Kannada, Tamil, and English with OS native synthesizer fallback.

