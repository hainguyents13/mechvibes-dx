use std::path::PathBuf;

#[cfg(target_os = "windows")]
use winreg::enums::*;
#[cfg(target_os = "windows")]
use winreg::RegKey;

const APP_NAME: &str = "MechvibesDX";

/// Get the current executable path
fn get_exe_path() -> Result<PathBuf, String> {
    std::env::current_exe().map_err(|e| format!("Failed to get executable path: {}", e))
}

/// Enable auto startup on Windows
#[cfg(target_os = "windows")]
pub fn enable_auto_startup() -> Result<(), String> {
    let exe_path = get_exe_path()?;
    let exe_path_str = exe_path.to_str().ok_or("Failed to convert executable path to string")?;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run_key = hkcu
        .open_subkey_with_flags("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run", KEY_WRITE)
        .map_err(|e| format!("Failed to open registry key: {}", e))?;

    run_key
        .set_value(APP_NAME, &exe_path_str)
        .map_err(|e| format!("Failed to set registry value: {}", e))?;

    println!("✅ Auto startup enabled: {}", exe_path_str);
    Ok(())
}

/// Disable auto startup on Windows
#[cfg(target_os = "windows")]
pub fn disable_auto_startup() -> Result<(), String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run_key = hkcu
        .open_subkey_with_flags("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run", KEY_WRITE)
        .map_err(|e| format!("Failed to open registry key: {}", e))?;

    match run_key.delete_value(APP_NAME) {
        Ok(_) => {
            println!("✅ Auto startup disabled");
            Ok(())
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // Entry doesn't exist, which is fine
            println!("ℹ️ Auto startup was not enabled");
            Ok(())
        }
        Err(e) => Err(format!("Failed to delete registry value: {}", e)),
    }
}

/// Check if auto startup is currently enabled
#[cfg(target_os = "windows")]
pub fn is_auto_startup_enabled() -> bool {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run_key = match hkcu.open_subkey("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run") {
        Ok(key) => key,
        Err(_) => {
            return false;
        }
    };

    match run_key.get_value::<String, _>(APP_NAME) {
        Ok(value) => {
            let current_exe = get_exe_path().unwrap_or_default();
            let current_exe_str = current_exe.to_str().unwrap_or("");
            value == current_exe_str
        }
        Err(_) => false,
    }
}

/// Set auto startup state (enable or disable)
pub fn set_auto_startup(enable: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        if enable { enable_auto_startup() } else { disable_auto_startup() }
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("Auto startup is only supported on Windows".to_string())
    }
}

/// Get current auto startup state
pub fn get_auto_startup_state() -> bool {
    #[cfg(target_os = "windows")]
    {
        is_auto_startup_enabled()
    }

    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}
