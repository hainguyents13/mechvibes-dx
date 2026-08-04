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
///
/// ## Linux AppImage
///
/// An AppImage runs from a read-only SquashFS mounted at a fresh temporary
/// directory on every launch:
///
/// ```text
/// /tmp/.mount_MechviXXXXXX/usr/bin/mechvibes-dx        <- current_exe()
/// /tmp/.mount_MechviXXXXXX/usr/lib/mechvibes-dx/assets
/// /tmp/.mount_MechviXXXXXX/usr/share/mechvibes-dx/soundpacks
/// ```
///
/// The layout deliberately mirrors the `.deb` (`usr/bin`, `usr/share`), which
/// is what lets one binary serve both. But the paths must be resolved
/// *relative to the mount point*, never as absolute `/usr/share/...`: that
/// would read a co-installed `.deb`'s soundpacks off the host instead of the
/// ones inside the image. Writable state goes to the XDG data dir, exactly as
/// it already did for the `.deb`, because the mount is read-only and its path
/// changes on every launch.
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

/// Root of the mounted AppImage (the AppDir), if `exe_path` is inside one.
///
/// Detection is structural, matching the macOS approach and for the same
/// reason: the identical binary is also installed by the `.deb` at
/// `/usr/bin/mechvibes-dx`, and that layout must keep resolving `/usr/share`
/// absolutely. Only the exact `<root>/usr/bin/<exe>` shape counts, and only
/// when `<root>` is not the filesystem root - `/usr/bin/mechvibes-dx` from the
/// `.deb` would otherwise yield an AppDir of `/`.
///
/// `APPDIR` (exported by AppRun) is deliberately NOT consulted. It is
/// inherited by child processes and by anything launched from a terminal that
/// once sourced it, so trusting it would let a stale value redirect a normally
/// installed app. The executable's own location cannot be spoofed that way.
///
/// Pure path logic with no filesystem access, so it is testable on every
/// platform - including the Windows CI that actually runs these tests.
fn appimage_root(exe_path: &Path) -> Option<PathBuf> {
    let bin_dir = exe_path.parent()?;
    if bin_dir.file_name()? != "bin" {
        return None;
    }

    let usr_dir = bin_dir.parent()?;
    if usr_dir.file_name()? != "usr" {
        return None;
    }

    // The AppDir root itself. `file_name()` is None for `/` and for a bare
    // relative `usr`, which is exactly the case that must not be treated as an
    // AppImage: that is the .deb's `/usr/bin/mechvibes-dx`.
    let app_dir = usr_dir.parent()?;
    app_dir.file_name()?;

    Some(app_dir.to_path_buf())
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
                    crate::always_print!("📂 App root (dev mode - from cwd): {}", cwd.display());
                    return cwd;
                }

                // Inside a macOS .app the resources live in ../Resources, not
                // beside the executable in Contents/MacOS.
                if let Some(resources) = macos_bundle_resource_root(&exe_path) {
                    crate::always_print!("📂 App root (macOS bundle): {}", resources.display());
                    return resources;
                }

                crate::always_print!("📂 App root (from exe): {}", exe_dir.display());
                return exe_dir.to_path_buf();
            }
        }

        // Fallback to current working directory (for development)
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        crate::always_print!("📂 App root (fallback - from cwd): {}", cwd.display());
        cwd
    })
}

/// True when `exe_path` is a Linux system-wide install location, i.e. what the
/// `.deb` produces: `/usr/bin/mechvibes-dx` (or `/usr/local/bin/...`).
///
/// These directories are root-owned. The app root for such a binary is
/// `/usr/bin`, so `{app_root}/data` resolved to `/usr/bin/data` - a directory a
/// normal user cannot create, let alone write. `AppConfig::save()` fails soft,
/// so this did not error: it produced an app that silently discarded every
/// setting and reset on each launch.
///
/// Absolute paths only, so a *portable* build that merely happens to live in
/// some `bin/` directory under the user's home keeps its existing
/// resources-beside-the-exe behavior. An AppImage's `/tmp/.mount_X/usr/bin`
/// is likewise excluded here (it is absolute but not rooted at `/usr`), and is
/// handled by `appimage_root` instead.
///
/// Pure path logic with no filesystem access, so it is testable on every
/// platform - including the Windows CI that actually runs these tests.
fn is_linux_system_install(exe_path: &Path) -> bool {
    let Some(parent) = exe_path.parent() else {
        return false;
    };
    matches!(
        parent.to_str(),
        Some("/usr/bin") | Some("/usr/local/bin")
    )
}

/// The mounted AppDir for this process, if it is running from an AppImage.
fn running_from_appimage() -> Option<&'static PathBuf> {
    static APP_DIR: OnceLock<Option<PathBuf>> = OnceLock::new();
    APP_DIR
        .get_or_init(|| std::env::current_exe().ok().as_deref().and_then(appimage_root))
        .as_ref()
}

/// Whether writable state must live in the system app data dir rather than
/// beside the executable, given where the executable is.
///
/// Split out from [`get_writable_data_dir`] as pure path logic so the decision
/// is unit-testable on every platform, including Windows CI. All three cases it
/// answers `true` for share one property: the app root is not writable by the
/// user running the app.
///
/// - macOS `.app` - `/Applications` is admin-owned.
/// - Linux AppImage - the image is mounted read-only, and its mount point is a
///   fresh temporary directory on every launch, so anything written beside the
///   binary would be unreachable next time even if the write succeeded.
/// - Linux system install (`.deb`) - `/usr/bin` is root-owned, and
///   `{app_root}/data` for it is `/usr/bin/data`.
fn writable_data_belongs_in_system_dir(exe_path: &Path) -> bool {
    macos_bundle_resource_root(exe_path).is_some()
        || appimage_root(exe_path).is_some()
        || is_linux_system_install(exe_path)
}

/// Directory holding writable application data (`config.json`, `themes.json`,
/// the soundpack cache).
///
/// Normally `{app_root}/data`, which keeps the Windows portable/installed
/// layout, the dev-mode cwd layout, and macOS/Linux bare builds exactly as they
/// were. It moves to the system app data dir only where the app root is
/// genuinely not writable - see [`writable_data_belongs_in_system_dir`].
///
/// `AppConfig::save()` fails soft, so getting this wrong does not error - it
/// produces an app that silently discards every setting and resets on each
/// launch. Hence the explicit branches rather than a best-effort write.
fn get_writable_data_dir() -> &'static PathBuf {
    static DATA_DIR: OnceLock<PathBuf> = OnceLock::new();
    DATA_DIR.get_or_init(|| {
        let in_system_dir = std::env::current_exe()
            .ok()
            .as_deref()
            .is_some_and(writable_data_belongs_in_system_dir);

        if !in_system_dir {
            return get_app_root().join("data");
        }

        let system_dir = get_system_app_data_dir().join("data");
        migrate_legacy_data_dir(&get_app_root().join("data"), &system_dir);
        system_dir
    })
}

/// One-time courtesy copy of a config left at the old, pre-fix location.
///
/// Only ever relevant to someone who ran a `.deb` install as root (or otherwise
/// had a writable `/usr/bin/data`), which is the sole way a file could exist
/// there.
/// Everyone else's old location was never writable, so there is nothing to
/// move and this is a cheap `exists()` check that finds nothing.
///
/// Deliberately conservative:
/// - never overwrites - if the new location already has a config, that one is
///   authoritative and the old file is left untouched;
/// - copies rather than moves, so a failure cannot destroy the only copy, and
///   the old file stays readable if the user downgrades;
/// - fails soft on every error, since this is a nicety and must never stop the
///   app from starting.
fn migrate_legacy_data_dir(legacy_dir: &Path, system_dir: &Path) {
    let legacy_config = legacy_dir.join("config.json");
    if !legacy_config.is_file() {
        return;
    }
    if system_dir.join("config.json").exists() {
        return;
    }
    if std::fs::create_dir_all(system_dir).is_err() {
        return;
    }

    // config.json is the one that carries user settings. The cache and manifest
    // are regenerated on demand, and themes.json is copied only if present.
    for name in ["config.json", "themes.json"] {
        let from = legacy_dir.join(name);
        if from.is_file() {
            match std::fs::copy(&from, system_dir.join(name)) {
                Ok(_) => {
                    crate::always_print!(
                        "📦 Migrated {} from {} to {}",
                        name,
                        legacy_dir.display(),
                        system_dir.display()
                    );
                }
                Err(e) => {
                    crate::always_eprint!("⚠️  Could not migrate {}: {}", name, e);
                }
            }
        }
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
    use super::{get_app_root, get_system_app_data_dir, running_from_appimage};
    use std::path::{Path, PathBuf};
    use std::sync::OnceLock;

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

    /// Get the base soundpacks directory for built-in soundpacks.
    ///
    /// Resolved once per process: every input it depends on (the executable
    /// location, and on Linux the presence of the system data directory) is
    /// fixed for the run. Callers hit this on hot paths - once per soundpack
    /// image request and per metadata load - so re-resolving and re-logging on
    /// each one produced a burst of identical lines whose count tracked the
    /// number of soundpacks rather than anything meaningful.
    ///
    /// Checks multiple locations in order:
    ///
    /// 1. {appdir}/usr/share/mechvibes-dx/soundpacks (running from an AppImage)
    /// 2. /usr/share/mechvibes-dx/soundpacks (installed via DEB/system package)
    /// 3. {app_root}/soundpacks (portable/dev mode)
    ///
    /// The AppImage check must come first and must be **relative to the mount
    /// point**. The absolute `/usr/share` path below is a real directory on a
    /// machine that also has the `.deb` installed, so checking it first would
    /// make a running AppImage load the *other* installation's soundpacks -
    /// silently, and with whatever version those happen to be.
    pub fn get_builtin_soundpacks_dir() -> PathBuf {
        static BUILTIN_SOUNDPACKS_DIR: OnceLock<PathBuf> = OnceLock::new();
        BUILTIN_SOUNDPACKS_DIR.get_or_init(|| {
            // Inside a mounted AppImage: resolve against the AppDir, never `/`.
            if let Some(app_dir) = running_from_appimage() {
                let bundled = app_dir.join("usr/share/mechvibes-dx/soundpacks");
                if bundled.exists() {
                    crate::always_print!(
                        "📂 Using AppImage soundpacks directory: {}",
                        bundled.display()
                    );
                    return bundled;
                }
            }

            // Check standard Linux data directory first (for installed packages)
            #[cfg(target_os = "linux")]
            {
                let system_soundpacks = PathBuf::from("/usr/share/mechvibes-dx/soundpacks");
                if system_soundpacks.exists() {
                    crate::always_print!(
                        "📂 Using system soundpacks directory: {}",
                        system_soundpacks.display()
                    );
                    return system_soundpacks;
                }
            }

            // Fallback to app root (for portable/dev mode)
            let app_root_soundpacks = get_app_root().join("soundpacks");
            crate::always_print!(
                "📂 Using app root soundpacks directory: {}",
                app_root_soundpacks.display()
            );
            app_root_soundpacks
        }).clone()
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
    use super::{
        appimage_root,
        is_linux_system_install,
        macos_bundle_resource_root,
        writable_data_belongs_in_system_dir,
    };
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

    /// The shipping AppImage layout. The mount point is generated fresh by the
    /// runtime on every launch, so only its *shape* can be relied on.
    #[test]
    fn detects_the_mounted_appimage_layout() {
        let exe = Path::new("/tmp/.mount_MechviAbC123/usr/bin/mechvibes-dx");
        assert_eq!(
            appimage_root(exe),
            Some(PathBuf::from("/tmp/.mount_MechviAbC123"))
        );
    }

    /// The mount point is not fixed: the runtime picks a random suffix each
    /// launch, and `TMPDIR` can move the whole thing elsewhere.
    #[test]
    fn appimage_detection_is_independent_of_the_mount_point() {
        for (exe, expected) in [
            ("/tmp/.mount_Mechvi000001/usr/bin/mechvibes-dx", "/tmp/.mount_Mechvi000001"),
            ("/run/user/1000/.mount_MechviXY/usr/bin/mechvibes-dx", "/run/user/1000/.mount_MechviXY"),
            // --appimage-extract produces a plain directory, and running the
            // binary out of it must behave identically to the mounted image.
            ("/home/someone/squashfs-root/usr/bin/mechvibes-dx", "/home/someone/squashfs-root"),
        ] {
            assert_eq!(
                appimage_root(Path::new(exe)),
                Some(PathBuf::from(expected)),
                "failed for {}",
                exe
            );
        }
    }

    /// The single most important case: the `.deb` installs the *same binary* to
    /// `/usr/bin/mechvibes-dx`. Treating that as an AppImage would compute an
    /// AppDir of `/`, and then resolve soundpacks and writable data against the
    /// filesystem root. It must fall through to the existing absolute-path
    /// behavior instead.
    #[test]
    fn the_deb_install_location_is_never_treated_as_an_appimage() {
        assert_eq!(appimage_root(Path::new("/usr/bin/mechvibes-dx")), None);
        assert_eq!(appimage_root(Path::new("/usr/local/bin/mechvibes-dx")), None);
    }

    /// Everything that is NOT an AppImage must keep resolving resources beside
    /// the executable. Runs on every platform, so Windows CI guards it too.
    #[test]
    fn non_appimage_layouts_are_never_treated_as_appimages() {
        let non_appimages = [
            // Windows installed and portable layouts
            r"C:\Program Files\MechvibesDX\mechvibes-dx.exe",
            r"D:\mechvibes-dx\target\release\mechvibes-dx.exe",
            // Linux dev build and other install shapes
            "/home/someone/mechvibes-dx/target/release/mechvibes-dx",
            "/opt/mechvibes-dx/mechvibes-dx",
            // macOS bundle - must be claimed by the macOS branch, not this one
            "/Applications/MechvibesDX.app/Contents/MacOS/mechvibes-dx",
            // `bin` present but not under `usr`
            "/home/someone/.local/bin/mechvibes-dx",
            // `usr` present but the binary is not in `bin`
            "/tmp/.mount_MechviAbC123/usr/lib/mechvibes-dx",
            // Nothing to walk up to
            "mechvibes-dx",
            "bin/mechvibes-dx",
        ];

        for exe in non_appimages {
            assert_eq!(
                appimage_root(Path::new(exe)),
                None,
                "'{}' was wrongly detected as an AppImage",
                exe
            );
        }
    }

    /// The `.deb` install locations. `{app_root}/data` for these is
    /// `/usr/bin/data` and `/usr/local/bin/data`, neither of which a normal user can
    /// write - which is the bug this detector exists to fix.
    #[test]
    fn detects_the_deb_system_install_locations() {
        assert!(is_linux_system_install(Path::new("/usr/bin/mechvibes-dx")));
        assert!(is_linux_system_install(Path::new("/usr/local/bin/mechvibes-dx")));
    }

    /// Everything a user can actually write to must keep the old
    /// data-beside-the-exe behavior, byte for byte.
    #[test]
    fn writable_locations_are_never_treated_as_system_installs() {
        let user_writable = [
            // Windows installed and portable
            r"C:\Program Files\MechvibesDX\mechvibes-dx.exe",
            r"D:\mechvibes-dx\target\release\mechvibes-dx.exe",
            // Linux dev build and portable layouts
            "/home/someone/mechvibes-dx/target/release/mechvibes-dx",
            "/opt/mechvibes-dx/mechvibes-dx",
            // A portable build that merely happens to sit in a `bin/` dir:
            // relative, and under the user's home, so it stays portable.
            "/home/someone/apps/mechvibes/bin/mechvibes-dx",
            "bin/mechvibes-dx",
            // The AppImage mount is handled by appimage_root, not this.
            "/tmp/.mount_MechviAbC123/usr/bin/mechvibes-dx",
            // Near misses on the exact directory match.
            "/usr/bin/subdir/mechvibes-dx",
            "/usr/sbin/mechvibes-dx",
            "/usr/lib/mechvibes-dx",
        ];

        for exe in user_writable {
            assert!(
                !is_linux_system_install(Path::new(exe)),
                "'{}' was wrongly treated as a system install",
                exe
            );
        }
    }

    /// The combined decision that `get_writable_data_dir` actually branches on.
    /// This is the regression guard for Windows and dev mode: those must keep
    /// resolving `{app_root}/data` exactly as before the .deb fix.
    #[test]
    fn only_unwritable_app_roots_move_data_to_the_system_dir() {
        let moves_to_system_dir = [
            // macOS bundle
            "/Applications/MechvibesDX.app/Contents/MacOS/mechvibes-dx",
            // Linux AppImage
            "/tmp/.mount_MechviAbC123/usr/bin/mechvibes-dx",
            // Linux .deb
            "/usr/bin/mechvibes-dx",
            "/usr/local/bin/mechvibes-dx",
        ];
        for exe in moves_to_system_dir {
            assert!(
                writable_data_belongs_in_system_dir(Path::new(exe)),
                "'{}' must write its data to the system app data dir",
                exe
            );
        }

        let stays_beside_the_exe = [
            r"C:\Program Files\MechvibesDX\mechvibes-dx.exe",
            r"D:\mechvibes-dx\target\release\mechvibes-dx.exe",
            "/home/someone/mechvibes-dx/target/release/mechvibes-dx",
            "/opt/mechvibes-dx/mechvibes-dx",
            // Bare macOS dev build - same binary, no bundle around it.
            "/Users/someone/mechvibes-dx/target/release/mechvibes-dx",
        ];
        for exe in stays_beside_the_exe {
            assert!(
                !writable_data_belongs_in_system_dir(Path::new(exe)),
                "'{}' must keep writing data beside the executable",
                exe
            );
        }
    }

    /// A scratch directory unique to the calling test, cleaned up first so a
    /// previous run cannot leak into this one.
    fn scratch_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("mechvibes-paths-test-{}", name));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("failed to create scratch dir");
        dir
    }

    /// The case the migration exists for: someone ran the .deb as root, so a
    /// config really does sit at the old location, and the new location is
    /// empty.
    #[test]
    fn a_legacy_config_is_copied_to_the_system_dir() {
        let root = scratch_dir("migrates");
        let legacy = root.join("legacy");
        let system = root.join("system");
        std::fs::create_dir_all(&legacy).unwrap();
        std::fs::write(legacy.join("config.json"), r#"{"volume":0.42}"#).unwrap();
        std::fs::write(legacy.join("themes.json"), "[]").unwrap();

        super::migrate_legacy_data_dir(&legacy, &system);

        assert_eq!(
            std::fs::read_to_string(system.join("config.json")).unwrap(),
            r#"{"volume":0.42}"#,
            "the user's settings must survive the move"
        );
        assert_eq!(std::fs::read_to_string(system.join("themes.json")).unwrap(), "[]");
        assert!(
            legacy.join("config.json").exists(),
            "the original must be copied, not moved - a downgrade should still find it"
        );

        let _ = std::fs::remove_dir_all(&root);
    }

    /// An existing config at the new location is authoritative and must never
    /// be clobbered by a stale file left in /usr/bin/data.
    #[test]
    fn migration_never_overwrites_an_existing_config() {
        let root = scratch_dir("no-overwrite");
        let legacy = root.join("legacy");
        let system = root.join("system");
        std::fs::create_dir_all(&legacy).unwrap();
        std::fs::create_dir_all(&system).unwrap();
        std::fs::write(legacy.join("config.json"), r#"{"volume":0.11}"#).unwrap();
        std::fs::write(system.join("config.json"), r#"{"volume":0.99}"#).unwrap();

        super::migrate_legacy_data_dir(&legacy, &system);

        assert_eq!(
            std::fs::read_to_string(system.join("config.json")).unwrap(),
            r#"{"volume":0.99}"#,
            "the current config must win over the legacy one"
        );

        let _ = std::fs::remove_dir_all(&root);
    }

    /// The overwhelmingly common case: nothing was ever written to /usr/bin/data
    /// because it was never writable. Must be a silent no-op that creates
    /// nothing.
    #[test]
    fn migration_does_nothing_when_there_is_no_legacy_config() {
        let root = scratch_dir("nothing-to-do");
        let legacy = root.join("legacy");
        let system = root.join("system");

        super::migrate_legacy_data_dir(&legacy, &system);

        assert!(
            !system.exists(),
            "no legacy config means the system dir must not be created here"
        );

        let _ = std::fs::remove_dir_all(&root);
    }

    /// The two detectors describe disjoint layouts. A path can never be both,
    /// which is what lets `get_writable_data_dir` check them with an `||`
    /// without having to decide precedence.
    #[test]
    fn the_macos_and_appimage_layouts_never_overlap() {
        for exe in [
            "/Applications/MechvibesDX.app/Contents/MacOS/mechvibes-dx",
            "/tmp/.mount_MechviAbC123/usr/bin/mechvibes-dx",
            "/usr/bin/mechvibes-dx",
            r"C:\Program Files\MechvibesDX\mechvibes-dx.exe",
        ] {
            let path = Path::new(exe);
            assert!(
                macos_bundle_resource_root(path).is_none() || appimage_root(path).is_none(),
                "'{}' was detected as both a macOS bundle and an AppImage",
                exe
            );
        }
    }
}
