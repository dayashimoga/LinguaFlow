---
name: svelte5-tauri2-adaptive-ui
description: Enforces Svelte 5 Runes, binary IPC efficiency, keyboard accessibility on Desktop, and touch/safe-area responsiveness on Android & iOS.
---
# Svelte 5 & Tauri 2 Adaptive UI Skill

1. **Svelte 5 Runes Only**:
   - Use `$state`, `$state.raw` (for large read-only arrays like dictionary hits and subtitle cues), `$derived`, `$effect`, and `$props`.
2. **Adaptive Desktop + Mobile Layout**:
   - Desktop (`>= 768px`): Left sidebar with Core Workspace (7 views) + collapsible System Drawer (4 views) and global keyboard shortcuts (`Ctrl+Enter`, `Ctrl+Shift+S`, `Ctrl+K`, `Esc`).
   - Mobile (`< 768px` on Android/iOS): Bottom navigation bar with `padding-bottom: env(safe-area-inset-bottom)`, touch targets `>= 44x44 px`, and swipeable drawer.
3. **Zero-Copy Binary IPC**:
   - Use `Uint8Array` / `ArrayBuffer` for audio waveforms and image crops across Tauri IPC instead of JSON number arrays.

