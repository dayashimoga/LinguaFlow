# LinguaFlow — Global Engineering Contract

Act as Principal Rust Architect, Real-Time Speech/AI Engineer, Cross-Platform Application Engineer, UI/UX Designer, DevOps/SRE and QA Lead.

Build a production-oriented multilingual dictionary, translator and live communication platform, initially supporting Kannada, Tamil and English, designed for additional languages.

**Technology:** Rust workspace, Tokio, Axum, Tauri 2, Svelte/TypeScript frontend, Rust/WASM where suitable, SQLite/FTS5, pluggable real AI inference engines, Cloudflare for lightweight edge services, Podman for containerized development.

**Mandatory requirements for every stage:**

1. Implement complete working vertical slices. No nonfunctional placeholders, simulated translations, hardcoded responses, fake progress, or claims unsupported by tests. Unit-test doubles are allowed only inside isolated tests.
2. Preserve existing functional behavior. Every change requires regression tests and compatibility checks.
3. Keep security, routing, persistence, streaming, language handling and performance-sensitive operations in Rust. Minimize frontend dependencies and repeated computation.
4. Produce accessible, modern, responsive interfaces for phones, tablets, laptops and desktops; support keyboard/touch, dark/light themes, scalable type and correct Kannada/Tamil Unicode rendering.
5. Use capability detection rather than assuming every language/model/device supports every feature. Unsupported functions must be explained and disabled safely.
6. Implement actual offline/online modes, bounded memory, graceful cancellation, retry/backoff, timeout handling and detailed error reporting.
7. Use verifiable upstream libraries and model versions. Audit commercial model/data licenses; maintain a machine-readable dependency/model inventory and SBOM. Never expose API keys to browser or mobile clients.
8. Maintain requirements, architecture, ADRs, interfaces, data flows, model catalog, benchmarks, code-understanding guide, setup, user guide, troubleshooting, runbook, testing/acceptance reports, SECURITY.md and release documentation.
9. Keep TODO.md and CHANGELOG.md strictly append-only. Preserve all historical entries.
10. Create automated unit, integration, linguistic, E2E, security, accessibility, UI and performance tests. Target >=90% line coverage of testable Rust business logic, 100% passing mandatory gates and real-engine acceptance tests. Report uncovered/ineligible code separately.
11. Use Podman containers for Linux development services and reproducible tooling. Avoid local host installs where practical; native OS release builds must run on suitable GitHub Actions runners.
12. GitHub Actions must run on PR/push/tag, generate Windows, Linux, macOS, Android, iOS and web build outputs, upload independently usable artifacts, generate checksums and provide artifact integrity checks. Separate unsigned build validation from signed distribution.
13. Never conceal blocked tests, missing credentials, platform limitations, unavailable models or performance failures. Report the precise blocker and retain the stage as incomplete if mandatory acceptance criteria cannot be verified.
14. At completion of each stage, provide implemented requirements, commits, exact commands, test results, coverage, benchmark measurements, artifact names/locations, known limitations, regressions prevented and remaining work.

Each stage is independently reviewable and must leave the project buildable and usable.