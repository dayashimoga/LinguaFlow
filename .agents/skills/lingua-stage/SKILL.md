
---
name: lingua-stage
description: Execute a LinguaFlow implementation stage with engineering contract enforcement, tests and release validation.
---

# LinguaFlow Stage Executor

1. Read AGENTS.md and enforce all engineering requirements.
2. Identify the requested stage number.
3. Read docs/stages/stage-XX-*.md for that stage.
4. Review existing implementation and identify gaps.
5. Implement every phase in dependency order.
6. Run all applicable automated verification.
7. Check for regressions and missing requirements.
8. Build and validate all applicable platform artifacts.
9. Update append-only TODO.md and CHANGELOG.md.
10. Report verified completion, blockers and remaining tasks.

Never claim success for unverified functionality.
Do not automatically proceed to the next stage.
