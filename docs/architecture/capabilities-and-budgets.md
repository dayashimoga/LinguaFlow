# LinguaFlow — Per-Language Capabilities & Performance Budgets

## 1. Per-Language Capability Matrix

LinguaFlow employs strict capability detection. Features are enabled only when verified engine adapters, models, or system services are active for that specific language code.

| Language Code | Language Name | Script | Text Translation | FTS5 Dictionary | Streaming ASR | Neural TTS | OCR Support |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `kn-IN` | Kannada | Kannada (`Knda`) | Stage 1 (Provider/Local) | Stage 1 (Indexed) | Stage 3 (Sherpa/Cloud) | Stage 4 (Platform/Neural) | Stage 6 (Tesseract/Paddle) |
| `ta-IN` | Tamil | Tamil (`Taml`) | Stage 1 (Provider/Local) | Stage 1 (Indexed) | Stage 3 (Sherpa/Cloud) | Stage 4 (Platform/Neural) | Stage 6 (Tesseract/Paddle) |
| `en-US` | English | Latin (`Latn`) | Stage 1 (Provider/Local) | Stage 1 (Indexed) | Stage 3 (Sherpa/Cloud) | Stage 4 (Platform/Neural) | Stage 6 (Tesseract/Paddle) |
| *Expansion* | Hindi (`hi-IN`), Telugu (`te-IN`), Malayalam (`ml-IN`) | Devanagari, Telugu, Malayalam | Capability-detected per engine pack | Capability-detected per engine pack | Capability-detected per engine pack | Capability-detected per engine pack | Capability-detected per engine pack |

### Language Pair Validation Rules
1. Self-translation (`kn-IN` -> `kn-IN`) is prevented and rejected with `LinguaError::IdenticalLanguagePair`.
2. Explicit directionality is required; auto-detection is allowed for source, but target must always be deterministically selected.
3. Transliteration (e.g. Romanized Kannada `namaskara` -> `ನಮಸ್ಕಾರ`) is treated as a specialized pre-translation transformer phase.

---

## 2. Latency Budgets

| Operation | Metric | Target Budget | Upper Bound Threshold | Measurement Condition |
| :--- | :--- | :--- | :--- | :--- |
| **Dictionary Prefix Lookup** | Response time | **< 20 ms** | < 50 ms | SQLite FTS5 on local SSD/eMMC |
| **Translation Debounce** | UI input lag | **250 ms** | 300 ms | Adaptive typing debounce timer |
| **Local Model Warm Translation** | Inference latency | **< 150 ms** | < 350 ms | 15-word sentence on modern CPU |
| **Local Model Cold Start** | Warmup latency | **< 800 ms** | < 1500 ms | Initial model memory map & weight load |
| **ASR Speech-to-First-Caption** | First token latency | **< 300 ms** | < 500 ms | From Voice Activity Detection (VAD) trigger |
| **Push-to-Talk (PTT) Audio Roundtrip** | Turn completion | **< 700 ms** | < 1200 ms | Speech end -> Text -> Translation -> First Audio Frame |
| **IPC Roundtrip (Frontend <-> Rust)** | Command transit | **< 5 ms** | < 15 ms | Tauri binary / JSON IPC channel |

---

## 3. Memory & Resource Budgets

| Resource Domain | Budget Limit | Hard Failure Threshold | Enforcement Mechanism |
| :--- | :--- | :--- | :--- |
| **Idle RAM Footprint (Desktop)** | **< 85 MB** | 150 MB | Cargo profile tuning, strip debug symbols |
| **Idle RAM Footprint (Mobile)** | **< 60 MB** | 100 MB | Android LowMemoryKiller watcher |
| **Active Inference RAM Footprint**| **< 350 MB** | 500 MB | Quantized INT8 / ONNX runtime memory arenas |
| **Audio Capture Ring Buffer** | **5 MB (160s PCM)** | 10 MB | Fixed-capacity ring buffer; oldest overwritten |
| **Local SQLite Database Cache** | **< 50 MB** | 100 MB | LRU eviction policy on history and temporary queries |
| **App Bundle Size (Installer)** | **< 45 MB** | 75 MB | Excludes downloaded model packs; zero silent asset bloat |
| **Model Pack Download Bounding** | **User-Prompted** | Block unapproved | Explicit byte size verification & user confirmation |

---

## 4. Capability Detection Architecture

The capability state is represented as a strongly typed enum in `lingua-core`:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EngineCapabilityState {
    /// Capability is fully verified, operational, and active
    Active,
    /// Capability is ready but currently idle
    Ready,
    /// Model pack or engine not installed; download required
    NotInstalled,
    /// Hardware lacks necessary acceleration or memory
    UnsupportedHardware,
    /// Missing cloud credentials or upstream network unavailable
    UnreachableProvider,
    /// Scheduled for implementation in an upcoming stage (Stage 0 baseline)
    ScheduledForStage(u8),
}
```

Clients and UI never speculate on capability availability; every view queries the Rust core capability detector before rendering actionable buttons or progress indicators.
