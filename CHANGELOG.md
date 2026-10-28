# LinguaFlow — Project Changelog

All notable changes to this project will be documented in this file.
This file is strictly append-only in accordance with the Global Engineering Contract.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased] - 2026-10-28

### Added
- **Global Engineering Contract Baseline**: Initialized project repository adhering to `AGENTS.md`.
- **Requirements & Architecture Specification**: Formulated Product Specification, Personas, Latency/Memory Budgets, and Per-Language Capability Matrix covering Kannada (`kn-IN`), Tamil (`ta-IN`), and English (`en-US`).
- **Requirements Traceability Matrix (RTM)**: Complete mapping across Stage 0 through Stage 8 covering phases, requirements, acceptance criteria, and verification test suites.
- **Architectural Decision Records (ADRs)**:
  - `ADR-0001`: Modular Rust Cargo Workspace and Typed IPC Protocol.
  - `ADR-0002`: Local Persistence Strategy with SQLite and Schema Migrations.
  - `ADR-0003`: Cross-Platform Build Strategy, Code Signing and Distribution.
  - `ADR-0004`: Capability-Driven Audio and AI Inference Orchestration.
- **Modular Rust Workspace**:
  - `lingua-core`: Language models, Unicode normalization, capability matrix, error handling, and typed IPC protocols.
  - `lingua-storage`: SQLite persistence, migrations, user preferences, history, and model pack inventory.
  - `lingua-engine`: Pluggable translation and speech inference traits, runtime capability detection, and routing foundation.
  - `lingua-server`: Axum-based high-throughput REST API and WebSocket gateway for server/edge deployments.
  - `lingua-cli`: Headless CLI tool for operations, migrations, diagnostics, and headless benchmarks.
  - `src-tauri`: Tauri 2 desktop and mobile host integration.
- **Accessible Svelte 5 / TypeScript Frontend**:
  - Unified, responsive UI for Desktop, Tablet, and Mobile viewport sizes.
  - First-class Kannada and Tamil typography and Unicode rendering support.
  - Views: Home, Dictionary, Translate, Conversation, Downloads (Model Packs), and Settings.
  - Live persistence of theme (Light/Dark/System), active language pair, and audio parameters via SQLite IPC.
  - Honest capability detection display (no fake translations or mock completions).
- **Containerized Development (Podman)**:
  - Idempotent lifecycle management scripts (`scripts/podman-up.*`, `scripts/podman-down.*`, `scripts/podman-clean.*`).
  - Containerfile and Podman Compose configuration with healthchecks and bounded volume persistence.
- **Continuous Integration & Delivery Pipelines**:
  - GitHub Actions CI matrix for Ubuntu, Windows, and macOS (`.github/workflows/ci.yml`).
  - Mobile compilation and verification pipeline for Android APK/AAB and iOS (`.github/workflows/mobile.yml`).
  - Release publication pipeline with automated SHA-256 manifest verification (`.github/workflows/release.yml`).
- **Quality Gates & Comprehensive Documentation**:
  - Rust unit, integration, and property test suites.
  - Security and Privacy Policy (`SECURITY.md`).
  - Setup Guide (`docs/SETUP.md`), Runbook (`docs/RUNBOOK.md`), User Guide (`docs/USER_GUIDE.md`), Troubleshooting Guide (`docs/TROUBLESHOOTING.md`), and Benchmarks Baseline (`docs/BENCHMARKS.md`).
