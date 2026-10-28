# LinguaFlow — Operational Runbook (SRE & Operations - Production Stage 7)

## 1. Service Operations

### 1.1 Starting the Edge Gateway Server
To run the native Axum REST & WebSocket gateway daemon:
```bash
lingua serve --host 0.0.0.0 --port 8080
```
Key environment variables:
- `LINGUAFLOW_DATA_DIR`: Explicit storage path for `linguaflow.db`.
- `RUST_LOG`: Logging verbosity filter (e.g., `info`, `warn`).

### 1.2 Health Check Monitoring
The server exposes an automated health check:
```bash
curl -f -s http://localhost:8080/api/v1/health | jq
```
Expected output:
```json
{
  "status": "healthy",
  "path": ".../linguaflow.db",
  "schema_version": 8,
  "settings_persisted": true,
  "error": null
}
```

---

## 2. Security, SBOM & Privacy Auditing

### 2.1 Generating & Auditing the CycloneDX SBOM
To inspect all components, PURLs, and commercial licenses:
```bash
lingua audit sbom
```
Audit output must confirm:
```
[PASS] 100% of dependencies comply with commercial permissive license requirements.
```

### 2.2 Verifying Zero-Log Transcript Sanitization
Verify that application logs strip all credentials and transcripts:
```bash
lingua audit log-check --input 'Error: Bearer sk-secret123 failed for transcript: "ನಮಸ್ಕಾರ"'
```
Expected sanitized output:
```
Error: Bearer [REDACTED_TOKEN] failed for transcript: "[REDACTED_TRANSCRIPT]"
```

### 2.3 Executing Reliability & Chaos Stress Simulation
Run simulated high burst error rate and network flap chaos testing:
```bash
lingua audit chaos
```
Expected verification:
```
[PASS] Chaos stress simulation passed: 0 unhandled panics, memory strictly bounded, system recovered in <100ms.
```

### 2.4 User Data Sovereignty Operations
- **Export User Data Archive**:
  ```bash
  lingua privacy export --output user_archive.json
  ```
  Exports settings, history, decks, and reviews with an immutable SHA-256 integrity hash.
- **Purge All User Data ("Right to be Forgotten")**:
  ```bash
  lingua privacy purge --confirm
  ```
  Executes an atomic transaction resetting tables and vacuuming the SQLite store.

---

## 3. SQLite Database Maintenance & Backup

### 3.1 Hot Backup
Because SQLite operates in WAL (Write-Ahead Logging) mode, backups can be performed live without locking readers:
```bash
sqlite3 "%APPDATA%/LinguaFlow/linguaflow.db" ".backup '%APPDATA%/LinguaFlow/linguaflow_backup.db'"
```

### 3.2 Integrity Verification & Schema 008 Check
Run the comprehensive diagnostic tool:
```bash
lingua doctor
```
Ensures SQLite schema is at version 8, FTS5 indexes are synchronized, audio pipelines are functional, and all Stage 7 tables exist.

---

## 4. Release Packaging & Code Signing Verification

### 4.1 Production Build & Packaging
- Web Assets:
  ```bash
  npm run build
  ```
  Generates production assets in `dist/` (< 280 kB).
- Tauri Desktop Bundle:
  ```bash
  cargo tauri build
  ```

### 4.2 Code Signing Verification (Contract Rule 12 & 13)
- Windows (`.msi` / `.exe`): Verify Authenticode signature using `signtool verify /pa target\release\bundle\msi\*.msi`.
- macOS (`.app` / `.dmg`): Verify code signature with `codesign --verify --deep --strict LinguaFlow.app`.
- **Transparency Policy**: Unsigned development builds are explicitly marked `unsigned` in metadata and must never be distributed as signed releases.
