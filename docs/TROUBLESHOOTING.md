# LinguaFlow — Troubleshooting & Diagnostics Guide

## 1. Common Issues & Resolutions

### 1.1 Windows: WebView2 Runtime Initialization Error
- **Symptom**: Tauri application launches with a blank white screen or error indicating `Failed to initialize WebView2`.
- **Cause**: WebView2 Runtime is either disabled or not installed on legacy Windows 10 editions.
- **Resolution**:
  - Download and install the [Evergreen WebView2 Bootstrapper](https://developer.microsoft.com/en-us/microsoft-edge/webview2/).
  - Ensure Windows 10 is at least build 1803+.

### 1.2 Linux: Missing WebKitGTK Libraries
- **Symptom**: Compilation fails with `pkg-config: webkit2gtk-4.1 not found`.
- **Cause**: Missing system development headers.
- **Resolution**:
  ```bash
  sudo apt-get install -y libwebkit2gtk-4.1-dev libayatana-appindicator3-dev
  ```

### 1.3 SQLite Database Locked / WAL Recovery
- **Symptom**: `LinguaError::Database("database is locked")`.
- **Cause**: An external process opened the `.db` file in exclusive locking mode, or abnormal termination left a stale lock file.
- **Resolution**:
  - LinguaFlow uses `PRAGMA synchronous = NORMAL` and WAL mode to prevent deadlocks.
  - Ensure no external SQLite browser has opened the database in exclusive write mode.
  - Run `lingua doctor` to automatically verify schema integrity.

### 1.4 Audio Device Permissions (Mobile / Desktop)
- **Symptom**: Microphone button triggers `AudioDeviceError` or remains silent.
- **Resolution**:
  - On macOS: Ensure Terminal / LinguaFlow has Microphone permission in `System Settings -> Privacy & Security -> Microphone`.
  - On iOS: Ensure `NSMicrophoneUsageDescription` is present in `Info.plist`.
  - On Android: Ensure `android.permission.RECORD_AUDIO` is approved in app permissions.

### 1.5 Port 8080 Conflict on Gateway Startup
- **Symptom**: `Failed to bind TCP listener: Only one usage of each socket address is normally permitted`.
- **Resolution**:
  - Specify an alternate port using `--port`:
    ```bash
    lingua serve --port 8085
    ```

### 1.6 Windows Application Displays 'localhost refused to connect' (ERR_CONNECTION_REFUSED)
- **Symptom**: Packaged Windows app `LinguaFlow.exe` opens with a window titled "LinguaFlow — Multilingual Communication" but displays a Chromium error: `localhost refused to connect` / `ERR_CONNECTION_REFUSED`.
- **Cause**: The application was compiled with the `custom-protocol` Cargo feature omitted or with `devUrl` pointing to `localhost:5173`. In Tauri 2, if `custom-protocol` is not enabled, Tauri defines `cfg(dev)` and attempts to navigate WebView2 to the `devUrl` on `localhost:5173` instead of embedding and serving the static bundle from `frontendDist`.
- **Resolution**:
  - `src-tauri/Cargo.toml` includes `default = ["custom-protocol"]` so that all standard `cargo build` and release compilations embed production assets.
  - `src-tauri/tauri.conf.json` omits `devUrl` in production configuration, ensuring WebView2 always loads the bundled offline assets from `frontendDist: "../dist"` via the native `tauri://localhost` scheme.
  - `src-tauri/build.rs` automatically validates that `../dist/index.html` exists prior to `tauri_build::build()`, running `npm run build` if missing.
  - All desktop operations (SQLite persistence, bilingual dictionary search, offline translation, TTS synthesis, model management) operate 100% offline in-process via direct Rust-to-UI IPC without requiring `lingua.exe`, Node.js, or any background HTTP server.
