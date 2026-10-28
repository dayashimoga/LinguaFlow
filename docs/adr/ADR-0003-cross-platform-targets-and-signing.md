# ADR-0003: Cross-Platform Build Strategy, Code Signing and Distribution

## Status
Accepted

## Context
The Global Engineering Contract mandates buildable targets for Windows, Linux, macOS, Android, iOS, and Web. Furthermore, Requirement 12 states: "Separate unsigned build validation from signed distribution... Never conceal platform limitations... never label unsigned iOS archives as installable IPAs."

## Decision
1. Use **Tauri 2** as the cross-platform shell for desktop (Windows, macOS, Linux) and mobile (Android, iOS).
2. Use **Vite + Svelte 5** to output a static web SPA/PWA bundle deployable to Cloudflare Pages/Workers or static CDNs.
3. Establish a two-tiered build pipeline:
   - **Validation Tier (CI PR & Push)**: Compiles all targets using test or debug signing keys, verifying that source code, resources, and dependencies assemble cleanly across all operating systems.
   - **Distribution Tier (Tags & Releases)**: Applies cryptographic Authenticode (Windows), Apple Developer ID / Notarization (macOS), Android Release Keystore (Google Play), and Apple Distribution Provisioning (iOS) ONLY when legitimate signing secrets are configured in the environment.
4. If signing keys are absent, artifacts are explicitly published and labeled as `unsigned` with documented manual installation or side-loading instructions. Unsigned iOS archives are emitted as `.xcarchive` bundles, never falsely claimed to be signed IPAs.

## Consequences
- Transparent, truthful release reporting without deceptive claims.
- Universal build validation across all 6 targets.
- Clear separation between developer builds, CI validation artifacts, and production-signed distributions.
