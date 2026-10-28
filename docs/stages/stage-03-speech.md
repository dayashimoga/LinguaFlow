# LinguaFlow — Stage 3: Real-Time Speech and Captions

Execute the Global Engineering Contract and retain all prior functionality.

**Phase 3.1 — Native audio capture**
Implement platform audio adapters with explicit permissions, 16 kHz or appropriately resampled PCM, low-latency frame capture, bounded ring buffers, voice activity detection, capture lifecycle management and safe recovery from microphone interruptions. Keep capture off UI threads.

**Phase 3.2 — Streaming ASR**
Integrate an actual streaming recognizer for Kannada, Tamil and English, using supported sherpa-onnx models and/or authorized cloud streaming providers. Define ASR engine traits and confidence-aware partial/final transcript events. Validate each supported language and engine independently.

**Phase 3.3 — Continuous translation**
Connect ASR to the Rust translation coordinator using phrase segmentation, context windows, sequence IDs, incremental partial updates, final-result stabilization and cancellation. Prevent stale responses, duplicate phrases, excessive retranslation and unbounded queues. Display source and translated captions simultaneously.

**Phase 3.4 — Reliable sessions**
Add microphone start/pause/resume/stop, device selection where supported, speech activity indicators, reconnection, timeout handling, adjustable caption size and optional export. Preserve session integrity across suspend/resume and changing network conditions.

**Phase 3.5 — Real performance verification**
Test real prerecorded, consented Kannada/Tamil/English audio and live microphone sessions. Evaluate natural speech, accents, noisy environments, code mixing, long sessions, rapid speech, silence and language switching. Measure speech-onset-to-first-caption, phrase-end-to-final-translation, WER/CER, memory, battery, dropped frames and reconnections.

Provide reproducible real-engine E2E tests and device evidence; cloud tests require real credentials and must never be represented as passed when not executed.

Rebuild/upload all native/web artifacts. Acceptance: continuous speech produces correctly ordered incremental source/translated captions and maintains responsive UI without memory growth or crashes.