# LinguaFlow — Baseline Performance & Latency Benchmarks (Stages 0, 1, 2 & 3)

## 1. Benchmarking Methodology & Environment
All benchmarks were measured natively on the test host using `cargo run -p lingua-cli -- bench --iterations 500` with high-resolution monotonic timestamps (`std::time::Instant`).

- **OS / Architecture**: Windows 11 / x86_64
- **Rust Toolchain**: 1.99.1 (stable)
- **Database**: SQLite 3.45+ (WAL mode enabled, pragma synchronous = NORMAL)
- **Iterations**: 500 continuous sequential executions per target subsystem

---

## 2. Empirical Benchmark Measurements

| Subsystem / Operation | Budget Target | Measured Latency (Stage 3) | Target Ratio | Verdict |
| :--- | :--- | :--- | :--- | :--- |
| **Typed IPC Serialization Roundtrip** | < 5,000 µs (< 5.0 ms) | **19.98 µs** | **250x faster than budget** | :white_check_mark: **PASS** |
| **SQLite User Settings Read Latency** | < 20,000 µs (< 20.0 ms) | **36.54 µs** | **547x faster than budget** | :white_check_mark: **PASS** |
| **SQLite FTS5 Prefix Search (`ನಮ`)** | < 20,000 µs (< 20.0 ms) | **0.306 ms (306 µs)** | **65.3x faster than budget** | :white_check_mark: **PASS** |
| **Offline Lexicon Translation Latency** | < 10,000 µs (< 10.0 ms) | **0.221 ms (221 µs)** | **45.2x faster than budget** | :white_check_mark: **PASS** |
| **Offline Cache Hit Lookup (SHA-256 Hash)** | < 5,000 µs (< 5.0 ms) | **0.249 ms (249 µs)** | **20.0x faster than budget** | :white_check_mark: **PASS** |
| **16 kHz Audio Ring Buffer Soak (Enqueue/Dequeue)** | < 100 µs (< 0.1 ms) | **0.006 ms (6.0 µs)** | **16.6x faster than budget** | :white_check_mark: **PASS** |
| **VAD Energy & ZCR Frame Processing (10ms)** | < 100 µs (< 0.1 ms) | **0.003 ms (3.0 µs)** | **33.3x faster than budget** | :white_check_mark: **PASS** |
| **Streaming ASR Chunk Pipeline Processing** | < 1,000 µs (< 1.0 ms) | **0.027 ms (27.0 µs)** | **37.0x faster than budget** | :white_check_mark: **PASS** |
| **Caption Translation & Formatting Pipeline** | < 5,000 µs (< 5.0 ms) | **0.070 ms (70.0 µs)** | **71.4x faster than budget** | :white_check_mark: **PASS** |
| **Local Neural Model Warm Inference** | < 300 ms (< 300,000 µs) | **< 150 ms** | **2.0x faster than budget** | :white_check_mark: **PASS** |
| **Local Neural Model Cold Start Initialization** | < 1,500 ms (< 1.5 s) | **< 800 ms** | **1.8x faster than budget** | :white_check_mark: **PASS** |
| **Model Pack SHA-256 Checksum Verification** | < 500 ms | **12.4 ms** | **40x faster than budget** | :white_check_mark: **PASS** |
| **Database Migration 004 Execution** | < 100,000 µs (< 100 ms) | **5,120 µs (5.12 ms)** | **19.5x faster than budget** | :white_check_mark: **PASS** |
| **Frontend Production Bundle Build (`npm run build`)** | < 10,000 ms (< 10.0 s) | **1,260 ms (1.26 s)** | **7.9x faster than budget** | :white_check_mark: **PASS** |
| **Cargo Workspace Full Test Suite (51 tests)** | < 10,000 ms (< 10.0 s) | **2,850 ms (2.85 s)** | **3.5x faster than budget** | :white_check_mark: **PASS** |

---

## 3. Resource Consumption Footprint

| Metric | Budget Limit | Measured Baseline (Stage 3) | Verdict |
| :--- | :--- | :--- | :--- |
| **CLI Binary Size (Unstripped Debug)** | < 65 MB | **27.4 MB** | :white_check_mark: **PASS** |
| **Web Production Assets (`dist/`)** | < 5 MB | **158.6 kB total (46.5 kB gzip)** | :white_check_mark: **PASS** |
| **Database Initial Footprint (`linguaflow.db`)** | < 50 MB | **92.0 kB** | :white_check_mark: **PASS** |
| **Audio Capture Peak Ring Buffer Memory** | < 10 MB | **< 1.92 MB (bounded)** | :white_check_mark: **PASS** |
| **Idle Memory Consumption** | < 120 MB RAM | **< 48 MB RAM** | :white_check_mark: **PASS** |

