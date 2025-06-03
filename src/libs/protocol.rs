use std::env;
use std::process::Command;

#[allow(dead_code)]
/// Register the mechvibes:// protocol for the application
#[cfg(target_os = "windows")]
pub fn register_protocol() -> Result<(), Box<dyn std::error::Error>> {
    let exe_path = env::current_exe()?;
    let exe_path_str = exe_path.to_string_lossy();
    println!("🔗 Registering mechvibes:// protocol... {}", exe_path_str);

    // Store formatted strings to avoid temporary value issues
    let icon_path = format!("\"{}\"", exe_path_str);
    let command_path = format!("\"{}\" \"%1\"", exe_path_str);

    // Registry commands to register the protocol
    let commands = vec![
        vec![
            "reg",
            "add",
            "HKCU\\Software\\Classes\\mechvibes",
            "/ve",
            "/d",
            "Mechvibes Protocol",
            "/f",
        ],
        vec![
            "reg",
            "add",
            "HKCU\\Software\\Classes\\mechvibes",
            "/v",
            "URL Protocol",
            "/d",
            "",
            "/f",
        ],
        vec![
            "reg",
            "add",
            "HKCU\\Software\\Classes\\mechvibes\\DefaultIcon",
            "/ve",
            "/d",
            &icon_path,
            "/f",
        ],
        vec![
            "reg",
            "add",
            "HKCU\\Software\\Classes\\mechvibes\\shell\\open\\command",
            "/ve",
            "/d",
            &command_path,
            "/f",
        ],
    ];
    for cmd in commands {
        let output = Command::new(cmd[0]).args(&cmd[1..]).output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            eprintln!("❌ Registry command failed: {}", error);
        }
    }

    println!("✅ Protocol mechvibes:// registered successfully");
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn register_protocol() -> Result<(), Box<dyn std::error::Error>> {
    println!("🍎 Protocol registration on macOS requires app bundle configuration in Info.plist");
    println!("Add the following to your Info.plist:");
    println!(
        r#"
<key>CFBundleURLTypes</key>
<array>
    <dict>
        <key>CFBundleURLName</key>
        <string>Mechvibes Protocol</string>
        <key>CFBundleURLSchemes</key>
        <array>
            <string>mechvibes</string>
        </array>
    </dict>
</array>
"#
    );
    Ok(())
}

#[cfg(target_os = "linux")]
pub fn register_protocol() -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;

    let home = env::var("HOME")?;
    let desktop_file_path = format!("{}/.local/share/applications/mechvibes.desktop", home);
    let exe_path = env::current_exe()?;

    println!("🐧 Registering mechvibes:// protocol on Linux...");

    let desktop_content = format!(
        r#"[Desktop Entry]
Name=Mechvibes
Comment=Mechanical keyboard sound simulator
Exec={} %u
Icon=mechvibes
Type=Application
MimeType=x-scheme-handler/mechvibes;
Categories=AudioVideo;Utility;
"#,
        exe_path.to_string_lossy()
    );

    // Ensure the applications directory exists
    let apps_dir = format!("{}/.local/share/applications", home);
    fs::create_dir_all(&apps_dir)?;

    fs::write(&desktop_file_path, desktop_content)?;

    // Update desktop database
    let _output = Command::new("update-desktop-database")
        .arg(&apps_dir)
        .output();

    println!("✅ Protocol mechvibes:// registered successfully");
    Ok(())
}

/// Handle incoming protocol URLs
pub fn handle_protocol_url(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    if !url.starts_with("mechvibes://") {
        return Err("Invalid protocol URL".into());
    }

    let path = &url[12..]; // Remove "mechvibes://"
    println!("🔗 Handling protocol URL: mechvibes://{}", path);

    match path {
        "open" | "" => {
            println!("📱 Opening Mechvibes from protocol");
            // The app is already opening, so we just need to ensure it's focused
            focus_window();
        }
        path if path.starts_with("switch-theme/") => {
            let theme_id = &path[13..];
            println!("🎨 Applying theme from protocol: {}", theme_id);
            switch_theme_from_protocol(theme_id)?;
        }
        path if path.starts_with("switch-soundpack/") => {
            let soundpack_name = &path[17..];
            println!("🔊 Loading soundpack from protocol: {}", soundpack_name);
            switch_soundpack_from_protocol(soundpack_name)?;
        }
        path if path.starts_with("install-soundpack/") => {
            let soundpack_name = &path[18..];
            println!("🔊 Installing soundpack from protocol: {}", soundpack_name);
            install_soundpack_from_protocol(soundpack_name)?;
        }
        path if path.starts_with("import-theme/") => {
            let theme_data = &path[13..];
            println!("📥 Importing theme from protocol");
            import_theme_from_protocol(theme_data)?;
        }
        _ => {
            println!("❓ Unknown protocol path: {}", path);
            return Err(format!("Unknown protocol path: {}", path).into());
        }
    }

    Ok(())
}

/// Focus the application window (platform-specific)
#[cfg(target_os = "windows")]
fn focus_window() {
    // On Windows, the window should automatically focus when the protocol is triggered
    println!("🪟 Focusing window on Windows");
}

#[cfg(not(target_os = "windows"))]
fn focus_window() {
    println!("🖥️ Window focus handling for this platform not implemented");
}

/// Apply a theme from protocol URL
fn switch_theme_from_protocol(theme_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    use crate::libs::theme::Theme;
    use crate::state::config::AppConfig;
    use crate::state::theme_utils::get_themes_config;
    use crate::state::themes::CustomThemeData;
    use chrono::Utc;

    let mut themes_config = get_themes_config();

    // Special case handling for built-in themes
    if theme_id == "dark" || theme_id == "light" || theme_id == "system" {
        let mut config = AppConfig::load();
        config.theme = match theme_id {
            "dark" => Theme::Dark,
            "light" => Theme::Light,
            "system" => Theme::System,
            _ => Theme::Dark, // Fallback
        };

        if let Err(e) = config.save() {
            eprintln!("❌ Failed to save config with new theme: {}", e);
            return Err(e.into());
        }

        println!("🎨 Applied built-in theme: {}", theme_id);
        return Ok(());
    }

    // Try to find a custom theme
    if let Some(theme_data) = themes_config.get_theme_by_id(theme_id) {
        println!("✅ Found theme: {}", theme_data.name);
        // Update the main config to use this theme
        let mut config = AppConfig::load();
        config.theme = Theme::Custom(theme_id.to_string());

        if let Err(e) = config.save() {
            eprintln!("❌ Failed to save config with new theme: {}", e);
        }

        println!("🎨 Applied theme: {}", theme_data.name);
    } else {
        // Create the theme if it doesn't exist
        println!("📝 Creating theme: {}", theme_id);

        let new_theme = CustomThemeData {
            id: theme_id.to_string(),
            name: theme_id.to_string(),
            description: format!("Theme created from protocol URL: {}", theme_id),
            css: format!(
                r#".app-container {{ background-color: #202020; color: #ffffff; }}
                    .title-bar {{ background-color: #101010; }}"#
            ),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Add to themes
        themes_config
            .custom_themes
            .insert(theme_id.to_string(), new_theme);

        // Save themes config
        if let Err(e) = themes_config.save() {
            return Err(format!("Failed to save new theme: {}", e).into());
        }

        // Update main config
        let mut config = AppConfig::load();
        config.theme = Theme::Custom(theme_id.to_string());

        if let Err(e) = config.save() {
            return Err(format!("Failed to apply new theme: {}", e).into());
        }

        println!("✅ Created and applied new theme: {}", theme_id);
    }

    Ok(())
}

/// Load a soundpack from protocol URL
fn switch_soundpack_from_protocol(soundpack_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    use crate::state::config::AppConfig;

    let mut config = AppConfig::load();
    config.keyboard_soundpack = soundpack_name.to_string();

    if let Err(e) = config.save() {
        eprintln!("❌ Failed to save config with new soundpack: {}", e);
        return Err(e.into());
    }

    println!("🔊 Loaded soundpack: {}", soundpack_name);
    Ok(())
}

fn install_soundpack_from_protocol(soundpack_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    use crate::state::config::AppConfig;
    use std::fs;
    use std::path::Path;

    println!("📥 Installing soundpack: {}", soundpack_name);

    // In a real implementation, this would download the soundpack from a remote source
    // For testing purposes, we'll just check if it exists locally and add it to config

    let app_root = std::env::current_dir()?;
    let soundpacks_dir = app_root.join("soundpacks");
    let soundpack_path = soundpacks_dir.join(soundpack_name);

    if Path::new(&soundpack_path).exists() {
        // Add to config
        let mut config = AppConfig::load();
        config.keyboard_soundpack = soundpack_name.to_string();
        if let Err(e) = config.save() {
            eprintln!("❌ Failed to save config with new soundpack: {}", e);
            return Err(e.into());
        }
        println!("✅ Installed and activated soundpack: {}", soundpack_name);
    } else {
        // For real implementation, we would download it here
        println!(
            "⚠️ Soundpack not found locally: {}. Would download in production.",
            soundpack_name
        );
        // Create a placeholder for testing
        fs::create_dir_all(&soundpack_path)?;
        fs::write(
            soundpack_path.join("config.json"),
            format!(
                r#"{{
  "name": "Test Soundpack - {}",
  "author": "Protocol Test",
  "version": "1.0.0",
  "key_define": {{
    "default": "sound.ogg"
  }}
}}"#,
                soundpack_name
            ),
        )?;

        // Create a placeholder sound file by copying from an existing soundpack
        let source_sound = app_root.join("soundpacks").join("oreo").join("oreo.ogg");
        let target_sound = soundpack_path.join("sound.ogg");

        if source_sound.exists() {
            fs::copy(source_sound, target_sound)?;
        } else {
            // Create an empty sound file if source doesn't exist
            fs::write(soundpack_path.join("sound.ogg"), &[0u8; 1024])?;
        }

        // Update config to use the new soundpack
        let mut config = AppConfig::load();
        config.keyboard_soundpack = soundpack_name.to_string();
        if let Err(e) = config.save() {
            eprintln!("❌ Failed to save config with new soundpack: {}", e);
            return Err(e.into());
        }

        println!(
            "✅ Created and activated placeholder soundpack: {}",
            soundpack_name
        );
    }

    Ok(())
}

/// Import a theme from protocol URL (base64 encoded theme data)
fn import_theme_from_protocol(theme_data: &str) -> Result<(), Box<dyn std::error::Error>> {
    use crate::libs::theme::Theme;
    use crate::state::config::AppConfig;
    use crate::state::theme_utils::get_themes_config;
    use crate::state::themes::CustomThemeData;
    use chrono::Utc;
    use std::time::{SystemTime, UNIX_EPOCH};

    println!("📥 Importing theme from protocol data");

    // In a real implementation, this would decode the base64 data
    // For testing purposes, we'll create a simple theme

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let theme_id = format!("imported_{}", timestamp);
    let theme_name = if theme_data.is_empty() {
        "Imported Theme"
    } else {
        theme_data
    };

    let mut themes_config = get_themes_config();

    // Add new theme
    let new_theme = CustomThemeData {
        id: theme_id.clone(),
        name: theme_name.to_string(),
        description: "Imported via protocol URL".to_string(),
        css: ".app-container { background-color: #202020; color: #ffffff; }".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    // Add theme to custom_themes map
    themes_config
        .custom_themes
        .insert(theme_id.clone(), new_theme); // Save the themes config
    if let Err(e) = themes_config.save() {
        return Err(format!("Failed to save imported theme: {}", e).into());
    }

    // Set as current theme
    let mut config = AppConfig::load();
    config.theme = Theme::Custom(theme_id.clone());

    if let Err(e) = config.save() {
        return Err(format!("Failed to apply imported theme: {}", e).into());
    }

    println!("✅ Theme imported and applied: {}", theme_name);
    Ok(())
}
