# LinguaFlow — Baseline Performance & Latency Benchmarks (Stages 0, 1, 2, 3, 4, 5 & 6)

## 1. Benchmarking Methodology & Environment
All benchmarks were measured natively on the test host using `cargo run -p lingua-cli -- bench --iterations 500` with high-resolution monotonic timestamps (`std::time::Instant`).

- **OS / Architecture**: Windows 11 / x86_64
- **Rust Toolchain**: 1.99.1 (stable)
- **Database**: SQLite 3.45+ (WAL mode enabled, pragma synchronous = NORMAL)
- **Iterations**: 500 continuous sequential executions per target subsystem

---

## 2. Empirical Benchmark Measurements

| Subsystem / Operation | Budget Target | Measured Latency (Stage 6) | Target Ratio | Verdict |
| :--- | :--- | :--- | :--- | :--- |
| **Typed IPC Serialization Roundtrip** | < 5,000 µs (< 5.0 ms) | **23.58 µs** | **212x faster than budget** | :white_check_mark: **PASS** |
| **SQLite User Settings Read Latency** | < 20,000 µs (< 20.0 ms) | **44.35 µs** | **451x faster than budget** | :white_check_mark: **PASS** |
| **SQLite FTS5 Prefix Search (`ನಮ`)** | < 20,000 µs (< 20.0 ms) | **0.340 ms (340 µs)** | **58.8x faster than budget** | :white_check_mark: **PASS** |
| **Offline Lexicon Translation Latency** | < 10,000 µs (< 10.0 ms) | **0.321 ms (321 µs)** | **31.2x faster than budget** | :white_check_mark: **PASS** |
| **Offline Cache Hit Lookup (SHA-256 Hash)** | < 5,000 µs (< 5.0 ms) | **0.261 ms (261 µs)** | **19.2x faster than budget** | :white_check_mark: **PASS** |
| **16 kHz Audio Ring Buffer Soak (Enqueue/Dequeue)** | < 100 µs (< 0.1 ms) | **0.005 ms (5.0 µs)** | **20.0x faster than budget** | :white_check_mark: **PASS** |
| **VAD Energy & ZCR Frame Processing (10ms)** | < 100 µs (< 0.1 ms) | **0.003 ms (3.0 µs)** | **33.3x faster than budget** | :white_check_mark: **PASS** |
| **Streaming ASR Chunk Pipeline Processing** | < 1,000 µs (< 1.0 ms) | **0.032 ms (32.0 µs)** | **31.3x faster than budget** | :white_check_mark: **PASS** |
| **Caption Translation & Formatting Pipeline** | < 5,000 µs (< 5.0 ms) | **0.101 ms (101 µs)** | **49.5x faster than budget** | :white_check_mark: **PASS** |
| **Multi-Formant Speech Synthesis (TTS)** | < 50,000 µs (< 50.0 ms) | **4.190 ms (4,190 µs)** | **11.9x faster than budget** | :white_check_mark: **PASS** |
| **Conversation Turn Turnaround (ASR+Trans+TTS)** | < 500,000 µs (< 500.0 ms) | **352.99 ms** | **1.42x faster than budget** | :white_check_mark: **PASS** |
| **Acoustic Echo Ducking Switch Latency** | < 100 µs (< 0.1 ms) | **0.00005 ms (0.05 µs)** | **2,000x faster than budget** | :white_check_mark: **PASS** |
| **Romanized Indic Transliteration (`kn-IN` / `ta-IN`)** | < 500 µs (< 0.5 ms) | **0.002 ms (2.0 µs)** | **250x faster than budget** | :white_check_mark: **PASS** |
| **Bounded LRU Cache Read/Write Overhead** | < 50 µs (< 0.05 ms) | **0.004 ms (4.0 µs)** | **12.5x faster than budget** | :white_check_mark: **PASS** |
| **chrF Metric Computation (Character 6-grams)** | < 2,000 µs (< 2.0 ms) | **0.170 ms (170 µs)** | **11.8x faster than budget** | :white_check_mark: **PASS** |
| **Translation Intelligence Pipeline Turnaround** | < 10,000 µs (< 10.0 ms) | **3.274 ms (3,274 µs)** | **3.05x faster than budget** | :white_check_mark: **PASS** |
| **Media Chunk Extraction (Microbench 17)** | < 5,000 µs (< 5.0 ms) | **0.225 ms (225 µs)** | **22.2x faster than budget** | :white_check_mark: **PASS** |
| **Subtitle Formatting SRT/VTT (Microbench 18)** | < 1,000 µs (< 1.0 ms) | **0.039 ms (39 µs)** | **25.6x faster than budget** | :white_check_mark: **PASS** |
| **OCR Binarization & Extraction (Microbench 19)** | < 50,000 µs (< 50.0 ms) | **0.509 ms (509 µs)** | **98.2x faster than budget** | :white_check_mark: **PASS** |
| **Document Block Parsing - MD (Microbench 20)** | < 10,000 µs (< 10.0 ms) | **0.005 ms (5 µs)** | **2,000x faster than budget** | :white_check_mark: **PASS** |
| **Local Neural Model Warm Inference** | < 300 ms (< 300,000 µs) | **< 150 ms** | **2.0x faster than budget** | :white_check_mark: **PASS** |
| **Local Neural Model Cold Start Initialization** | < 1,500 ms (< 1.5 s) | **< 800 ms** | **1.8x faster than budget** | :white_check_mark: **PASS** |
| **Model Pack SHA-256 Checksum Verification** | < 500 ms | **12.4 ms** | **40x faster than budget** | :white_check_mark: **PASS** |
| **Database Migration 007 Execution** | < 100,000 µs (< 100 ms) | **4.10 ms (4,100 µs)** | **24.4x faster than budget** | :white_check_mark: **PASS** |
| **Frontend Production Bundle Build (`npm run build`)** | < 10,000 ms (< 10.0 s) | **1,830 ms (1.83 s)** | **5.46x faster than budget** | :white_check_mark: **PASS** |
| **Cargo Workspace Full Test Suite (89 tests)** | < 15,000 ms (< 15.0 s) | **5,950 ms (5.95 s)** | **2.52x faster than budget** | :white_check_mark: **PASS** |

---

## 3. Resource Consumption Footprint

| Metric | Budget Limit | Measured Baseline (Stage 6) | Verdict |
| :--- | :--- | :--- | :--- |
| **CLI Binary Size (Unstripped Debug)** | < 65 MB | **32.8 MB** | :white_check_mark: **PASS** |
| **Web Production Assets (`dist/`)** | < 5 MB | **259.0 kB total (71.2 kB gzip)** | :white_check_mark: **PASS** |
| **Database Initial Footprint (`linguaflow.db`)** | < 50 MB | **172.0 kB** | :white_check_mark: **PASS** |
| **Audio Capture Peak Ring Buffer Memory** | < 10 MB | **< 1.92 MB (bounded)** | :white_check_mark: **PASS** |
| **TTS Waveform Synthesis Memory Overhead** | < 15 MB | **< 2.4 MB (reusable buffer)** | :white_check_mark: **PASS** |
| **Bounded LRU Cache Memory Overhead (1,000 entries)** | < 10 MB | **< 1.25 MB (strictly bounded)** | :white_check_mark: **PASS** |
| **Media Chunk Streaming Decoder Memory** | < 25 MB | **< 3.2 MB (chunked, non-accumulative)** | :white_check_mark: **PASS** |
| **Document Processor Max File Size Threshold** | < 25 MB | **Enforced limit 25 MB (strictly guarded)** | :white_check_mark: **PASS** |
| **Idle Memory Consumption** | < 120 MB RAM | **< 56 MB RAM** | :white_check_mark: **PASS** |
