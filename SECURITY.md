# LinguaFlow Security Policy

## Overview
LinguaFlow is an offline-capable, cross-platform multilingual communication platform. Security and privacy are core architectural tenets.

## Supported Versions
Only the latest release and the current active development branch receive active security patches.

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |
| < 0.1.0 | :x:                |

## Threat Model and Core Tenets
1. **Zero Secret Exposure**: Third-party API keys or credentials must never be bundled into, hardcoded within, or delivered to client applications (desktop, web, or mobile). Cloud inference routes through authenticated edge proxies or Axum gateways.
2. **Offline-First Privacy**: Speech audio and translation inputs remain strictly on-device in offline mode. No audio buffers or transcripts are transmitted externally unless the user explicitly enables an authenticated cloud inference engine.
3. **Memory Safety and Resource Bounding**: Audio rings, translation text buffers, and SQLite caches operate within strict bounded limits to prevent memory exhaustion and denial-of-service vectors.
4. **Local Data Isolation**: SQLite persistence enforces local schema encryption where sensitive session history is stored and isolates user records.
5. **No Fake Security**: Security boundaries are never simulated. Unsigned builds are explicitly distinguished from signed distribution packages.

## Reporting a Vulnerability
If you discover a security vulnerability in LinguaFlow, please report it privately:

- **Email**: `security@linguaflow.org` (or directly contact repository maintainers)
- **Encryption**: Use PGP key if sensitive data or exploit payloads are included.
- **Response Timeline**:
  - Initial acknowledgement: within 24 hours.
  - Triage and severity assessment: within 72 hours.
  - Patch release: prioritized according to CVSS severity.

Please do **not** open public GitHub issues or forum posts regarding suspected vulnerabilities until an advisory and coordinated fix have been prepared.

## Automated Auditing
- Continuous cargo-audit and npm audit checks run on all pull requests.
- SBOM (Software Bill of Materials) manifests are generated for every release artifact.
