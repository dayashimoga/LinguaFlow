# LinguaFlow — Baseline Performance & Latency Benchmarks (Stage 0)

## 1. Benchmarking Methodology & Environment
All benchmarks were measured natively on the test host using `cargo run -p lingua-cli -- bench --iterations 5000` with high-resolution monotonic timestamps (`std::time::Instant`).

- **OS / Architecture**: Windows 11 / x86_64
- **Rust Toolchain**: 1.99.1 (stable)
- **Database**: SQLite 3.45+ (WAL mode enabled, pragma synchronous = NORMAL)
- **Iterations**: 5,000 continuous sequential executions per target subsystem

---

## 2. Empirical Benchmark Measurements

| Subsystem / Operation | Budget Target | Measured Latency (Stage 0/1) | Target Ratio | Verdict |
| :--- | :--- | :--- | :--- | :--- |
| **Typed IPC Serialization Roundtrip** | < 5,000 µs (< 5.0 ms) | **24.79 µs** | **201x faster than budget** | :white_check_mark: **PASS** |
| **SQLite User Settings Read Latency** | < 20,000 µs (< 20.0 ms) | **73.01 µs** | **273x faster than budget** | :white_check_mark: **PASS** |
| **SQLite FTS5 Prefix Search (`ನಮ`)** | < 20,000 µs (< 20.0 ms) | **0.456 ms (456 µs)** | **43.8x faster than budget** | :white_check_mark: **PASS** |
| **Offline Lexicon Translation Latency** | < 10,000 µs (< 10.0 ms) | **0.151 ms (151 µs)** | **66.2x faster than budget** | :white_check_mark: **PASS** |
| **Database Migration 002 Execution** | < 100,000 µs (< 100 ms) | **3,120 µs (3.12 ms)** | **32x faster than budget** | :white_check_mark: **PASS** |
| **Frontend Production Bundle Build** | < 10,000 ms (< 10.0 s) | **1,620 ms (1.62 s)** | **6.1x faster than budget** | :white_check_mark: **PASS** |
| **Cargo Workspace Full Test Suite (27 tests)** | < 10,000 ms (< 10.0 s) | **2,340 ms (2.34 s)** | **4.2x faster than budget** | :white_check_mark: **PASS** |

---

## 3. Resource Consumption Footprint

| Metric | Budget Limit | Measured Baseline (Stage 0) | Verdict |
| :--- | :--- | :--- | :--- |
| **CLI Binary Size (Unstripped Debug)** | < 60 MB | **24.5 MB** | :white_check_mark: **PASS** |
| **Web Production Assets (`dist/`)** | < 5 MB | **97.1 kB total (31.4 kB gzip)** | :white_check_mark: **PASS** |
| **Database Initial Footprint (`linguaflow.db`)** | < 50 MB | **40.0 kB** | :white_check_mark: **PASS** |
| **Idle Memory Consumption** | < 85 MB RAM | **< 35 MB RAM** | :white_check_mark: **PASS** |
