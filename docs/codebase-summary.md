# MechvibesDX — Codebase Summary & Directory Map

## 1. Directory Structure Overview
```
src/
├── bin/                       # Standalone binaries
├── components/                # Dioxus UI components
│   ├── pages/                 # Full application page views (Home, Customize, Mood, Settings, Catalog)
│   └── ui/                    # Reusable UI elements (Header, Titlebar, Volume Sliders, Tables)
├── libs/                      # Core backend libraries & services
│   ├── audio/                 # Sound manager, audio context, music/ambiance players
│   ├── focused_input_listener.rs # Keyboard polling and event mapping
│   ├── device_query_mouse_listener.rs # Mouse event polling
│   ├── input_manager.rs       # macOS TCC accessibility checks & channel initializers
│   ├── sound_processor.rs     # Background thread processing loop
│   ├── tray.rs                # System tray icon and native menu handling
│   └── window_manager.rs      # Window state and actions
├── state/                     # Global state & persistent app configurations
│   ├── app.rs                 # Dioxus app state management
│   ├── config.rs              # AppConfig serialization (JSON)
│   ├── music.rs               # Music player state & track models
│   ├── paths.rs               # Path resolution for dev, release, and OS data dirs
│   └── soundpack.rs           # Active soundpack signals
└── utils/                     # Helper modules
    ├── auto_updater.rs        # GitHub release updater
    ├── config_converter.rs    # Legacy Mechvibes v1 to v2 converter
    ├── soundpack_installer.rs # ZIP soundpack unpacker & installer
    └── soundpack_validator.rs # Soundpack structure & manifest validator
```

## 2. Main Execution Flow
1. **`main()` in `src/main.rs`**:
   - Initializes debug logging (`env_logger`).
   - Resolves system paths and creates application directories.
   - Disables macOS App Nap (`disable_app_nap()`).
   - Checks macOS TCC permissions (`AXIsProcessTrusted`).
   - Spawns background input listeners (`device_query` / `rdev` / `evdev`).
   - Initializes global `AudioContext` and starts the background `sound_processor` thread.
   - Builds Dioxus desktop window with transparency and custom protocol configuration.
   - Launches `app_with_stylesheets()`.
