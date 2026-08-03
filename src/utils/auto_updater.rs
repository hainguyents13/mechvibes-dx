use crate::state::config::AppConfig;
use crate::utils::constants::APP_NAME;
use crate::utils::update_installer::{
    self,
    StageError,
    StagedUpdate,
};
use chrono::{ DateTime, Utc };
use reqwest;
use semver::Version;
use serde::{ Deserialize, Serialize };
use std::error::Error;
use std::fmt;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{ interval, Duration as TokioDuration };

// Fixed repository information
const REPO_OWNER: &str = "hainguyents13";
const REPO_NAME: &str = "mechvibes-dx";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub update_available: bool,
    pub download_url: Option<String>,
    pub release_notes: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub is_prerelease: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub name: String,
    pub body: String,
    pub published_at: String,
    pub prerelease: bool,
    pub assets: Vec<GitHubAsset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubAsset {
    pub name: String,
    pub browser_download_url: String,
    pub content_type: String,
    pub size: u64,
}

#[derive(Debug)]
pub enum UpdateError {
    NetworkError(String),
    ParseError(String),
    NotFound,
    InvalidVersion(String),
}

impl fmt::Display for UpdateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UpdateError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            UpdateError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            UpdateError::NotFound => write!(f, "No releases found"),
            UpdateError::InvalidVersion(msg) => write!(f, "Invalid version: {}", msg),
        }
    }
}

impl Error for UpdateError {}

pub struct AutoUpdater {
    pub current_version: String,
}

impl AutoUpdater {
    pub fn new() -> Self {
        Self {
            current_version: crate::utils::constants::APP_VERSION.to_string(),
        }
    }
    pub async fn check_for_updates(&self) -> Result<UpdateInfo, UpdateError> {
        let url = format!("https://api.github.com/repos/{}/{}/releases", REPO_OWNER, REPO_NAME);

        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .header("User-Agent", format!("{}/{}", APP_NAME, self.current_version))
            .send().await
            .map_err(|e| UpdateError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(UpdateError::NetworkError(format!("HTTP {}", response.status())));
        }

        let releases: Vec<GitHubRelease> = response
            .json().await
            .map_err(|e| UpdateError::ParseError(e.to_string()))?;

        if releases.is_empty() {
            return Err(UpdateError::NotFound);
        }

        // Find the latest release (excluding prereleases)
        let latest_release = releases
            .iter()
            .find(|release| !release.prerelease)
            .ok_or(UpdateError::NotFound)?;

        println!(
            "Latest release: {} ({}), published at {}",
            latest_release.tag_name,
            latest_release.name,
            latest_release.published_at
        );

        let current_version = Version::parse(&self.current_version).map_err(|e|
            UpdateError::InvalidVersion(e.to_string())
        )?;

        let latest_version_str = latest_release.tag_name.trim_start_matches('v');
        let latest_version = Version::parse(latest_version_str).map_err(|e|
            UpdateError::InvalidVersion(e.to_string())
        )?;

        let update_available = latest_version > current_version;

        // Find appropriate download URL for current platform
        let download_url = self.find_download_url(&latest_release.assets);

        let published_at = DateTime::parse_from_rfc3339(&latest_release.published_at)
            .map(|dt| dt.with_timezone(&Utc))
            .ok();

        Ok(UpdateInfo {
            current_version: self.current_version.clone(),
            latest_version: latest_version.to_string(),
            update_available,
            download_url,
            release_notes: Some(latest_release.body.clone()),
            published_at,
            is_prerelease: latest_release.prerelease,
        })
    }

    /// Picks the release asset this platform can actually install.
    ///
    /// Returns `None` rather than guessing when nothing matches. The previous
    /// implementation fell back to `assets[0]`, which on Linux and macOS meant
    /// offering the *Windows* `.exe` installer, since the Windows asset is
    /// alphabetically first in every release. A missing button is correct;
    /// a button that downloads an unusable file is not.
    fn find_download_url(&self, assets: &[GitHubAsset]) -> Option<String> {
        #[cfg(target_os = "windows")]
        let matches = |name: &str| {
            // Must contain "x64" and end in ".exe" - the same contract the
            // release pipeline asserts on (see release.yml).
            name.ends_with(".exe") && name.contains("x64")
        };

        // Linux and macOS have no unattended in-place upgrade path yet, so
        // these arms exist only to keep the "what's new" UI honest; the UI
        // links to the Releases page instead of downloading anything.
        #[cfg(target_os = "macos")]
        let matches = |name: &str| {
            name.ends_with(".dmg") || name.ends_with(".pkg") || name.contains("macos")
        };

        #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
        let matches = |name: &str| {
            name.ends_with(".deb") || name.ends_with(".appimage")
        };

        assets
            .iter()
            .find(|asset| matches(&asset.name.to_lowercase()))
            .map(|asset| asset.browser_download_url.clone())
    }

    /// Link to the release page, used wherever a direct download is not
    /// something this platform can act on.
    pub fn releases_page_url(version: &str) -> String {
        format!("https://github.com/{}/{}/releases/tag/v{}", REPO_OWNER, REPO_NAME, version)
    }

    /// User-Agent GitHub expects on both API and asset requests.
    pub fn user_agent(&self) -> String {
        format!("{}/{}", APP_NAME, self.current_version)
    }
}

// Configuration for auto-update settings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AutoUpdateConfig {
    pub last_check: Option<u64>,
    pub available_version: Option<String>,
    pub available_download_url: Option<String>,
    /// An installer already downloaded and verified, kept so that choosing
    /// "Later" survives a restart. `serde(default)` so configs written by
    /// versions before this field deserialize instead of being reset.
    #[serde(default)]
    pub staged_update: Option<StagedUpdate>,
}

/*
 * AUTO-UPDATE STRATEGY:
 *
 * 1. STARTUP CHECK (check_for_updates_on_startup):
 *    - Runs when app starts from completely closed state
 *    - Only checks if last_check was more than 1 hour ago (to avoid spam on frequent restarts)
 *    - Updates config.json with results
 *    - Updates global UI state for immediate titlebar notification
 *
 * 2. BACKGROUND SERVICE (UpdateService.start):
 *    - Runs periodic checks every 24 hours while app is running
 *    - Independent of startup checks
 *    - Continues normal background operation
 *
 * 3. MANUAL CHECK (in settings page):
 *    - User can trigger immediate check via "Check for Updates" button
 *    - Always performs check regardless of timing
 *
 * This ensures users always get fresh update info when they launch the app,
 * while avoiding excessive API calls on frequent app restarts.
 */

// Even simpler function without parameters
pub async fn check_for_updates_simple() -> Result<UpdateInfo, UpdateError> {
    let updater = AutoUpdater::new();
    updater.check_for_updates().await
}

// Service for auto-update background checking
pub struct UpdateService {
    config: Arc<Mutex<AppConfig>>,
}

impl UpdateService {
    pub fn new(config: Arc<Mutex<AppConfig>>) -> Self {
        Self {
            config,
        }
    }

    pub async fn start(&self) {
        let config = self.config.clone();

        tokio::spawn(async move {
            let mut interval = interval(TokioDuration::from_secs(86400)); // Check every 24 hours

            loop {
                interval.tick().await;
                let update_config = {
                    let config_guard = config.lock().await;
                    config_guard.auto_update.clone()
                }; // Check if it's time to check for updates (every 24 hours)
                if let Some(last_check) = update_config.last_check {
                    let now = std::time::SystemTime
                        ::now()
                        .duration_since(std::time::SystemTime::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    let check_interval = 24 * 3600; // 24 hours in seconds
                    if now.saturating_sub(last_check) < check_interval {
                        continue;
                    }
                }

                println!("🔄 Checking for updates...");

                match check_for_updates_simple().await {
                    Ok(update_info) => {
                        // Update last check time and save available update info
                        {
                            let mut config_guard = config.lock().await;
                            config_guard.auto_update.last_check = Some(
                                std::time::SystemTime
                                    ::now()
                                    .duration_since(std::time::SystemTime::UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_secs()
                            );
                            if update_info.update_available {
                                // Save update info to config
                                config_guard.auto_update.available_version = Some(
                                    update_info.latest_version.clone()
                                );
                                config_guard.auto_update.available_download_url =
                                    update_info.download_url.clone();
                            } else {
                                // Clear update info if no updates
                                config_guard.auto_update.available_version = None;
                                config_guard.auto_update.available_download_url = None;
                            }

                            let _ = config_guard.save();
                        }
                        if update_info.update_available {
                            println!(
                                "🆕 Update available: {} -> {}",
                                update_info.current_version,
                                update_info.latest_version
                            );
                            // Notify only. Downloading is a deliberate user
                            // action (the "Download & install" button in
                            // Settings), never something a background check
                            // starts on its own.
                            crate::state::app::set_update_info(Some(update_info));
                        } else {
                            println!("✅ No updates available");
                            // Clear update info if no updates
                            crate::state::app::set_update_info(None);
                        }
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to check for updates: {}", e);
                    }
                }
            }
        });
    }

    // pub async fn check_now(&self) -> Result<UpdateInfo, Box<dyn std::error::Error>> {
    //     let update_info = check_for_updates_simple().await?; // Update last check time
    //     {
    //         let mut config_guard = self.config.lock().await;
    //         config_guard.auto_update.last_check = Some(
    //             std::time::SystemTime
    //                 ::now()
    //                 .duration_since(std::time::SystemTime::UNIX_EPOCH)
    //                 .unwrap_or_default()
    //                 .as_secs()
    //         );
    //         let _ = config_guard.save();
    //     }

    //     Ok(update_info)
    // }

    // pub async fn download_and_install_update(
    //     &self,
    //     update_info: &UpdateInfo
    // ) -> Result<(), Box<dyn std::error::Error>> {
    //     if let Some(download_url) = &update_info.download_url {
    //         println!("📥 Downloading update...");

    //         let temp_dir = std::env::temp_dir();
    //         let default_filename = format!("mechvibes_dx_v{}.exe", update_info.latest_version);
    //         let filename = download_url.split('/').last().unwrap_or(&default_filename);
    //         let installer_path = temp_dir.join(filename);

    //         let updater = AutoUpdater::new();
    //         updater.download_update(download_url, &installer_path).await?;

    //         println!("🔧 Installing update...");
    //         updater.install_update(&installer_path)?;

    //         println!("✅ Update installed successfully. Please restart the application.");
    //         Ok(())
    //     } else {    //         Err("No download URL available".into())
    //     }
    // }
}

/// Where the update flow currently stands, for the UI to render.
///
/// Only ever moves forward within a session: `Idle -> Downloading -> Ready`,
/// with `Failed` as the terminal state that puts the UI back on the plain
/// "open the browser" button that existed before staging did. `Ready` can also
/// be entered directly at startup when a previous session staged an installer
/// and the user chose "Later".
#[derive(Debug, Clone, PartialEq, Default)]
pub enum UpdateStage {
    /// Nothing found, or nothing to do on this platform.
    #[default]
    Idle,
    /// Installer transfer in flight.
    Downloading {
        version: String,
    },
    /// Verified installer on disk, waiting for the user to pick Restart/Later.
    Ready {
        version: String,
        installer_path: String,
    },
    /// Staging gave up. `reason` is shown as a hint; the UI falls back to
    /// linking the Releases page, which always works.
    Failed {
        version: String,
        reason: String,
    },
}

static UPDATE_STAGE: once_cell::sync::Lazy<std::sync::Mutex<UpdateStage>> =
    once_cell::sync::Lazy::new(|| std::sync::Mutex::new(UpdateStage::Idle));

/// Current stage, cloned so callers never hold the lock.
pub fn get_update_stage() -> UpdateStage {
    UPDATE_STAGE.lock()
        .map(|guard| guard.clone())
        .unwrap_or(UpdateStage::Idle)
}

fn set_update_stage(stage: UpdateStage) {
    if let Ok(mut guard) = UPDATE_STAGE.lock() {
        *guard = stage;
    }
}

/// Downloads and verifies the installer for `update_info`.
///
/// Only ever called from the "Download & install" button - nothing in the
/// check paths starts a transfer, because a ~50 MB download is the user's
/// decision to make, not a side effect of a background version check.
///
/// Idempotent: a no-op on platforms without a silent installer, when a
/// download is already in flight, or when this version is already staged and
/// still verifies. Never returns an error - failures land in
/// [`UpdateStage::Failed`] and the UI falls back to the browser link.
pub async fn download_and_stage_update(update_info: &UpdateInfo) {
    if !update_installer::silent_update_supported() {
        return;
    }

    if !update_info.update_available {
        return;
    }

    let version = update_info.latest_version.clone();

    // Don't stack a second download on top of one already in flight, and
    // don't re-download something already verified and waiting.
    match get_update_stage() {
        UpdateStage::Downloading { .. } => {
            return;
        }
        UpdateStage::Ready { version: ref ready, .. } if *ready == version => {
            return;
        }
        _ => {}
    }

    let Some(download_url) = update_info.download_url.clone() else {
        set_update_stage(UpdateStage::Failed {
            version,
            reason: "No installer asset in this release".to_string(),
        });
        return;
    };

    // A staged installer from a previous session that still verifies is
    // reused as-is, so choosing "Later" and coming back never costs a second
    // download.
    let config = AppConfig::load();
    if let Some(staged) = &config.auto_update.staged_update {
        if let Ok(path) = staged.validate(&version) {
            set_update_stage(UpdateStage::Ready {
                version,
                installer_path: path.to_string_lossy().into_owned(),
            });
            return;
        }
        // Stale or corrupt - drop it before downloading a replacement.
        update_installer::discard_staged(staged);
    }

    set_update_stage(UpdateStage::Downloading { version: version.clone() });
    println!("📥 Downloading update {} (user requested)...", version);

    let user_agent = AutoUpdater::new().user_agent();
    match update_installer::download_and_verify(&download_url, &version, &user_agent).await {
        Ok(staged) => {
            let installer_path = staged.installer_path.clone();
            let mut config = AppConfig::load();
            config.auto_update.staged_update = Some(staged);
            if let Err(e) = config.save() {
                eprintln!("⚠️  Staged update downloaded but not recorded in config: {}", e);
            }
            println!("✅ Update {} verified and ready to install", version);
            set_update_stage(UpdateStage::Ready { version, installer_path });
        }
        Err(e) => {
            // Expected for any release published without SHA256SUMS.txt, and
            // for every network hiccup. Log and let the browser link stand in.
            let reason = match &e {
                StageError::ChecksumUnavailable(_) =>
                    "This release has no published checksums; use the download page.".to_string(),
                StageError::ChecksumMismatch { .. } =>
                    "The downloaded installer failed verification and was discarded.".to_string(),
                other => other.to_string(),
            };
            eprintln!("⚠️  Update download failed ({}); falling back to manual download", e);
            set_update_stage(UpdateStage::Failed { version, reason });
        }
    }
}

/// Whether `candidate` is strictly newer than `current`.
///
/// The guard against installing a staged file after the app was upgraded some
/// other way (a manual download, a reinstall) - running an older installer
/// then would be a silent downgrade. Unparseable versions answer `false`:
/// when in doubt, do not run an installer.
pub fn is_upgrade(current: &str, candidate: &str) -> bool {
    match (Version::parse(current), Version::parse(candidate)) {
        (Ok(current), Ok(candidate)) => candidate > current,
        _ => false,
    }
}

/// Re-checks a staged installer at startup and, if it is still valid for an
/// update we still want, puts the UI straight into `Ready`.
///
/// This is the other half of "Later": the file survives, the prompt returns
/// on the next launch. The hash is verified again rather than trusted from
/// config, so a file altered between sessions is discarded, not executed.
pub fn restore_staged_update() {
    if !update_installer::silent_update_supported() {
        return;
    }

    let mut config = AppConfig::load();
    let Some(staged) = config.auto_update.staged_update.clone() else {
        return;
    };

    let current = crate::utils::constants::APP_VERSION;

    if !is_upgrade(current, &staged.version) {
        println!("🧹 Discarding staged update {} (no longer newer than {})", staged.version, current);
        update_installer::discard_staged(&staged);
        config.auto_update.staged_update = None;
        let _ = config.save();
        return;
    }

    match staged.validate(&staged.version) {
        Ok(path) => {
            println!("📦 Staged update {} is ready from a previous session", staged.version);
            set_update_stage(UpdateStage::Ready {
                version: staged.version.clone(),
                installer_path: path.to_string_lossy().into_owned(),
            });
        }
        Err(e) => {
            eprintln!("⚠️  Discarding unusable staged update: {}", e);
            update_installer::discard_staged(&staged);
            config.auto_update.staged_update = None;
            let _ = config.save();
        }
    }
}

/// Launches the staged installer and hands control to it.
///
/// Order matters and is the whole risk of this feature:
/// 1. Re-verify the file (never execute something unverified).
/// 2. Clear it from config, so a crashed install cannot loop on the same file.
/// 3. Spawn the installer detached, so our exit does not kill it.
///
/// The caller must then close the app. The input worker child process needs no
/// separate signal: it exits on stdin EOF as soon as our handles close (see
/// `input_worker.rs`), so no orphan survives. Inno's `CloseApplications=yes`
/// closes anything still holding the binary, and the silent-only `[Run]` entry
/// in the `.iss` starts the app again once the files are in place - that entry
/// is the single relaunch path, verified against a real /LOG trace.
///
/// Returns an error without touching the app's lifecycle if anything fails, so
/// a blocked or missing installer leaves the user exactly where they were.
pub fn install_staged_update() -> Result<(), StageError> {
    let stage = get_update_stage();
    let UpdateStage::Ready { version, installer_path } = stage else {
        return Err(StageError::Io("No verified update is ready to install".to_string()));
    };

    let mut config = AppConfig::load();
    let staged = config.auto_update.staged_update.clone().ok_or_else(||
        StageError::Io("Staged update is missing from config".to_string())
    )?;

    // Verify one last time, immediately before execution.
    let path = staged.validate(&version)?;
    if path.to_string_lossy() != installer_path {
        return Err(StageError::Io("Staged installer path changed unexpectedly".to_string()));
    }

    // Cleared before spawning: if the install dies halfway, the next launch
    // re-checks and re-downloads rather than retrying a file that just failed.
    config.auto_update.staged_update = None;
    let _ = config.save();

    update_installer::spawn_installer_detached(&path)?;
    println!("🚀 Installer for {} launched; shutting down for upgrade", version);
    Ok(())
}


/// The saved update from config, but **only** if it is genuinely newer than
/// the running build.
///
/// This is the single read path for `auto_update.available_version`, and it
/// must stay that way. The config file on disk is untrusted input: version
/// 0.6.1 records "0.6.2 is available", the user upgrades to 0.6.2 by any
/// route that does not go through a check (a manual download, the installer,
/// a reinstall), and that stale line now describes an update to the very
/// version already running. The write paths do clear it, but they only run
/// when a check actually reaches the network - and the startup check is
/// rate-limited to once an hour, so there is a window where nothing clears
/// anything. Filtering on read closes that window regardless of what the
/// file says.
///
/// Comparison is semver, never string equality: "0.10.0" > "0.9.0" is only
/// true numerically.
pub fn get_saved_update_info() -> Option<UpdateInfo> {
    let config = crate::state::config::AppConfig::load();
    let available_version = config.auto_update.available_version.as_ref()?;
    let current_version = crate::utils::constants::APP_VERSION;

    if !is_upgrade(current_version, available_version) {
        return None;
    }

    Some(UpdateInfo {
        current_version: current_version.to_string(),
        latest_version: available_version.clone(),
        update_available: true,
        download_url: config.auto_update.available_download_url.clone(),
        release_notes: Some(AutoUpdater::releases_page_url(available_version)),
        published_at: None, // Not saved in config
        is_prerelease: false, // Not saved in config
    })
}

/// Drops a saved update that is no longer newer than the running build.
///
/// The companion to the read-side filter: without this, a stale entry stays
/// in `config.json` forever, invisibly, and any future code that reads the
/// field directly would resurrect the bug. Called at startup, so upgrading
/// by any means cleans the record up once rather than on every read.
///
/// Returns whether anything was cleared, which the tests assert on.
pub fn clear_stale_available_version() -> bool {
    let mut config = AppConfig::load();
    let Some(available_version) = config.auto_update.available_version.clone() else {
        return false;
    };

    if is_upgrade(crate::utils::constants::APP_VERSION, &available_version) {
        return false;
    }

    println!(
        "🧹 Clearing stale update record: v{} is not newer than the running v{}",
        available_version,
        crate::utils::constants::APP_VERSION
    );
    config.auto_update.available_version = None;
    config.auto_update.available_download_url = None;
    let _ = config.save();
    true
}

// Check for updates on app startup (when app was completely closed)
pub async fn check_for_updates_on_startup() -> Result<UpdateInfo, UpdateError> {
    println!("🔄 Checking for updates on startup...");

    // Only the timestamp is kept across the request: holding the whole struct
    // would invite saving it back after the await and reverting anything the
    // user changed meanwhile.
    let last_check = crate::state::config::AppConfig::load().auto_update.last_check;
    let now = std::time::SystemTime
        ::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Check if we should perform startup check
    // Only check if:
    // 1. Never checked before (last_check is None), OR
    // 2. Last check was more than 1 hour ago (to avoid spam on frequent restarts)
    let should_check = match last_check {
        None => {
            println!("📅 First time checking for updates");
            true
        }
        Some(last_check) => {
            let time_since_last_check = now.saturating_sub(last_check);
            let one_hour = 3600; // 1 hour in seconds
            if time_since_last_check >= one_hour {
                println!(
                    "📅 Last check was {} hours ago, checking again",
                    time_since_last_check / 3600
                );
                true
            } else {
                println!(
                    "📅 Recently checked ({} minutes ago), skipping startup check",
                    time_since_last_check / 60
                );
                false
            }
        }
    };

    if !should_check {
        // Return cached info if available
        if let Some(saved_update) = get_saved_update_info() {
            return Ok(saved_update);
        } else {
            // Create a "no update" response
            return Ok(UpdateInfo {
                current_version: crate::utils::constants::APP_VERSION.to_string(),
                latest_version: crate::utils::constants::APP_VERSION.to_string(),
                update_available: false,
                download_url: None,
                release_notes: None,
                published_at: None,
                is_prerelease: false,
            });
        }
    }

    // Perform actual check
    match check_for_updates_simple().await {
        Ok(update_info) => {
            // Re-read the config instead of reusing the copy taken before the
            // request: `save()` rewrites the whole struct, and the user can
            // change the volume, theme or mute state while the network round
            // trip is in flight. Saving the pre-request copy silently reverted
            // every one of those edits.
            let mut config = crate::state::config::AppConfig::load();

            // Update last check time and save info
            config.auto_update.last_check = Some(now);

            if update_info.update_available {
                config.auto_update.available_version = Some(update_info.latest_version.clone());
                config.auto_update.available_download_url = update_info.download_url.clone();
                println!(
                    "🆕 Startup check: Update available {} -> {}",
                    update_info.current_version,
                    update_info.latest_version
                );
            } else {
                config.auto_update.available_version = None;
                config.auto_update.available_download_url = None;
                println!("✅ Startup check: No updates available");
            }

            config.last_updated = chrono::Utc::now();
            let _ = config.save();

            // Update global state. Notification only - see the note in
            // UpdateService::start: nothing downloads without a user click.
            if update_info.update_available {
                crate::state::app::set_update_info(Some(update_info.clone()));
            } else {
                crate::state::app::set_update_info(None);
            }

            Ok(update_info)
        }
        Err(e) => {
            eprintln!("❌ Startup update check failed: {}", e);
            // Return cached info if check failed
            if let Some(saved_update) = get_saved_update_info() {
                Ok(saved_update)
            } else {
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newer_versions_are_upgrades() {
        assert!(is_upgrade("0.6.1", "0.6.2"));
        assert!(is_upgrade("0.6.1", "0.7.0"));
        assert!(is_upgrade("0.6.1", "1.0.0"));
    }

    #[test]
    fn the_same_version_is_not_an_upgrade() {
        assert!(!is_upgrade("0.6.1", "0.6.1"));
    }

    #[test]
    fn older_versions_are_never_upgrades() {
        // The downgrade guard: a staged 0.6.0 must not be installed over an
        // app that is already 0.6.1, however it got there.
        assert!(!is_upgrade("0.6.1", "0.6.0"));
        assert!(!is_upgrade("1.0.0", "0.9.9"));
    }

    #[test]
    fn prereleases_rank_below_their_release() {
        // semver ordering, asserted so a future switch to a different compare
        // cannot quietly start offering 0.7.0-alpha to a 0.7.0 user.
        assert!(!is_upgrade("0.7.0", "0.7.0-alpha.1"));
        assert!(is_upgrade("0.7.0-alpha.1", "0.7.0"));
    }

    #[test]
    fn unparseable_versions_never_trigger_an_install() {
        assert!(!is_upgrade("0.6.1", "not-a-version"));
        assert!(!is_upgrade("garbage", "0.7.0"));
        assert!(!is_upgrade("", ""));
    }

    #[test]
    fn releases_page_url_points_at_the_tag() {
        assert_eq!(
            AutoUpdater::releases_page_url("0.7.0"),
            "https://github.com/hainguyents13/mechvibes-dx/releases/tag/v0.7.0"
        );
    }

    fn asset(name: &str) -> GitHubAsset {
        GitHubAsset {
            name: name.to_string(),
            browser_download_url: format!("https://example.com/{}", name),
            content_type: "application/octet-stream".to_string(),
            size: 1,
        }
    }

    /// The full asset list of a real release, in the order the GitHub API
    /// returns it - the Windows installer first, which is exactly why the old
    /// `assets[0]` fallback served .exe files to Linux and macOS.
    fn release_assets() -> Vec<GitHubAsset> {
        vec![
            asset("MechvibesDX-0.7.0-Setup-x64.exe"),
            asset("SHA256SUMS.txt"),
            asset("mechvibes-dx-0.7.0-macos-arm64-experimental.tar.gz"),
            asset("mechvibes-dx_0.7.0_amd64.deb")
        ]
    }

    #[test]
    fn picks_the_asset_this_platform_can_actually_install() {
        let url = AutoUpdater::new().find_download_url(&release_assets());

        #[cfg(target_os = "windows")]
        assert_eq!(url.as_deref(), Some("https://example.com/MechvibesDX-0.7.0-Setup-x64.exe"));

        #[cfg(target_os = "macos")]
        assert_eq!(
            url.as_deref(),
            Some("https://example.com/mechvibes-dx-0.7.0-macos-arm64-experimental.tar.gz")
        );

        #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
        assert_eq!(url.as_deref(), Some("https://example.com/mechvibes-dx_0.7.0_amd64.deb"));
    }

    /// The regression that mattered: no platform may ever be handed a
    /// Windows installer it cannot run. Asserted for every platform rather
    /// than only the one being compiled, via the filename that comes back.
    #[test]
    fn never_offers_a_windows_installer_to_a_non_windows_build() {
        let url = AutoUpdater::new().find_download_url(&release_assets()).unwrap_or_default();
        if !cfg!(target_os = "windows") {
            assert!(!url.ends_with(".exe"), "non-Windows build was offered {}", url);
        }
    }

    /// A release with no asset for this platform must yield None so the UI
    /// shows a link to the release page, rather than falling back to whatever
    /// asset happened to be listed first.
    #[test]
    fn returns_none_when_no_asset_suits_this_platform() {
        let assets = vec![asset("SHA256SUMS.txt"), asset("source-code.zip")];
        assert_eq!(AutoUpdater::new().find_download_url(&assets), None);
    }

    #[test]
    fn returns_none_for_a_release_with_no_assets_at_all() {
        assert_eq!(AutoUpdater::new().find_download_url(&[]), None);
    }

    /// Decides whether a saved `available_version` may be shown, given the
    /// running build. Extracted so the rule can be tested against arbitrary
    /// version pairs - `get_saved_update_info` itself is pinned to the real
    /// `APP_VERSION` and the on-disk config, neither of which a unit test
    /// should mutate.
    fn saved_update_is_showable(current: &str, saved: &str) -> bool {
        is_upgrade(current, saved)
    }

    /// The exact bug a user hit on 0.6.2: an older build had written
    /// "0.6.2 is available" into config, the user upgraded to 0.6.2 outside
    /// the update flow, and Settings then offered an update to the version
    /// already running.
    #[test]
    fn a_saved_update_equal_to_the_running_version_is_never_offered() {
        assert!(!saved_update_is_showable("0.6.2", "0.6.2"));
    }

    #[test]
    fn a_saved_update_older_than_the_running_version_is_never_offered() {
        // Reinstalling an older build must not make the app offer to
        // "update" backwards either.
        assert!(!saved_update_is_showable("0.6.2", "0.6.1"));
        assert!(!saved_update_is_showable("1.0.0", "0.9.9"));
    }

    #[test]
    fn a_saved_update_newer_than_the_running_version_is_still_offered() {
        // The fix must not break the case the feature exists for.
        assert!(saved_update_is_showable("0.6.2", "0.6.3"));
        assert!(saved_update_is_showable("0.6.2", "0.7.0"));
    }

    /// String comparison would rank "0.9.0" above "0.10.0"; semver does not.
    #[test]
    fn version_comparison_is_numeric_not_lexicographic() {
        assert!(saved_update_is_showable("0.9.0", "0.10.0"));
        assert!(!saved_update_is_showable("0.10.0", "0.9.0"));
    }

    /// A config written by a future build, or hand-edited, must not crash or
    /// be treated as an update.
    #[test]
    fn a_malformed_saved_version_is_never_offered() {
        assert!(!saved_update_is_showable("0.6.2", "not-a-version"));
        assert!(!saved_update_is_showable("0.6.2", ""));
    }

    /// The clearing rule used by `clear_stale_available_version`: an entry is
    /// dropped exactly when it is not an upgrade.
    #[test]
    fn the_clear_rule_drops_stale_entries_and_keeps_real_ones() {
        let should_clear = |current: &str, saved: &str| !is_upgrade(current, saved);

        // Stale: same version (the reported bug) and older versions.
        assert!(should_clear("0.6.2", "0.6.2"));
        assert!(should_clear("0.6.2", "0.6.1"));
        // Junk that could never be installed.
        assert!(should_clear("0.6.2", "garbage"));
        // A genuine pending update must survive the cleanup.
        assert!(!should_clear("0.6.2", "0.6.3"));
    }
}
