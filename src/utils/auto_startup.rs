use std::path::PathBuf;
use crate::utils::constants::APP_NAME;

#[cfg(target_os = "windows")]
use winreg::enums::*;
#[cfg(target_os = "windows")]
use winreg::RegKey;

/// Get the current executable path
fn get_exe_path() -> Result<PathBuf, String> {
    std::env::current_exe().map_err(|e| format!("Failed to get executable path: {}", e))
}

/// Enable auto startup on Windows
#[cfg(target_os = "windows")]
pub fn enable_auto_startup() -> Result<(), String> {
    let exe_path = get_exe_path()?;
    let exe_path_str = exe_path.to_str().ok_or("Failed to convert executable path to string")?;

    // Check if we should start minimized
    let config = crate::state::config::AppConfig::load();
    let command = if config.start_minimized {
        format!("\"{}\" --minimized", exe_path_str)
    } else {
        exe_path_str.to_string()
    };

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run_key = hkcu
        .open_subkey_with_flags("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run", KEY_WRITE)
        .map_err(|e| format!("Failed to open registry key: {}", e))?;

    run_key
        .set_value(APP_NAME, &command)
        .map_err(|e| format!("Failed to set registry value: {}", e))?;

    println!("✅ Auto startup enabled: {}", command);
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

/// Pull the executable path back out of a registry Run command line.
///
/// `enable_auto_startup` writes two shapes depending on `start_minimized`:
/// a bare path, or `"path" --minimized`. Comparing the whole value against
/// the bare path only ever matches the first shape, so with "start minimized"
/// on the check reported "not enabled" for an entry it had just written -
/// and `AppConfig::load` then "synced" `auto_start` back to false and saved.
/// Both shapes have to round-trip.
fn exe_path_from_run_command(command: &str) -> &str {
    let command = command.trim();

    // Quoted form: take what is between the first pair of quotes and ignore
    // any arguments that follow.
    if let Some(rest) = command.strip_prefix('"') {
        return match rest.find('"') {
            Some(end) => &rest[..end],
            // Unbalanced quote - no sane reading, treat the remainder as path.
            None => rest,
        };
    }

    // Unquoted form is written without arguments, so the whole value is the
    // path. Splitting on whitespace here would break paths like
    // `C:\Program Files\...`.
    command
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
            let current_exe_str = match current_exe.to_str() {
                Some(path) if !path.is_empty() => path,
                // Without a path to compare against, "" would match a "" value
                // and claim auto start is on for an entry pointing elsewhere.
                _ => {
                    return false;
                }
            };

            // Windows paths are case-insensitive, and the registry value was
            // written by a possibly different-cased invocation of this exe.
            exe_path_from_run_command(&value).eq_ignore_ascii_case(current_exe_str)
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

#[cfg(test)]
mod tests {
    use super::exe_path_from_run_command;

    /// Mirrors the unquoted shape `enable_auto_startup` writes when
    /// `start_minimized` is off.
    #[test]
    fn a_bare_path_is_returned_unchanged() {
        let path = r"C:\Program Files\MechvibesDX\mechvibes-dx.exe";
        assert_eq!(exe_path_from_run_command(path), path);
    }

    /// The shape that used to be unrecognizable: with "start minimized" on the
    /// value carries quotes and an argument, and the old whole-string compare
    /// reported "not enabled" for an entry this app had just written.
    #[test]
    fn a_quoted_path_with_the_minimized_flag_yields_just_the_path() {
        let path = r"C:\Program Files\MechvibesDX\mechvibes-dx.exe";
        let command = format!("\"{}\" --minimized", path);
        assert_eq!(exe_path_from_run_command(&command), path);
    }

    /// A quoted path with no arguments must also round-trip, since that is
    /// what a user or an older build may have left behind.
    #[test]
    fn a_quoted_path_without_arguments_yields_just_the_path() {
        let path = r"C:\Apps\mechvibes-dx.exe";
        assert_eq!(exe_path_from_run_command(&format!("\"{}\"", path)), path);
    }

    /// Spaces inside an unquoted path must not be treated as an argument
    /// separator, or every "Program Files" install compares unequal and
    /// self-disables exactly like the quoted bug did.
    #[test]
    fn spaces_in_an_unquoted_path_do_not_truncate_it() {
        let path = r"C:\Program Files\MechvibesDX\mechvibes-dx.exe";
        assert_eq!(exe_path_from_run_command(path), path);
    }

    /// The registry value is written by whatever casing launched the app;
    /// the enabled check compares case-insensitively, so both forms must
    /// reduce to a path that compares equal to the real one.
    #[test]
    fn both_written_forms_match_the_exe_case_insensitively() {
        let exe = r"C:\Program Files\MechvibesDX\mechvibes-dx.exe";
        let bare = r"c:\program files\mechvibesdx\MECHVIBES-DX.EXE";
        let minimized = format!("\"{}\" --minimized", bare);

        assert!(exe_path_from_run_command(bare).eq_ignore_ascii_case(exe));
        assert!(exe_path_from_run_command(&minimized).eq_ignore_ascii_case(exe));
    }

    /// A Run entry pointing at some other program must not be read as "our
    /// auto start is on", or the config sync would flip `auto_start` true.
    #[test]
    fn a_different_executable_does_not_match() {
        let exe = r"C:\Program Files\MechvibesDX\mechvibes-dx.exe";
        let other = "\"C:\\Other\\thing.exe\" --minimized";
        assert!(!exe_path_from_run_command(other).eq_ignore_ascii_case(exe));
    }
}
