# LinguaFlow — Stage 4: Real-Time Voice Conversation

Execute the Global Engineering Contract; preserve Stages 0–3.

**Phase 4.1 — Conversation orchestration**
Implement Rust bidirectional conversation sessions with speaker turns, independent source/target language selection, microphone state, translation history, transcript ordering and robust interruption/cancellation behavior.

**Phase 4.2 — Speech synthesis**
Integrate actual platform TTS, compatible offline voices and optional cloud streaming TTS. Support Kannada, Tamil and English only where verified. Implement audio queues, output device management, voice/rate configuration, cancellation and correct language pronunciation.

**Phase 4.3 — Conversation interface**
Build a polished dual-person interface with large bilingual captions, natural speaker indicators, one-tap conversation start, language swap, replay, transcript correction, copy/export and accessible controls. Optimize for one-device face-to-face interactions, then two-device sessions.

**Phase 4.4 — Low-latency interaction**
Implement push-to-talk as the reliable baseline. Add hands-free turn detection with VAD, echo cancellation where supported, silence handling, barge-in and full-duplex operation only after successful acoustic testing. Avoid microphone feedback from translated speaker output.

**Phase 4.5 — Testing and certification**
Test real two-way Kannada↔English, Tamil↔English and Kannada↔Tamil conversations, including rapid speaker changes, interruptions, natural accents, ambient noise, long sessions, network loss and permission changes.

Measure source-audio-to-transcript, transcript-to-translation, translation-to-first-audio, end-to-end conversation latency, intelligibility, dropped/duplicated speech, battery and RAM. Compare against selected existing translators using equivalent test conditions.

Publish independently verified conversation-mode performance and rebuild/upload all supported platform artifacts.

Acceptance: Two users can communicate through actual translated speech with stable sessions, accurate direction handling and clearly reported offline/cloud availability.