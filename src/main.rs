#![windows_subsystem = "windows"] // Re-enabled after debugging
#![allow(non_snake_case)]

mod components;
mod libs;
mod state;
mod utils;

use dioxus::desktop::{ Config, LogicalPosition, LogicalSize, WindowBuilder };
use dioxus::prelude::*;
use libs::protocol;
use utils::constants::{ APP_NAME, APP_PROTOCOL_URL };
use libs::ui;
use libs::window_manager::{ WindowAction, WINDOW_MANAGER };
use libs::input_listener::start_unified_input_listener;
use libs::input_manager::init_input_channels;
use std::sync::mpsc;

// Function to conditionally set windows subsystem based on config
fn should_show_console() -> bool {
    // Try to load config to check debug console setting
    match std::panic::catch_unwind(|| { state::config::AppConfig::load().show_debug_console }) {
        Ok(show_debug) => show_debug,
        Err(_) => false, // Default to false if config loading fails
    }
}

// Use .ico format for better Windows compatibility
const EMBEDDED_ICON: &[u8] = include_bytes!("../assets/icon.ico");

fn load_icon() -> Option<dioxus::desktop::tao::window::Icon> {
    // Try to create icon from embedded ICO data
    match image::load_from_memory_with_format(EMBEDDED_ICON, image::ImageFormat::Ico) {
        Ok(img) => {
            let rgba = img.to_rgba8();
            let (width, height) = rgba.dimensions();
            match dioxus::desktop::tao::window::Icon::from_rgba(rgba.into_raw(), width, height) {
                Ok(icon) => {
                    debug_print!("✅ Loaded embedded ICO icon ({}x{})", width, height);
                    Some(icon)
                }
                Err(e) => {
                    always_eprint!("❌ Failed to create icon from embedded ICO data: {}", e);
                    None
                }
            }
        }
        Err(e) => {
            debug_eprint!("❌ Failed to load embedded ICO data: {}", e);
            None
        }
    }
}

fn main() {
    // Initialize debug logging first
    utils::logger::init_debug_logging();

    // Hide console window if debug console is disabled in config
    if !should_show_console() {
        #[cfg(windows)]
        {
            unsafe {
                let console = winapi::um::wincon::GetConsoleWindow();
                if !console.is_null() {
                    winapi::um::winuser::ShowWindow(console, winapi::um::winuser::SW_HIDE);
                }
            }
        }
    }
    env_logger::init();

    debug_print!("🚀 Initializing {}...", APP_NAME);

    // Initialize app manifest first
    let _manifest = state::manifest::AppManifest::load(); // Check for command line arguments (protocol handling and startup options)
    let args: Vec<String> = std::env::args().collect();
    debug_print!("🔍 Command line args: {:?}", args); // Check if we should start minimized (from auto-startup)
    let should_start_minimized =
        args.contains(&"--minimized".to_string()) ||
        (state::config::AppConfig::load().auto_start &&
            state::config::AppConfig::load().start_minimized);

    if should_start_minimized {
        debug_print!("🔽 Starting minimized to system tray");
    }

    if args.len() > 1 {
        // Handle protocol URL if passed as argument
        for arg in &args[1..] {
            if arg == "--minimized" {
                debug_print!("🔽 Starting minimized to tray");
                continue;
            }
            if arg == "--debug-conversion" {
                debug_print!("🧪 Running conversion debug test");
                debug_conversion_test();
                return;
            }

            if arg == "--test-import-conversion" {
                debug_print!("🧪 Running import conversion test");
                test_import_conversion();
                return;
            }

            if arg.starts_with(APP_PROTOCOL_URL) {
                debug_print!("✅ Detected protocol URL: {}", arg);
                if let Err(e) = protocol::handle_protocol_url(arg) {
                    always_eprint!("❌ Failed to handle protocol URL {}: {}", arg, e);
                } else {
                    debug_print!("✅ Protocol URL handled successfully");
                }
                return; // Exit after handling protocol
            }
        }
    } else {
        debug_print!("ℹ️ No command line arguments provided");
    }

    // Register protocol on first run
    // if let Err(e) = protocol::register_protocol() {
    //     eprintln!("Warning: Failed to register mechvibes:// protocol: {}", e);
    // }    // Initialize global app state before rendering
    state::app::init_app_state();

    // Create input event channels for communication between input listener and UI
    let (keyboard_tx, keyboard_rx) = mpsc::channel::<String>();
    let (mouse_tx, mouse_rx) = mpsc::channel::<String>();
    let (hotkey_tx, hotkey_rx) = mpsc::channel::<String>();

    // Initialize global input channels for UI to access
    init_input_channels(keyboard_rx, mouse_rx, hotkey_rx);

    // Start the unified input listener early in main
    debug_print!("🎮 Starting unified input listener from main...");
    start_unified_input_listener(keyboard_tx, mouse_tx, hotkey_tx);

    // Create window action channel
    let (window_tx, _window_rx) = mpsc::channel::<WindowAction>();
    WINDOW_MANAGER.set_action_sender(window_tx); // Create a WindowBuilder with custom appearance
    let window_builder = WindowBuilder::default()
        .with_title(APP_NAME)
        .with_transparent(false) // Disable transparency for better performance
        .with_always_on_top(false) // Allow normal window behavior for taskbar
        .with_position(LogicalPosition::new(1700.0, 300.0))
        .with_inner_size(LogicalSize::new(500.0, 850.0))
        .with_fullscreen(None)
        .with_decorations(false) // Use custom title bar
        .with_resizable(false) // Disable window resizing
        .with_visible(!should_start_minimized) // Hide window if starting minimized
        .with_window_icon(load_icon()); // Set window icon for taskbar

    // Create config with our window settings and custom protocol handlers
    let config = Config::new().with_window(window_builder).with_menu(None);

    // Launch the app with our config
    dioxus::LaunchBuilder::desktop().with_cfg(config).launch(app_with_stylesheets)
}

fn app_with_stylesheets() -> Element {
    rsx! {
        ui::app {}
    }
}

fn debug_conversion_test() {
    use std::io::Write;
    let mut log_file = std::fs::File
        ::create("conversion_debug.log")
        .expect("Failed to create log file");

    writeln!(
        log_file,
        "🧪 Testing V1 to V2 conversion for custom-sound-pack-1660581102261"
    ).unwrap();

    let soundpack_id = "custom-sound-pack-1660581102261";
    let config_path = format!("soundpacks/{}/config.json", soundpack_id);

    // Check if config exists
    if !std::path::Path::new(&config_path).exists() {
        writeln!(log_file, "❌ Config file not found: {}", config_path).unwrap();
        return;
    }

    writeln!(log_file, "📁 Found config file: {}", config_path).unwrap();

    // Read current config content
    match std::fs::read_to_string(&config_path) {
        Ok(config_content) => {
            writeln!(log_file, "📄 Current config preview:").unwrap();
            writeln!(
                log_file,
                "{}",
                &config_content[..std::cmp::min(300, config_content.len())]
            ).unwrap();
            if config_content.len() > 300 {
                writeln!(log_file, "... (truncated)").unwrap();
            }
        }
        Err(e) => {
            writeln!(log_file, "❌ Failed to read config: {}", e).unwrap();
            return;
        }
    }

    // Read and validate current config
    let validation_result = utils::soundpack_validator::validate_soundpack_config(&config_path);

    writeln!(log_file, "🔍 Validation result:").unwrap();
    writeln!(log_file, "   Status: {:?}", validation_result.status).unwrap();
    writeln!(log_file, "   Config version: {:?}", validation_result.config_version).unwrap();
    writeln!(log_file, "   Is valid V2: {}", validation_result.is_valid_v2).unwrap();
    writeln!(log_file, "   Can be converted: {}", validation_result.can_be_converted).unwrap();

    // If it's V1 and can be converted, perform conversion
    if
        validation_result.status ==
            utils::soundpack_validator::SoundpackValidationStatus::VersionOneNeedsConversion &&
        validation_result.can_be_converted
    {
        writeln!(log_file, "🔄 Starting V1 to V2 conversion...").unwrap();

        // Create backup
        let backup_path = format!("{}.debug.backup", config_path);
        match std::fs::copy(&config_path, &backup_path) {
            Ok(_) => writeln!(log_file, "💾 Created backup: {}", backup_path).unwrap(),
            Err(e) => {
                writeln!(log_file, "❌ Failed to create backup: {}", e).unwrap();
                return;
            }
        }

        // Perform conversion
        match utils::config_converter::convert_v1_to_v2(&config_path, &config_path) {
            Ok(()) => {
                writeln!(log_file, "✅ Conversion completed successfully!").unwrap();

                // Read converted config
                match std::fs::read_to_string(&config_path) {
                    Ok(converted_content) => {
                        writeln!(log_file, "📄 Converted config preview:").unwrap();
                        writeln!(
                            log_file,
                            "{}",
                            &converted_content[..std::cmp::min(500, converted_content.len())]
                        ).unwrap();
                        if converted_content.len() > 500 {
                            writeln!(log_file, "... (truncated)").unwrap();
                        }
                        // Look for timing values in the converted config
                        if converted_content.contains("[0.0, 0.0]") {
                            writeln!(
                                log_file,
                                "⚠️  Found [0.0, 0.0] timing values in converted config!"
                            ).unwrap();
                            // Count how many [0.0, 0.0] entries there are
                            let count = converted_content.matches("[0.0, 0.0]").count();
                            writeln!(log_file, "   Total [0.0, 0.0] entries: {}", count).unwrap();
                        }
                        if converted_content.contains("[0.0, 100.0]") {
                            writeln!(
                                log_file,
                                "✅ Found some valid [0.0, 100.0] timing values"
                            ).unwrap();
                            let count = converted_content.matches("[0.0, 100.0]").count();
                            writeln!(log_file, "   Total [0.0, 100.0] entries: {}", count).unwrap();
                        }
                        if converted_content.contains("\"timing\"") {
                            writeln!(log_file, "🔍 Found timing entries in config").unwrap();
                        }
                    }
                    Err(e) =>
                        writeln!(log_file, "❌ Failed to read converted config: {}", e).unwrap(),
                }

                // Re-validate the converted config
                let post_validation = utils::soundpack_validator::validate_soundpack_config(
                    &config_path
                );
                writeln!(log_file, "🔍 Post-conversion validation:").unwrap();
                writeln!(log_file, "   Status: {:?}", post_validation.status).unwrap();
                writeln!(
                    log_file,
                    "   Config version: {:?}",
                    post_validation.config_version
                ).unwrap();
                writeln!(log_file, "   Is valid V2: {}", post_validation.is_valid_v2).unwrap();

                // Try to load metadata
                match utils::soundpack::load_soundpack_metadata(soundpack_id) {
                    Ok(metadata) => {
                        writeln!(log_file, "📊 Loaded metadata successfully:").unwrap();
                        writeln!(log_file, "   Name: {}", metadata.name).unwrap();
                        writeln!(log_file, "   Author: {:?}", metadata.author).unwrap();
                        writeln!(log_file, "   Version: {}", metadata.version).unwrap();
                        writeln!(log_file, "   Mouse: {}", metadata.mouse).unwrap();
                        writeln!(log_file, "   Icon: {:?}", metadata.icon).unwrap();
                        writeln!(
                            log_file,
                            "   Validation status: {}",
                            metadata.validation_status
                        ).unwrap();
                    }
                    Err(e) => {
                        writeln!(log_file, "❌ Failed to load metadata: {}", e).unwrap();
                    }
                }

                // Check if combined_audio.wav exists
                let combined_audio_path = format!("soundpacks/{}/combined_audio.wav", soundpack_id);
                if std::path::Path::new(&combined_audio_path).exists() {
                    match std::fs::metadata(&combined_audio_path) {
                        Ok(metadata) => {
                            writeln!(
                                log_file,
                                "🎵 Combined audio file: {} ({} bytes)",
                                combined_audio_path,
                                metadata.len()
                            ).unwrap();
                        }
                        Err(e) =>
                            writeln!(
                                log_file,
                                "⚠️  Error reading combined audio metadata: {}",
                                e
                            ).unwrap(),
                    }
                } else {
                    writeln!(
                        log_file,
                        "⚠️  Combined audio file not found: {}",
                        combined_audio_path
                    ).unwrap();
                }
            }
            Err(e) => {
                writeln!(log_file, "❌ Conversion failed: {}", e).unwrap();

                // Restore backup
                if std::path::Path::new(&backup_path).exists() {
                    match std::fs::copy(&backup_path, &config_path) {
                        Ok(_) =>
                            writeln!(log_file, "🔙 Restored original config from backup").unwrap(),
                        Err(e) => writeln!(log_file, "❌ Failed to restore backup: {}", e).unwrap(),
                    }
                }
            }
        }
    } else {
        writeln!(log_file, "ℹ️  Config doesn't need conversion or cannot be converted").unwrap();
    }

    writeln!(log_file, "🏁 Debug conversion test completed").unwrap();
}

fn test_import_conversion() {
    use std::io::Write;
    let mut log_file = std::fs::File
        ::create("import_conversion_test.log")
        .expect("Failed to create log file");

    writeln!(log_file, "🧪 Testing V1 to V2 automatic conversion during soundpack import").unwrap();
    let test_config =
        r#"{
  "name": "Test V1 Pack",
  "version": "1.0.0", 
  "author": "Test Author",
  "description": "A test V1 soundpack for testing conversion",
  "sound": "test_sound.wav",
  "key_define_type": "multi",
  "defines": {
    "30": "test_sound.wav",
    "31": "test_sound.wav", 
    "32": "test_sound.wav"
  }
}"#;

    let test_soundpack_id = "test-conversion-pack";

    writeln!(log_file, "📝 Test V1 config:").unwrap();
    writeln!(log_file, "{}", test_config).unwrap();

    // Test the handle_config_conversion function
    match utils::soundpack_installer::handle_config_conversion_test(test_config, test_soundpack_id) {
        Ok(converted_config) => {
            writeln!(log_file, "✅ Conversion successful!").unwrap();
            writeln!(log_file, "📄 Converted config:").unwrap();
            writeln!(log_file, "{}", converted_config).unwrap();
            // Check if it's V2 format
            if converted_config.contains("\"config_version\":2") {
                writeln!(log_file, "✅ Config version correctly set to 2").unwrap();
            } else {
                writeln!(log_file, "❌ Config version not set to 2").unwrap();
            }

            if converted_config.contains("\"defs\"") {
                writeln!(log_file, "✅ 'defs' field present (V2 format)").unwrap();
            } else {
                writeln!(log_file, "❌ 'defs' field missing").unwrap();
            }

            if converted_config.contains("\"source\"") {
                writeln!(log_file, "✅ 'source' field present (V2 format)").unwrap();
            } else {
                writeln!(log_file, "❌ 'source' field missing").unwrap();
            }
            if converted_config.contains("\"method\":\"multi\"") {
                writeln!(log_file, "✅ Method correctly set to 'multi'").unwrap();
            } else {
                writeln!(log_file, "❌ Method not correctly set to 'multi'").unwrap();
            }

            // Check if expected keys are present
            if converted_config.contains("\"KeyA\"") {
                writeln!(log_file, "✅ KeyA found in converted config").unwrap();
            } else {
                writeln!(log_file, "❌ KeyA not found in converted config").unwrap();
            }

            if converted_config.contains("\"KeyS\"") {
                writeln!(log_file, "✅ KeyS found in converted config").unwrap();
            } else {
                writeln!(log_file, "❌ KeyS not found in converted config").unwrap();
            }

            if converted_config.contains("\"KeyD\"") {
                writeln!(log_file, "✅ KeyD found in converted config").unwrap();
            } else {
                writeln!(log_file, "❌ KeyD not found in converted config").unwrap();
            }
        }
        Err(e) => {
            writeln!(log_file, "❌ Conversion failed: {}", e).unwrap();
        }
    }

    writeln!(log_file, "🏁 Import conversion test completed").unwrap();
}
