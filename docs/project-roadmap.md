# MechvibesDX — Project Roadmap & Future Enhancements

## Phase 1: Stability & OS Fixes (Completed)
- [x] Continuous `device_query` key polling for background capture on macOS Sonoma/Sequoia.
- [x] App Nap disabler implementation via Objective-C runtime FFI.
- [x] Dynamic macOS TCC Accessibility permission detection (`AXIsProcessTrusted`) and UI banner.
- [x] Dead code cleanup and 100% warning elimination across compiler targets.
- [x] Core unit test suite implementation for validators and path resolvers.

## Phase 2: User Experience & Catalog Expansion (Current)
- [ ] Expanded remote soundpack catalog with tag filtering (Tactile, Clicky, Linear, Silence, Custom).
- [ ] User custom key mapping editor within the Dioxus GUI.
- [ ] Per-key sound assignment overrides.

## Phase 3: Platform & Community (Future)
- [ ] Full Linux Wayland native portal integration.
- [ ] Cloud sync for customized themes and soundpack configurations.
- [ ] Custom soundpack creation wizard with automatic audio slicing and pitch shifting.
