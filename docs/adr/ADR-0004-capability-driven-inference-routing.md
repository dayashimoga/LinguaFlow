# ADR-0004: Capability-Driven Audio and AI Inference Orchestration

## Status
Accepted

## Context
Multilingual translation and speech processing involve heterogenous execution environments. High-end desktops may have dedicated GPUs; mobile phones may have neural NPUs or be constrained to small quantized CPU models; offline scenarios have zero internet connectivity; and web clients may lack local neural runtime capabilities entirely. Fake functionality and simulated translations are strictly forbidden by the Global Engineering Contract.

## Decision
1. Introduce a core `CapabilityMatrix` and dynamic `EngineRegistry` in Rust.
2. Every engine (Cloud Provider, Local ONNX Runtime, Platform TTS/ASR) implements strongly typed Rust traits (`TranslationEngine`, `SpeechToTextEngine`, `TextToSpeechEngine`).
3. Capability queries return rich state information:
   - Whether the engine is ready, active, uninstalled, unsupported by current hardware, or scheduled for a future stage.
4. During Stage 0:
   - The engine registry reports honest capability status: the baseline foundation is established, persistence and IPC are active, while translation, speech, and conversation AI models are explicitly reported as `ScheduledForStage(1)` through `ScheduledForStage(4)`.
   - UI views reflect this real status transparently, offering interactive inspection of engine requirements, model catalogs, and readiness without any mock or simulated translations.

## Consequences
- Zero fake progress or mock outputs.
- Complete architectural extensibility as real neural engines (IndicTrans, sherpa-onnx, Piper) are brought online in Stages 1–4.
- Automatic runtime capability adaptation to user device constraints.
