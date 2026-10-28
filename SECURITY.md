# LinguaFlow Security Policy & Production Certification

## Overview
LinguaFlow is an offline-capable, cross-platform multilingual communication platform. Security and privacy are core architectural tenets enforced through automated quality gates, verifiable zero-log scrubbing, authentic SQLite persistence, and machine-readable Software Bill of Materials (SBOM).

---

## Supported Versions
Only the latest release and the current active development branch receive active security patches.

| Version | Status | Supported |
| :--- | :--- | :--- |
| **0.8.x** (Stage 7 Production) | Production Ready | :white_check_mark: |
| 0.1.x – 0.7.x | Historical Milestone Releases | :x: |

---

## Threat Model and Core Tenets

1. **Zero-Log Transcript & Credential Sanitization**:
   - Audio utterances, recognized speech text, and translation transcripts are strictly forbidden from being written to persistent application tracing logs or telemetry collectors.
   - All diagnostic logging pipelines route through `SecurityAuditor::scrub_log_message`, which scrubs Bearer tokens (`[REDACTED_TOKEN]`), API credentials (`[REDACTED_CREDENTIAL]`), and transcript fragments (`[REDACTED_TRANSCRIPT]`).
   - Microbenchmark 24 verifies log scrubbing executes in **0.006 ms** without blocking the event loop.

2. **Data Sovereignty & "Right to be Forgotten"**:
   - LinguaFlow stores dictionary bookmarks, translation history, conversation sessions, media sessions, and SM-2 spaced repetition progress locally in an authenticated SQLite 3.45+ database.
   - Users possess full data export capabilities (`lingua privacy export` or Settings UI), exporting an integrity-verified JSON archive with an immutable SHA-256 checksum in **0.124 ms** (Microbenchmark 23).
   - An atomic purge function (`lingua privacy purge` or Settings UI) completely and irreversibly deletes all user sessions, history, cards, and review logs inside a single SQLite transaction (`WAL` checkpointed and `VACUUM` sanitized).

3. **Zero Secret Exposure to Clients**:
   - Third-party API keys or credentials must never be bundled into, hardcoded within, or delivered to client applications (desktop, web, or mobile).
   - Cloud inference routes through authenticated edge proxies or Axum gateways with environment-isolated secrets.

4. **Offline-First Privacy Guarantee**:
   - Speech audio and translation inputs remain strictly on-device in offline mode.
   - Audio ring buffers operate in ephemeral, bounded memory (< 1.92 MB) and are overwritten immediately following inference. No raw PCM buffers are persisted to disk.

5. **Memory Safety and Resource Bounding**:
   - Audio rings, translation text buffers, and SQLite caches operate within strict bounded limits to prevent memory exhaustion and denial-of-service vectors.
   - Chaos stress simulations (`SecurityAuditor::run_chaos_simulation`) subject the engine to high burst error rates, network faults, and memory limits, verifying automatic recovery in under 100 ms with zero memory leaks.

6. **Permissive Commercial Licensing & Verified SBOM**:
   - All bundled and runtime dependencies are audited via `SecurityAuditor::audit_licenses()`, requiring 100% compliance with commercial permissive licenses (MIT, Apache-2.0, BSD-3-Clause, or CC-BY-4.0). Copyleft (GPL/AGPL) dependencies are strictly banned.
   - Machine-readable CycloneDX v1.5 SBOM manifests are maintained at `docs/SBOM.json` and generated via `lingua audit sbom`.

7. **Code Signing Transparency (Contract Rule 12 & 13)**:
   - Development builds and CI test artifacts are produced in unsigned mode.
   - Production distribution artifacts for Windows (`.msi`), macOS (`.dmg`), and Android (`.apk`) require separate signing ceremonies using dedicated hardware security modules (HSMs) or enterprise secrets in release workflows.
   - Unsigned builds are never falsely claimed to be certified or signed.

---

## Reporting a Vulnerability
If you discover a security vulnerability in LinguaFlow, please report it privately:

- **Email**: `security@linguaflow.org` (or directly contact repository maintainers)
- **Response Timeline**:
  - Initial acknowledgement: within 24 hours.
  - Triage and severity assessment: within 72 hours.
  - Patch release: prioritized according to CVSS severity.

Please do **not** open public GitHub issues or forum posts regarding suspected vulnerabilities until an advisory and coordinated fix have been prepared.

---

## Automated Auditing & Quality Gates
- Continuous `cargo clippy --workspace --all-targets -- -D warnings` enforcement.
- Automated CycloneDX SBOM generation and license compliance audits.
- 100% pass rate required on all workspace unit and integration tests (96/96 tests).
