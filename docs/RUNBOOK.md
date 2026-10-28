# LinguaFlow — Operational Runbook (SRE & Operations)

## 1. Service Operations

### 1.1 Starting the Edge Gateway Server
To run the native Axum REST & WebSocket gateway daemon:
```bash
lingua serve --host 0.0.0.0 --port 8080
```
Optional environment variables:
- `LINGUAFLOW_DATA_DIR`: Explicit storage path for `linguaflow.db`.
- `RUST_LOG`: Logging verbosity filter (e.g., `info`, `debug`, `trace`).

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
  "schema_version": 1,
  "settings_persisted": true,
  "error": null
}
```

---

## 2. Podman Container Management

### 2.1 Starting Services
- Windows: `.\scripts\podman-up.ps1`
- Linux/macOS: `./scripts/podman-up.sh`

### 2.2 Inspecting Logs & Container Health
```bash
podman ps -a --filter name=linguaflow-gateway
podman logs --tail 100 -f linguaflow-gateway
```

### 2.3 Stopping & Cleaning Services
- Bring down cleanly:
  - Windows: `.\scripts\podman-down.ps1`
  - Linux/macOS: `./scripts/podman-down.sh`
- Purge all containers, temporary volumes, and images:
  - Windows: `.\scripts\podman-clean.ps1`
  - Linux/macOS: `./scripts/podman-clean.sh`

---

## 3. SQLite Database Maintenance & Backup

### 3.1 Hot Backup
Because SQLite is running in WAL (Write-Ahead Logging) mode, backups can be taken without locking active reads:
```bash
sqlite3 "%APPDATA%/LinguaFlow/linguaflow.db" ".backup '%APPDATA%/LinguaFlow/linguaflow_backup.db'"
```

### 3.2 Integrity Verification
Run the built-in doctor command:
```bash
lingua doctor
```
Or via sqlite CLI:
```bash
sqlite3 linguaflow.db "PRAGMA integrity_check;"
```
Expected response: `ok`.
