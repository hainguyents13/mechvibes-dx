use std::env;
use std::process::Command;

/// Register the mechvibes:// protocol for the application
#[cfg(target_os = "windows")]
pub fn register_protocol() -> Result<(), Box<dyn std::error::Error>> {
    let exe_path = env::current_exe()?;
    let exe_path_str = exe_path.to_string_lossy();
    println!("🔗 Registering mechvibes:// protocol...");

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
        path if path.starts_with("theme/") => {
            let theme_id = &path[6..];
            println!("🎨 Applying theme from protocol: {}", theme_id);
            apply_theme_from_protocol(theme_id)?;
        }
        path if path.starts_with("soundpack/") => {
            let soundpack_name = &path[10..];
            println!("🔊 Loading soundpack from protocol: {}", soundpack_name);
            load_soundpack_from_protocol(soundpack_name)?;
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
fn apply_theme_from_protocol(theme_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    use crate::libs::theme::Theme;
    use crate::state::config::AppConfig;
    use crate::state::theme_utils::get_themes_config;

    let themes_config = get_themes_config();

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
        return Err(format!("Theme not found: {}", theme_id).into());
    }

    Ok(())
}

/// Load a soundpack from protocol URL
fn load_soundpack_from_protocol(soundpack_name: &str) -> Result<(), Box<dyn std::error::Error>> {
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

/// Import a theme from protocol URL (base64 encoded theme data)
fn import_theme_from_protocol(_theme_data: &str) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Implement theme import from base64 encoded data
    println!("📥 Theme import from protocol not yet implemented");
    Ok(())
}
