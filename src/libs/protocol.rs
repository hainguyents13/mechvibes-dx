use std::env;
use std::process::Command;
use crate::utils::constants::{ APP_PROTOCOL, APP_PROTOCOL_URL, APP_NAME, APP_NAME_LOWERCASE };

#[allow(dead_code)]
/// Register the mechvibes:// protocol for the application
#[cfg(target_os = "windows")]
pub fn register_protocol() -> Result<(), Box<dyn std::error::Error>> {
    let exe_path = env::current_exe()?;
    let exe_path_str = exe_path.to_string_lossy();
    println!("🔗 Registering {}// protocol... {}", APP_PROTOCOL, exe_path_str); // Store formatted strings to avoid temporary value issues
    let icon_path = format!("\"{}\"", exe_path_str);
    let command_path = format!("\"{}\" \"%1\"", exe_path_str); // Registry commands to register the protocol
    let protocol_key = format!("HKCU\\Software\\Classes\\{}", APP_PROTOCOL);
    let protocol_description = format!("{} Protocol", APP_NAME);
    let default_icon_key = format!("{}\\DefaultIcon", protocol_key);
    let shell_command_key = format!("{}\\shell\\open\\command", protocol_key);

    let commands = vec![
        vec!["reg", "add", &protocol_key, "/ve", "/d", &protocol_description, "/f"],
        vec!["reg", "add", &protocol_key, "/v", "URL Protocol", "/d", "", "/f"],
        vec!["reg", "add", &default_icon_key, "/ve", "/d", &icon_path, "/f"],
        vec!["reg", "add", &shell_command_key, "/ve", "/d", &command_path, "/f"]
    ];
    for cmd in commands {
        let output = Command::new(cmd[0])
            .args(&cmd[1..])
            .output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            eprintln!("❌ Registry command failed: {}", error);
        }
    }

    println!("✅ Protocol {}// registered successfully", APP_PROTOCOL);
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn register_protocol() -> Result<(), Box<dyn std::error::Error>> {
    println!("🍎 Protocol registration on macOS requires app bundle configuration in Info.plist");
    println!("Add the following to your Info.plist:");
    println!(
        r#"
<key>CFBundleURLTypes</key>    <array>
        <dict>
            <key>CFBundleURLName</key>
            <string>{} Protocol</string>
            <key>CFBundleURLSchemes</key>
            <array>
                <string>{}</string>
            </array>
        </dict>
    </array>
"#,
        APP_NAME,
        APP_PROTOCOL
    );
    Ok(())
}

#[cfg(target_os = "linux")]
pub fn register_protocol() -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;

    let home = env::var("HOME")?;
    let desktop_file_path = format!(
        "{}/.local/share/applications/{}.desktop",
        home,
        APP_NAME_LOWERCASE
    );
    let exe_path = env::current_exe()?;

    println!("🐧 Registering {}// protocol on Linux...", APP_PROTOCOL);

    let desktop_content = format!(
        r#"[Desktop Entry]
Name={}
Comment=Mechanical keyboard sound simulator
Exec={} %u
Icon={}
Type=Application
MimeType=x-scheme-handler/{};
Categories=AudioVideo;Utility;
"#,
        APP_NAME,
        exe_path.to_string_lossy(),
        APP_NAME_LOWERCASE,
        APP_PROTOCOL
    );

    // Ensure the applications directory exists
    let apps_dir = format!("{}/.local/share/applications", home);
    fs::create_dir_all(&apps_dir)?;
    fs::write(&desktop_file_path, desktop_content)?;

    // Update desktop database
    let _output = Command::new("update-desktop-database").arg(&apps_dir).output();

    println!("✅ Protocol {}// registered successfully", APP_PROTOCOL);
    Ok(())
}

/// Handle incoming protocol URLs
pub fn handle_protocol_url(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    if !url.starts_with(APP_PROTOCOL_URL) {
        return Err("Invalid protocol URL".into());
    }

    let protocol_prefix_len = APP_PROTOCOL_URL.len();
    let path = &url[protocol_prefix_len..]; // Remove protocol prefix
    println!("🔗 Handling protocol URL: {}{}", APP_PROTOCOL_URL, path);

    match path {
        "open" | "" => {
            println!("📱 Opening {} from protocol", APP_NAME);
            // The app is already opening, so we just need to ensure it's focused
            focus_window();
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
        println!("⚠️ Soundpack not found locally: {}. Would download in production.", soundpack_name);
        // Create a placeholder for testing
        fs::create_dir_all(&soundpack_path)?;
        fs::write(
            soundpack_path.join("config.json"),
            format!(r#"{{
  "name": "Test Soundpack - {}",
  "author": "Protocol Test",
  "version": "1.0.0",
  "key_define": {{
    "default": "sound.ogg"
  }}
}}"#, soundpack_name)
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

        println!("✅ Created and activated placeholder soundpack: {}", soundpack_name);
    }

    Ok(())
}

/// Import a theme from protocol URL (base64 encoded theme data)
fn import_theme_from_protocol(theme_data: &str) -> Result<(), Box<dyn std::error::Error>> {
    use crate::libs::theme::Theme;
    use crate::state::config::AppConfig;
    use crate::state::themes::CustomThemeData;
    use crate::utils::theme::get_themes_config;
    use chrono::Utc;
    use std::time::{ SystemTime, UNIX_EPOCH };

    println!("📥 Importing theme from protocol data");

    // In a real implementation, this would decode the base64 data
    // For testing purposes, we'll create a simple theme

    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

    let theme_id = format!("imported-{}", timestamp);
    let theme_name = if theme_data.is_empty() { "Imported Theme" } else { theme_data };

    let mut themes_config = get_themes_config();

    // Add new theme
    let new_theme = CustomThemeData {
        id: theme_id.clone(),
        name: theme_name.to_string(),
        description: "Imported via protocol URL".to_string(),
        css: ".app-container { background-color: #202020; color: #ffffff; }".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        is_built_in: false, // Mark as custom theme
    };

    // Add theme to custom_themes map
    themes_config.custom_themes.insert(theme_id.clone(), new_theme);

    // Save the themes config
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
