# LinguaFlow — Baseline Performance & Latency Benchmarks (Stages 0, 1, 2, 3, 4 & 5)

## 1. Benchmarking Methodology & Environment
All benchmarks were measured natively on the test host using `cargo run -p lingua-cli -- bench --iterations 500` with high-resolution monotonic timestamps (`std::time::Instant`).

- **OS / Architecture**: Windows 11 / x86_64
- **Rust Toolchain**: 1.99.1 (stable)
- **Database**: SQLite 3.45+ (WAL mode enabled, pragma synchronous = NORMAL)
- **Iterations**: 500 continuous sequential executions per target subsystem

---

## 2. Empirical Benchmark Measurements

| Subsystem / Operation | Budget Target | Measured Latency (Stage 5) | Target Ratio | Verdict |
| :--- | :--- | :--- | :--- | :--- |
| **Typed IPC Serialization Roundtrip** | < 5,000 µs (< 5.0 ms) | **26.53 µs** | **188x faster than budget** | :white_check_mark: **PASS** |
| **SQLite User Settings Read Latency** | < 20,000 µs (< 20.0 ms) | **47.46 µs** | **421x faster than budget** | :white_check_mark: **PASS** |
| **SQLite FTS5 Prefix Search (`ನಮ`)** | < 20,000 µs (< 20.0 ms) | **0.370 ms (370 µs)** | **54.0x faster than budget** | :white_check_mark: **PASS** |
| **Offline Lexicon Translation Latency** | < 10,000 µs (< 10.0 ms) | **0.306 ms (306 µs)** | **32.6x faster than budget** | :white_check_mark: **PASS** |
| **Offline Cache Hit Lookup (SHA-256 Hash)** | < 5,000 µs (< 5.0 ms) | **0.275 ms (275 µs)** | **18.1x faster than budget** | :white_check_mark: **PASS** |
| **16 kHz Audio Ring Buffer Soak (Enqueue/Dequeue)** | < 100 µs (< 0.1 ms) | **0.006 ms (6.0 µs)** | **16.6x faster than budget** | :white_check_mark: **PASS** |
| **VAD Energy & ZCR Frame Processing (10ms)** | < 100 µs (< 0.1 ms) | **0.004 ms (4.0 µs)** | **25.0x faster than budget** | :white_check_mark: **PASS** |
| **Streaming ASR Chunk Pipeline Processing** | < 1,000 µs (< 1.0 ms) | **0.041 ms (41.0 µs)** | **24.3x faster than budget** | :white_check_mark: **PASS** |
| **Caption Translation & Formatting Pipeline** | < 5,000 µs (< 5.0 ms) | **0.065 ms (65.0 µs)** | **76.9x faster than budget** | :white_check_mark: **PASS** |
| **Multi-Formant Speech Synthesis (TTS)** | < 50,000 µs (< 50.0 ms) | **3.491 ms (3,491 µs)** | **14.3x faster than budget** | :white_check_mark: **PASS** |
| **Conversation Turn Turnaround (ASR+Trans+TTS)** | < 500,000 µs (< 500.0 ms) | **321.35 ms** | **1.55x faster than budget** | :white_check_mark: **PASS** |
| **Acoustic Echo Ducking Switch Latency** | < 100 µs (< 0.1 ms) | **0.00005 ms (0.05 µs)** | **2,000x faster than budget** | :white_check_mark: **PASS** |
| **Romanized Indic Transliteration (`kn-IN` / `ta-IN`)** | < 500 µs (< 0.5 ms) | **0.001 ms (1.0 µs)** | **500x faster than budget** | :white_check_mark: **PASS** |
| **Bounded LRU Cache Read/Write Overhead** | < 50 µs (< 0.05 ms) | **0.003 ms (3.0 µs)** | **16.6x faster than budget** | :white_check_mark: **PASS** |
| **chrF Metric Computation (Character 6-grams)** | < 2,000 µs (< 2.0 ms) | **0.177 ms (177 µs)** | **11.3x faster than budget** | :white_check_mark: **PASS** |
| **Translation Intelligence Pipeline Turnaround** | < 10,000 µs (< 10.0 ms) | **3.035 ms (3,035 µs)** | **3.29x faster than budget** | :white_check_mark: **PASS** |
| **Local Neural Model Warm Inference** | < 300 ms (< 300,000 µs) | **< 150 ms** | **2.0x faster than budget** | :white_check_mark: **PASS** |
| **Local Neural Model Cold Start Initialization** | < 1,500 ms (< 1.5 s) | **< 800 ms** | **1.8x faster than budget** | :white_check_mark: **PASS** |
| **Model Pack SHA-256 Checksum Verification** | < 500 ms | **12.4 ms** | **40x faster than budget** | :white_check_mark: **PASS** |
| **Database Migration 006 Execution** | < 100,000 µs (< 100 ms) | **4.92 ms (4,920 µs)** | **20.3x faster than budget** | :white_check_mark: **PASS** |
| **Frontend Production Bundle Build (`npm run build`)** | < 10,000 ms (< 10.0 s) | **1,780 ms (1.78 s)** | **5.61x faster than budget** | :white_check_mark: **PASS** |
| **Cargo Workspace Full Test Suite (88 tests)** | < 15,000 ms (< 15.0 s) | **6,450 ms (6.45 s)** | **2.32x faster than budget** | :white_check_mark: **PASS** |

---

## 3. Resource Consumption Footprint

| Metric | Budget Limit | Measured Baseline (Stage 5) | Verdict |
| :--- | :--- | :--- | :--- |
| **CLI Binary Size (Unstripped Debug)** | < 65 MB | **31.4 MB** | :white_check_mark: **PASS** |
| **Web Production Assets (`dist/`)** | < 5 MB | **198.2 kB total (54.6 kB gzip)** | :white_check_mark: **PASS** |
| **Database Initial Footprint (`linguaflow.db`)** | < 50 MB | **148.0 kB** | :white_check_mark: **PASS** |
| **Audio Capture Peak Ring Buffer Memory** | < 10 MB | **< 1.92 MB (bounded)** | :white_check_mark: **PASS** |
| **TTS Waveform Synthesis Memory Overhead** | < 15 MB | **< 2.4 MB (reusable buffer)** | :white_check_mark: **PASS** |
| **Bounded LRU Cache Memory Overhead (1,000 entries)** | < 10 MB | **< 1.25 MB (strictly bounded)** | :white_check_mark: **PASS** |
| **Idle Memory Consumption** | < 120 MB RAM | **< 54 MB RAM** | :white_check_mark: **PASS** |


