---
name: rust-crossplatform-resource-guard
description: Enforces ultra-low memory footprint, zero-unwrap safety, mmap ONNX lifecycle, lock-free audio resampling, and 5-platform compilation (Windows, Linux, macOS, Android, iOS).
---
# Rust Cross-Platform & Resource Guard Skill

1. **Zero-Unwrap & Error Taxonomy**:
   - Reject any `.unwrap()` or `.expect()` in `crates/*` and `src-tauri/*`. Map all errors into `lingua_core::error::LinguaError`.
2. **Ultra-Low Memory & `mmap` Model Eviction**:
   - Load ONNX models using memory-mapped file descriptors (`mmap`) rather than reading entire `.onnx` files into `Vec<u8>`.
   - Register every loaded neural session in `ModelLifecyclePool` with `last_used: Instant`. A background tick every `15s` must drop sessions idle for `> 90s` and shrink heap buffers (`shrink_to_fit()`).
3. **Desktop vs. Mobile Compilation Gates**:
   - Always guard `tauri::WebviewWindowBuilder` multi-window creation with `#[cfg(desktop)]`.
   - Always provide a `#[cfg(mobile)]` fallback so `cargo check` passes on `aarch64-linux-android` and `aarch64-apple-ios`.
4. **Verification Gate**:
   - Run `cargo clippy --workspace --all-targets -- -D warnings -D clippy::unwrap_used`
   - Run `cargo test --workspace`

