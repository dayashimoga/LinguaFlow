# LinguaFlow — Stage 6: Media, Captions and Integrations

Execute the Global Engineering Contract and preserve all earlier stages.

**Phase 6.1 — Media translation**
Support importing actual audio/video files, decoding supported formats, speech recognition, timestamped bilingual captions, transcript editing and export to TXT, SRT, VTT and JSON. Use efficient streaming rather than loading entire videos into RAM.

**Phase 6.2 — Desktop live captioning**
Implement permission-based microphone/system-audio capture where supported by each OS. Provide an optional always-on-top, movable, resizable, transparent caption overlay with adjustable languages, fonts, colors and opacity. Document protected-content and application-capture restrictions.

**Phase 6.3 — Camera and OCR**
Integrate real camera/image capture, Unicode-aware OCR, selectable text regions, text translation and original/translated presentation. Support Kannada, Tamil and English with benchmarked OCR accuracy and appropriate image permissions.

**Phase 6.4 — Document handling**
Implement PDF, TXT and supported Office document import/export using approved libraries. Preserve text order, detect language, batch translation efficiently and maintain source/target mapping. Preserve formatting only for document types and transformations actually validated. Add sensible size limits and cancellation.

**Phase 6.5 — Platform integrations**
Add approved desktop global shortcuts, clipboard actions, drag/drop, file associations and share intents where OS APIs permit. No invasive permissions, hidden capture or unsupported system-wide interception.

**Phase 6.6 — End-to-end testing**
Test genuine videos, audio, images and documents; verify timestamps, subtitles, Unicode, language direction, multi-hour memory behavior, cancellation, error recovery and export integrity. Run OS-specific permission and integration tests. Unsupported platform capabilities must have clear explanations and disabled controls.

Rebuild and upload all artifacts. Acceptance: real content can be imported, processed, reviewed and exported correctly, with stable media performance.