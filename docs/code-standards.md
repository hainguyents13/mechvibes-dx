# MechvibesDX — Code Standards & Engineering Guidelines

## 1. Rust Language Standards
- **Rust Edition**: Rust 2024 Edition.
- **Formatting**: Format code according to `rustfmt` standard (`cargo fmt`).
- **Linter Hygiene**: All code must pass `cargo check` and `cargo test` cleanly.
- **Dead Code Annotations**: Use `#[allow(dead_code)]` explicitly on reserved API surfaces or multi-platform abstractions rather than leaving silent warnings.

## 2. Multi-Threading & Thread Safety
- **UI & Audio Isolation**: Audio processing MUST run on a dedicated thread (`src/libs/sound_processor.rs`), completely isolated from the Dioxus GUI thread to avoid UI latency or App Nap throttling.
- **Mutex Lock Scope**: Keep locks on `Arc<Mutex<T>>` short and avoid holding locks across async await points or I/O operations.
- **Channels**: Use `mpsc::channel` or `tokio::sync::mpsc` for inter-thread message passing.

## 3. Platform Scoping (`#[cfg]`)
- Always scope platform-specific APIs clearly:
  - macOS: `#[cfg(target_os = "macos")]`
  - Windows: `#[cfg(target_os = "windows")]`
  - Linux: `#[cfg(target_os = "linux")]`
- Provide non-failing stub implementations for non-supported operating systems.

## 4. Error Handling & Logging
- Use standard `Result<T, String>` or `Result<T, Box<dyn std::error::Error>>` for utility functions.
- Log diagnostic information via `crate::debug_print!` and `env_logger`. Avoid panicking via raw `.unwrap()` in production paths.
