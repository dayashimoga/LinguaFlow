# LinguaFlow — Stage 0: Foundation

Execute the Global Engineering Contract. Complete all phases and validate before closing the stage.

**Phase 0.1 — Requirements and architecture**
Define product goals, personas, MVP scope, feature matrix, per-language capabilities, privacy model, latency/resource budgets, modular Rust architecture, frontend/backend interfaces and ADRs. Document real-time audio and inference pipelines, deployment strategies, license restrictions and release limitations.

**Phase 0.2 — Repository**
Create/configure the authorized GitHub repository. Initialize Cargo workspace, shared Rust crates, Tauri 2 desktop/mobile host, shared Svelte frontend, web application, Axum gateway and Cloudflare edge configuration. Implement typed IPC/API contracts, dependency injection, versioning and structured errors.

**Phase 0.3 — Working user experience**
Deliver functional Home, Dictionary, Translate, Conversation, Downloads and Settings navigation; persisted theme/language/preferences; accessible layouts; real native window and mobile lifecycle integration. Do not present unavailable AI capabilities as implemented.

**Phase 0.4 — Local development**
Provide idempotent Podman bring-up/bring-down/cleanup scripts, reproducible dependencies, bounded container storage, health checks and sample configurations. Avoid residual services and secrets.

**Phase 0.5 — GitHub Actions**
Configure PR, main and tag pipelines across ubuntu, windows and macOS runners. Compile Rust workspace, web bundle, Windows installer/portable executable, Linux AppImage/DEB, macOS app/DMG, Android APK/AAB and iOS simulator/archive outputs. Publish architecture-specific artifacts, SHA-256 manifests and diagnostics. Produce signed distributable artifacts only when legitimate signing credentials are configured; never label unsigned iOS archives as installable IPAs.

**Phase 0.6 — Quality gate**
Run Rust lint/format/unit/security checks, UI component tests, launch/navigation/persistence E2E smoke tests, Android emulator tests, platform-specific bundle checks and CI artifact inspection. Confirm real app launch on each testable target and document untested physical-device cases.

Deliver a reproducible, buildable, visually polished foundation with passing CI, baseline reports and append-only project history.