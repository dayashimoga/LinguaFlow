# LinguaFlow — License & Dependency Audit (SBOM Framework)

## 1. Compliance Policy & Commercial Safeguards
LinguaFlow strictly audits third-party libraries, linguistic corpora, and neural network weights to ensure permissive licensing, commercial redistributability, and intellectual property compliance.

### License Acceptance Policy
- **Permitted**: Apache-2.0, MIT, BSD-2-Clause, BSD-3-Clause, ISC, CC0-1.0.
- **Permitted with Conditions (Data/Documentation)**: CC-BY-4.0, CC-BY-SA-4.0 (for public linguistic datasets, provided attribution is retained and derivative requirements are met).
- **Restricted/Prohibited in Core Binaries**: AGPL-3.0, GPL-3.0, non-commercial restrictions (CC-BY-NC), or unverified research-only model weights.

---

## 2. Core Upstream Dependencies Audit

### 2.1 Rust Workspace Crates
| Crate | Version | License | Purpose |
| :--- | :--- | :--- | :--- |
| `tokio` | 1.x | MIT | Asynchronous runtime |
| `axum` | 0.8.x | MIT | Gateway web framework |
| `rusqlite` / `sqlx` | 0.32.x | MIT | SQLite persistence & FTS5 |
| `serde`, `serde_json` | 1.0.x | MIT / Apache-2.0 | Serialization |
| `thiserror` | 2.0.x | MIT / Apache-2.0 | Strongly typed errors |
| `tracing`, `tracing-subscriber` | 0.1.x / 0.3.x | MIT | Structured logging & diagnostics |
| `tauri`, `tauri-build` | 2.x | Apache-2.0 / MIT | Cross-platform desktop/mobile framework |

### 2.2 Frontend Libraries
| Package | Version | License | Purpose |
| :--- | :--- | :--- | :--- |
| `svelte` | 5.x | MIT | Reactive UI framework |
| `vite` | 6.x | MIT | Build tool and dev server |
| `typescript` | 5.x | Apache-2.0 | Type checking and interfaces |
| `@tauri-apps/api` | 2.x | Apache-2.0 / MIT | Tauri IPC communication bindings |

---

## 3. Linguistic Datasets & AI Models Catalog

| Component | Target Artifact / Source | License | Attribution Requirement |
| :--- | :--- | :--- | :--- |
| **Kannada Lexicon (Stage 9)** | Alar Dictionary (V. Krishna) / Wiktextract (`kn`) | CC-BY-4.0 / ODC-By | Attribute V. Krishna, Alar.ink & Wiktextract |
| **Tamil Lexicon (Stage 9)** | Univ. of Madras Tamil Lexicon / Open-Tamil / Wiktextract (`ta`) | Public Domain / CC-BY / MIT | Attribute Univ. of Madras & Project Madurai |
| **English Lexical Core (Stage 9)** | WordNet 3.1 / Wiktextract (`en`) | WordNet BSD-style / CC-BY-SA 4.0 | Attribute Princeton University & Wiktionary |
| **Local Translation (Stage 2)** | IndicTrans2 / Small ONNX quantized export | MIT / Permissive | AI4Bharat attribution manifest |
| **Streaming ASR (Stage 3)** | Next-gen Kaldi / sherpa-onnx Indic models | Apache-2.0 | Sherpa ONNX attribution |
| **Neural TTS (Stage 4)** | Piper TTS / Indic-TTS | MIT | Piper / Open Audio attribution |

> Detailed URLs, dataset versions, and cryptographic hashes are documented in [`docs/LEXICON_PROVENANCE.md`](../LEXICON_PROVENANCE.md).

---

## 4. Software Bill of Materials (SBOM) Generation
Every tagged GitHub Actions release workflow invokes:
1. `cargo cyclonedx --all --format json -o target/sbom-cargo.json`
2. `npm sbom --json > target/sbom-npm.json`
The resulting CycloneDX SBOMs are signed and uploaded alongside distribution binaries to ensure complete provenance and verifiable supply chain integrity.
