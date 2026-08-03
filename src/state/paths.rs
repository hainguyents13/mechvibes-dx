/// Centralized path definitions
///
/// ## Path Structure
/// - `data/` - Application data and configuration files (relative to app root)
/// - `soundpacks/` - Built-in soundpack directories (relative to app root)
/// - Custom soundpacks - Stored in system app data directory (e.g., %APPDATA%/Mechvibes/soundpacks)
/// - Custom images - Stored in system app data directory (e.g., %APPDATA%/Mechvibes/custom_images)
///
/// All paths are relative to the application executable directory unless specified otherwise.
///
/// ## macOS `.app` bundles
///
/// Inside a bundle the executable does not sit next to its resources:
///
/// ```text
/// MechvibesDX.app/Contents/MacOS/mechvibes-dx   <- current_exe()
/// MechvibesDX.app/Contents/Resources/soundpacks
/// MechvibesDX.app/Contents/Resources/assets
/// ```
///
/// So the read-only resource root becomes `../Resources`, which matches where
/// `dioxus-asset-resolver` looks for `asset!()` files on macOS. Writable state
/// must NOT go there: an app installed in `/Applications` is not user-writable,
/// so `data/` moves to the system app data dir on macOS only.
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Resource root for a macOS `.app` bundle, if `exe_path` is inside one.
///
/// Detection is structural rather than "am I on macOS": the same binary is also
/// run bare (`cargo build && ./mechvibes-dx`) during development, and that layout
/// must keep resolving resources next to the executable. Only the exact
/// `<name>.app/Contents/MacOS/<exe>` shape counts as a bundle.
///
/// Pure string/path logic with no filesystem access, so it is testable on every
/// platform - the CI that verifies this runs on Windows.
fn macos_bundle_resource_root(exe_path: &Path) -> Option<PathBuf> {
    let macos_dir = exe_path.parent()?;
    if macos_dir.file_name()? != "MacOS" {
        return None;
    }

    let contents_dir = macos_dir.parent()?;
    if contents_dir.file_name()? != "Contents" {
        return None;
    }

    // The bundle directory itself must end in `.app`, otherwise a project that
    // merely has `Contents/MacOS/` somewhere in its tree would be misdetected.
    let bundle_dir = contents_dir.parent()?;
    let bundle_name = bundle_dir.file_name()?.to_str()?;
    if !bundle_name.ends_with(".app") {
        return None;
    }

    Some(contents_dir.join("Resources"))
}

/// Get the application root directory (where the executable is located)
/// This ensures resources are found regardless of working directory
fn get_app_root() -> &'static PathBuf {
    static APP_ROOT: OnceLock<PathBuf> = OnceLock::new();
    APP_ROOT.get_or_init(|| {
        // Try to get the directory where the executable is located
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                // Check if running in dev mode (dx serve creates target/dx/... path)
                let exe_path_str = exe_path.to_string_lossy();
                if exe_path_str.contains("target\\dx\\") || exe_path_str.contains("target/dx/") {
                    // In dev mode, use current working directory (project root)
                    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
                    println!("📂 App root (dev mode - from cwd): {}", cwd.display());
                    return cwd;
                }

                // Inside a macOS .app the resources live in ../Resources, not
                // beside the executable in Contents/MacOS.
                if let Some(resources) = macos_bundle_resource_root(&exe_path) {
                    println!("📂 App root (macOS bundle): {}", resources.display());
                    return resources;
                }

                println!("📂 App root (from exe): {}", exe_dir.display());
                return exe_dir.to_path_buf();
            }
        }

        // Fallback to current working directory (for development)
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        println!("📂 App root (fallback - from cwd): {}", cwd.display());
        cwd
    })
}

/// True when this process is running from inside a macOS `.app` bundle.
///
/// Used to route *writable* state away from the bundle: `/Applications` is not
/// user-writable, so a config written next to the resources would silently fail
/// to save and every setting would reset on restart.
fn is_running_from_macos_bundle() -> bool {
    static IN_BUNDLE: OnceLock<bool> = OnceLock::new();
    *IN_BUNDLE.get_or_init(|| {
        std::env::current_exe()
            .ok()
            .and_then(|exe| macos_bundle_resource_root(&exe))
            .is_some()
    })
}

/// Directory holding writable application data (`config.json`, `themes.json`,
/// the soundpack cache).
///
/// Normally `{app_root}/data`, which keeps the Windows portable/installed layout
/// and the Linux behavior exactly as they were. Inside a macOS `.app` it becomes
/// `~/Library/Application Support/Mechvibes/data`, because the bundle is
/// read-only for anyone who did not install it as an admin.
fn get_writable_data_dir() -> PathBuf {
    if is_running_from_macos_bundle() {
        get_system_app_data_dir().join("data")
    } else {
        get_app_root().join("data")
    }
}

/// Get the system app data directory for Mechvibes
/// Returns platform-specific app data directory:
/// - Windows: %APPDATA%/Mechvibes
/// - macOS: ~/Library/Application Support/Mechvibes
/// - Linux: ~/.local/share/mechvibes
fn get_system_app_data_dir() -> PathBuf {
    use directories::BaseDirs;

    if let Some(base_dirs) = BaseDirs::new() {
        #[cfg(target_os = "windows")]
        {
            // Windows: %APPDATA%/Mechvibes
            base_dirs.data_dir().join("Mechvibes")
        }
        #[cfg(target_os = "macos")]
        {
            // macOS: ~/Library/Application Support/Mechvibes
            base_dirs.data_dir().join("Mechvibes")
        }
        #[cfg(target_os = "linux")]
        {
            // Linux: ~/.local/share/mechvibes
            base_dirs.data_dir().join("mechvibes")
        }
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            // Other Unix-like systems
            base_dirs.data_dir().join("mechvibes")
        }
    } else {
        // Fallback to app root if system directories not available
        get_app_root().join("data")
    }
}

/// Application data directory paths
pub mod data {
    use super::{get_system_app_data_dir, get_writable_data_dir};
    use std::path::PathBuf;

    /// Application configuration file
    pub fn config_json() -> PathBuf {
        get_writable_data_dir().join("config.json")
    }

    /// Application manifest file
    pub fn manifest_json() -> PathBuf {
        get_writable_data_dir().join("manifest.json")
    }

    /// Custom themes configuration file
    pub fn themes_json() -> PathBuf {
        get_writable_data_dir().join("themes.json")
    }

    /// Soundpack cache file
    pub fn soundpack_cache_json() -> PathBuf {
        get_writable_data_dir().join("soundpack_cache.json")
    }

    /// Custom images directory for user-uploaded images
    /// Uses system app data directory (e.g., %APPDATA%/Mechvibes/custom_images on Windows)
    pub fn custom_images_dir() -> PathBuf {
        get_system_app_data_dir().join("custom_images")
    }
}

/// Soundpack directory paths
pub mod soundpacks {
    use super::{get_app_root, get_system_app_data_dir};
    use std::path::{Path, PathBuf};

    /// List of built-in soundpack IDs that ship with the app
    /// These are stored in the app root soundpacks directory
    pub const BUILTIN_SOUNDPACKS: &[&str] = &[
        "keyboard/cherrymx-black-abs",
        "keyboard/cherrymx-black-pbt",
        "keyboard/cherrymx-blue-abs",
        "keyboard/cherrymx-blue-pbt",
        "keyboard/cherrymx-brown-abs",
        "keyboard/cherrymx-brown-pbt",
        "keyboard/cherrymx-red-abs",
        "keyboard/cherrymx-red-pbt",
        "keyboard/eg-crystal-purple",
        "keyboard/eg-oreo",
        "keyboard/topre-purple-hybrid-pbt",
        "mouse/chat",
        "mouse/ping",
        "mouse/vibrate",
        "mouse/wooden",
    ];

    /// Check if a soundpack ID is a built-in soundpack
    pub fn is_builtin_soundpack(soundpack_id: &str) -> bool {
        BUILTIN_SOUNDPACKS.contains(&soundpack_id)
    }

    /// Get the base soundpacks directory for built-in soundpacks
    /// Checks multiple locations in order:
    /// 1. /usr/share/mechvibes-dx/soundpacks (installed via DEB/system package)
    /// 2. {app_root}/soundpacks (portable/dev mode)
    pub fn get_builtin_soundpacks_dir() -> PathBuf {
        // Check standard Linux data directory first (for installed packages)
        #[cfg(target_os = "linux")]
        {
            let system_soundpacks = PathBuf::from("/usr/share/mechvibes-dx/soundpacks");
            if system_soundpacks.exists() {
                println!("📂 Using system soundpacks directory: {}", system_soundpacks.display());
                return system_soundpacks;
            }
        }

        // Fallback to app root (for portable/dev mode)
        let app_root_soundpacks = get_app_root().join("soundpacks");
        println!("📂 Using app root soundpacks directory: {}", app_root_soundpacks.display());
        app_root_soundpacks
    }

    /// Get the base soundpacks directory for custom soundpacks (system app data)
    pub fn get_custom_soundpacks_dir() -> PathBuf {
        get_system_app_data_dir().join("soundpacks")
    }

    /// Get soundpack directory path for a specific soundpack ID
    /// Checks built-in location first, then custom location
    /// soundpack_id format: "keyboard/Soundpack Name" or "mouse/Soundpack Name"
    pub fn soundpack_dir(soundpack_id: &str) -> String {
        // Normalize the soundpack_id by splitting on both / and \ and rejoining with PathBuf
        let parts: Vec<&str> = soundpack_id.split(&['/', '\\'][..]).collect();

        // Check if it's a built-in soundpack
        if is_builtin_soundpack(soundpack_id) {
            let mut path = get_builtin_soundpacks_dir();
            for part in parts {
                path = path.join(part);
            }
            path.to_string_lossy().to_string()
        } else {
            // Check custom location first
            let mut custom_path = get_custom_soundpacks_dir();
            for part in &parts {
                custom_path = custom_path.join(part);
            }

            if custom_path.exists() {
                custom_path.to_string_lossy().to_string()
            } else {
                // Fallback to built-in location (for backwards compatibility)
                let mut builtin_path = get_builtin_soundpacks_dir();
                for part in parts {
                    builtin_path = builtin_path.join(part);
                }
                builtin_path.to_string_lossy().to_string()
            }
        }
    }

    /// Get config.json path for a specific soundpack
    /// soundpack_id format: "keyboard/Soundpack Name" or "mouse/Soundpack Name"
    pub fn config_json(soundpack_id: &str) -> String {
        Path::new(&soundpack_dir(soundpack_id))
            .join("config.json")
            .to_string_lossy()
            .to_string()
    }

    /// Get the base soundpacks directory (containing keyboard/ and mouse/ folders)
    /// Returns built-in soundpacks directory
    #[allow(dead_code)]
    pub fn get_soundpacks_dir() -> String {
        get_builtin_soundpacks_dir().to_string_lossy().to_string()
    }

    /// Get keyboard soundpacks directory (built-in)
    #[allow(dead_code)]
    pub fn keyboard_soundpacks_dir() -> String {
        get_builtin_soundpacks_dir()
            .join("keyboard")
            .to_string_lossy()
            .to_string()
    }

    /// Get mouse soundpacks directory (built-in)
    #[allow(dead_code)]
    pub fn mouse_soundpacks_dir() -> String {
        get_builtin_soundpacks_dir()
            .join("mouse")
            .to_string_lossy()
            .to_string()
    }

    /// Get custom keyboard soundpacks directory (system app data)
    #[allow(dead_code)]
    pub fn custom_keyboard_soundpacks_dir() -> String {
        get_custom_soundpacks_dir()
            .join("keyboard")
            .to_string_lossy()
            .to_string()
    }

    /// Get custom mouse soundpacks directory (system app data)
    #[allow(dead_code)]
    pub fn custom_mouse_soundpacks_dir() -> String {
        get_custom_soundpacks_dir()
            .join("mouse")
            .to_string_lossy()
            .to_string()
    }

    /// Ensure soundpack directories exist (keyboard and mouse)
    /// Creates the directories if they don't exist
    ///
    /// On Linux, built-in soundpacks are installed to system directories by DEB/AppImage
    /// and should not be created here (would require root permissions).
    /// Only custom soundpack directories are created (in user's home directory).
    pub fn ensure_soundpack_directories() -> Result<(), std::io::Error> {
        use std::fs;

        // Check if built-in soundpack directories exist (don't try to create them on Linux)
        let builtin_soundpacks_dir = get_builtin_soundpacks_dir();

        #[cfg(not(target_os = "linux"))]
        {
            // On Windows/macOS, create built-in soundpack directories if needed
            let builtin_keyboard_dir = builtin_soundpacks_dir.join("keyboard");
            let builtin_mouse_dir = builtin_soundpacks_dir.join("mouse");

            if !builtin_soundpacks_dir.exists() {
                fs::create_dir_all(&builtin_soundpacks_dir)?;
                crate::debug_print!(
                    "📁 Created built-in soundpacks directory: {}",
                    builtin_soundpacks_dir.display()
                );
            }

            if !builtin_keyboard_dir.exists() {
                fs::create_dir_all(&builtin_keyboard_dir)?;
                crate::debug_print!(
                    "⌨️ Created built-in keyboard soundpacks directory: {}",
                    builtin_keyboard_dir.display()
                );
            }

            if !builtin_mouse_dir.exists() {
                fs::create_dir_all(&builtin_mouse_dir)?;
                crate::debug_print!(
                    "🖱️ Created built-in mouse soundpacks directory: {}",
                    builtin_mouse_dir.display()
                );
            }
        }

        #[cfg(target_os = "linux")]
        {
            // On Linux, just log the built-in soundpacks directory location
            // (should be installed by DEB/AppImage package)
            crate::debug_print!(
                "📂 Built-in soundpacks directory: {}",
                builtin_soundpacks_dir.display()
            );
            if !builtin_soundpacks_dir.exists() {
                crate::debug_print!(
                    "⚠️  Built-in soundpacks directory not found (expected for installed packages)"
                );
            }
        }

        // Ensure custom soundpack directories exist
        let custom_soundpacks_dir = get_custom_soundpacks_dir();
        let custom_keyboard_dir = custom_soundpacks_dir.join("keyboard");
        let custom_mouse_dir = custom_soundpacks_dir.join("mouse");

        if !custom_soundpacks_dir.exists() {
            fs::create_dir_all(&custom_soundpacks_dir)?;
            crate::debug_print!(
                "📁 Created custom soundpacks directory: {}",
                custom_soundpacks_dir.display()
            );
        }

        if !custom_keyboard_dir.exists() {
            fs::create_dir_all(&custom_keyboard_dir)?;
            crate::debug_print!(
                "⌨️ Created custom keyboard soundpacks directory: {}",
                custom_keyboard_dir.display()
            );
        }

        if !custom_mouse_dir.exists() {
            fs::create_dir_all(&custom_mouse_dir)?;
            crate::debug_print!(
                "🖱️ Created custom mouse soundpacks directory: {}",
                custom_mouse_dir.display()
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::macos_bundle_resource_root;
    use std::path::{Path, PathBuf};

    /// The shipping layout: resources resolve to the sibling `Resources` dir,
    /// NOT to `Contents/MacOS` where the executable sits.
    #[test]
    fn detects_the_app_bundle_layout() {
        let exe = Path::new("/Applications/MechvibesDX.app/Contents/MacOS/mechvibes-dx");
        assert_eq!(
            macos_bundle_resource_root(exe),
            Some(PathBuf::from("/Applications/MechvibesDX.app/Contents/Resources"))
        );
    }

    /// The bundle can be anywhere, not just /Applications (users run it from
    /// ~/Downloads or straight off the mounted DMG).
    #[test]
    fn bundle_detection_is_independent_of_install_location() {
        for exe in [
            "/Users/someone/Downloads/MechvibesDX.app/Contents/MacOS/mechvibes-dx",
            "/Volumes/MechvibesDX/MechvibesDX.app/Contents/MacOS/mechvibes-dx",
        ] {
            assert_eq!(
                macos_bundle_resource_root(Path::new(exe)),
                Some(Path::new(exe).parent().unwrap().parent().unwrap().join("Resources")),
                "failed for {}",
                exe
            );
        }
    }

    /// The executable name is not fixed - a rename must not break resolution.
    #[test]
    fn bundle_detection_ignores_the_executable_name() {
        let exe = Path::new("/Applications/MechvibesDX.app/Contents/MacOS/AnythingElse");
        assert!(macos_bundle_resource_root(exe).is_some());
    }

    /// Everything that is NOT a bundle must fall through to the existing
    /// "resources sit beside the exe" behavior. This is the regression guard
    /// for Windows and Linux, and it runs on every platform.
    #[test]
    fn non_bundle_layouts_are_never_treated_as_bundles() {
        let non_bundles = [
            // Windows installed and portable layouts
            r"C:\Program Files\MechvibesDX\mechvibes-dx.exe",
            r"D:\mechvibes-dx\target\release\mechvibes-dx.exe",
            // Linux
            "/usr/bin/mechvibes-dx",
            "/opt/mechvibes-dx/mechvibes-dx",
            // A bare macOS dev build - same binary, no bundle around it
            "/Users/someone/mechvibes-dx/target/release/mechvibes-dx",
            // Right directory names, but no `.app` wrapper
            "/Users/someone/Contents/MacOS/mechvibes-dx",
            // `.app` present but the inner structure is wrong
            "/Applications/MechvibesDX.app/mechvibes-dx",
            "/Applications/MechvibesDX.app/Contents/mechvibes-dx",
            // Only one directory level - nothing to walk up to
            "mechvibes-dx",
        ];

        for exe in non_bundles {
            assert_eq!(
                macos_bundle_resource_root(Path::new(exe)),
                None,
                "'{}' was wrongly detected as a macOS .app bundle",
                exe
            );
        }
    }

    /// `Contents/MacOS` is case-sensitive on purpose: that is the exact casing
    /// Apple's bundle format uses, and matching loosely would risk classifying
    /// an unrelated Windows path as a bundle.
    #[test]
    fn bundle_detection_requires_apples_exact_casing() {
        for exe in [
            "/Applications/MechvibesDX.app/contents/macos/mechvibes-dx",
            "/Applications/MechvibesDX.app/Contents/macos/mechvibes-dx",
        ] {
            assert_eq!(macos_bundle_resource_root(Path::new(exe)), None, "failed for {}", exe);
        }
    }
}
