use std::env;
use std::process::Command;
use crate::utils::constants::{ APP_PROTOCOL, APP_NAME };

#[cfg(target_os = "linux")]
use crate::utils::constants::APP_NAME_LOWERCASE;

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

