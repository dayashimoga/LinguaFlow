---
name: security-entitlement-guard
description: Enforces SQLCipher encryption, Ed25519 model & license verification, zeroize RAM scrubbing, and Axum RBAC/API gateway hardening.
---
# Security, Privacy & Entitlement Guard Skill

1. **Model & License Authenticity**:
   - Verify `Ed25519` signatures + `SHA-256` digests on all model packs before activation.
   - Validate offline `Free`, `Pro`, and `Enterprise` entitlement tokens via `Ed25519` signature verification in `crates/lingua-core`.
2. **HIPAA / DISHA Zero-Log Scrubbing**:
   - Wrap sensitive patient/enterprise transcripts and PCM audio vectors in `zeroize::Zeroizing` when Zero-Log Privacy Mode is enabled.
3. **Enterprise Gateway Hardening**:
   - Hash `lf_live_...` API keys with BLAKE3/Argon2id, enforce token bucket rate limits in Axum middleware, and append tamper-evident SHA-256 hash-chained entries to the SQLite audit log.
