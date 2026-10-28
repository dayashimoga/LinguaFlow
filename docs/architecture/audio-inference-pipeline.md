# LinguaFlow — Audio & AI Inference Pipeline Architecture

## 1. High-Level Pipeline Architecture

The real-time speech and translation pipeline operates through an asynchronous, decoupled pipeline designed for bounded memory usage and low audio latency:

```
[ Microphone Input ]
         │ (16 kHz 16-bit PCM)
         ▼
[ Dedicated Audio Thread ] ──(Bounded Ring Buffer: 10MB max)
         │
         ▼
[ Voice Activity Detector (VAD) ] ── Silero / Energy-based VAD
         │ (Speech frames)
         ▼
[ Streaming ASR Engine ] ── sherpa-onnx / Cloud streaming
         │ (Incremental tokens / Partial transcripts)
         ▼
[ Translation Coordinator ] ── Debounced partial & stable final events
         │ (Sequence-stamped translations)
         ▼
[ UI Caption Stream / TTS Queue ] ── Platform TTS / Acoustic Echo Cancellation (AEC)
         │
         ▼
[ Speaker Output ]
```

---

## 2. Key Pipeline Components

### 2.1 Dedicated Audio Thread & Ring Buffer
- **Thread Isolation**: The native audio capture callback executes on a high-priority, real-time operating system thread. It performs zero allocation and locks zero mutexes shared with the UI.
- **Ring Buffer**: Audio frames are written into a lock-free or bounded crossbeam channel/ring buffer. If downstream processing stalls, older frames are systematically dropped to prevent unbound memory growth, and a dropped-frame diagnostic is logged.
- **Standardized Format**: Audio is normalized to 16,000 Hz, 16-bit signed integer linear PCM, single channel (mono).

### 2.2 Voice Activity Detection (VAD)
- Segmenting continuous speech streams into discrete utterance chunks.
- Triggers speech-start within 50ms of vocalization onset.
- Enforces an end-of-speech hangover window (typically 400-600ms of trailing silence) to prevent premature truncation of natural pauses.

### 2.3 Streaming ASR & Incremental Updates
- Emits two classes of transcript events:
  1. `PartialTranscript`: Transient hypotheses displayed immediately in italicized/subtle styling.
  2. `FinalTranscript`: High-confidence, endpointed segment committed to conversation history.
- Each event carries a monotonically increasing `sequence_id` to guarantee strictly ordered display and prevent out-of-order visual jumping.

### 2.4 Translation Coordinator & Cancellation
- When a user continues speaking and alters a partial hypothesis, prior inflight partial translation tasks for that utterance are cancelled via `tokio_util::sync::CancellationToken`.
- Only endpointed sentences or stabilized phrase boundaries trigger heavy translation inference.

### 2.5 Acoustic Echo Cancellation (AEC) & Turn Management
- In Stage 4 conversation mode, device speaker output can bleed into the microphone, creating feedback loops.
- Hardware AEC (provided by OS Audio APIs on iOS/Android/Windows) is engaged whenever full-duplex mode is enabled.
- Push-to-talk (PTT) acts as the fail-safe baseline where the microphone is hardware-muted while TTS audio is rendering.
