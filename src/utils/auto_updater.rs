use crate::state::config::AppConfig;
use crate::utils::constants::APP_NAME;
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

    fn find_download_url(&self, assets: &[GitHubAsset]) -> Option<String> {
        // Priority order for Windows
        let preferred_extensions = if cfg!(windows) {
            vec![".msi", ".exe", "-setup.exe"]
        } else if cfg!(target_os = "macos") {
            vec![".dmg", ".pkg"]
        } else {
            vec![".AppImage", ".deb", ".tar.gz"]
        };

        for ext in preferred_extensions {
            if
                let Some(asset) = assets
                    .iter()
                    .find(|asset| {
                        asset.name.to_lowercase().ends_with(ext) &&
                            asset.name.to_lowercase().contains("x64")
                    })
            {
                return Some(asset.browser_download_url.clone());
            }
        }

        // Fallback to first asset if no platform-specific found
        assets.first().map(|asset| asset.browser_download_url.clone())
    }

    // pub async fn download_update(
    //     &self,
    //     download_url: &str,
    //     destination: &PathBuf
    // ) -> Result<(), UpdateError> {
    //     let client = reqwest::Client::new();
    //     let response = client
    //         .get(download_url)
    //         .header("User-Agent", format!("{}/{}", APP_NAME, self.current_version))
    //         .send().await
    //         .map_err(|e| UpdateError::NetworkError(e.to_string()))?;

    //     if !response.status().is_success() {
    //         return Err(
    //             UpdateError::NetworkError(format!("Download failed: HTTP {}", response.status()))
    //         );
    //     }

    //     let content = response.bytes().await.map_err(|e| UpdateError::NetworkError(e.to_string()))?;

    //     std::fs::write(destination, content).map_err(|e| UpdateError::NetworkError(e.to_string()))?;

    //     Ok(())
    // }

    // pub fn install_update(&self, installer_path: &PathBuf) -> Result<(), UpdateError> {
    //     #[cfg(windows)]
    //     {
    //         use std::process::Command;

    //         let extension = installer_path
    //             .extension()
    //             .and_then(|s| s.to_str())
    //             .unwrap_or("");

    //         match extension.to_lowercase().as_str() {
    //             "msi" => {
    //                 // Install MSI package
    //                 Command::new("msiexec")
    //                     .args(&["/i", installer_path.to_str().unwrap(), "/quiet"])
    //                     .spawn()
    //                     .map_err(|e| UpdateError::NetworkError(e.to_string()))?;
    //             }
    //             "exe" => {
    //                 // Run executable installer
    //                 Command::new(installer_path)
    //                     .arg("/S") // Silent install (for NSIS)
    //                     .spawn()
    //                     .map_err(|e| UpdateError::NetworkError(e.to_string()))?;
    //             }
    //             _ => {
    //                 return Err(UpdateError::ParseError("Unsupported installer type".to_string()));
    //             }
    //         }
    //     }

    //     #[cfg(not(windows))]
    //     {
    //         return Err(
    //             UpdateError::ParseError("Auto-install not supported on this platform".to_string())
    //         );
    //     }

    //     Ok(())
    // }
}

// Configuration for auto-update settings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AutoUpdateConfig {
    pub last_check: Option<u64>,
    pub available_version: Option<String>,
    pub available_download_url: Option<String>,
}

impl Default for AutoUpdateConfig {
    fn default() -> Self {
        Self {
            last_check: None,
            available_version: None,
            available_download_url: None,
        }
    }
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
                            // Set global update state for UI notification (no UI trigger here)
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

// Check if there's a saved update available in config
pub fn get_saved_update_info() -> Option<UpdateInfo> {
    let config = crate::state::config::AppConfig::load();
    if let Some(available_version) = &config.auto_update.available_version {
        let current_version = crate::utils::constants::APP_VERSION;

        // Check if saved version is newer than current version
        if
            let (Ok(current), Ok(available)) = (
                Version::parse(current_version),
                Version::parse(available_version),
            )
        {
            if available > current {
                return Some(UpdateInfo {
                    current_version: current_version.to_string(),
                    latest_version: available_version.clone(),
                    update_available: true,
                    download_url: config.auto_update.available_download_url.clone(),
                    release_notes: Some(
                        format!(
                            "https://github.com/{}/{}/releases/tag/v{}",
                            REPO_OWNER,
                            REPO_NAME,
                            available_version
                        )
                    ),
                    published_at: None, // Not saved in config
                    is_prerelease: false, // Not saved in config
                });
            }
        }
    }

    None
}

// Check for updates on app startup (when app was completely closed)
pub async fn check_for_updates_on_startup() -> Result<UpdateInfo, UpdateError> {
    println!("🔄 Checking for updates on startup...");

    let mut config = crate::state::config::AppConfig::load();
    let now = std::time::SystemTime
        ::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Check if we should perform startup check
    // Only check if:
    // 1. Never checked before (last_check is None), OR
    // 2. Last check was more than 1 hour ago (to avoid spam on frequent restarts)
    let should_check = match config.auto_update.last_check {
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

            let _ = config.save();

            // Update global state
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
