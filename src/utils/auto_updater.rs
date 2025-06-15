use crate::utils::constants::APP_NAME;
use chrono::{ DateTime, Utc };
use reqwest;
use semver::Version;
use serde::{ Deserialize, Serialize };
use std::error::Error;
use std::fmt;
use std::path::PathBuf;

// Fixed repository information
const REPO_OWNER: &str = "hainguyents13";
const REPO_NAME: &str = "mechvibes-dx";

#[derive(Debug, Clone, Serialize, Deserialize)]
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
            current_version: env!("CARGO_PKG_VERSION").to_string(),
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

    pub async fn download_update(
        &self,
        download_url: &str,
        destination: &PathBuf
    ) -> Result<(), UpdateError> {
        let client = reqwest::Client::new();
        let response = client
            .get(download_url)
            .header("User-Agent", format!("{}/{}", APP_NAME, self.current_version))
            .send().await
            .map_err(|e| UpdateError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(
                UpdateError::NetworkError(format!("Download failed: HTTP {}", response.status()))
            );
        }

        let content = response.bytes().await.map_err(|e| UpdateError::NetworkError(e.to_string()))?;

        std::fs::write(destination, content).map_err(|e| UpdateError::NetworkError(e.to_string()))?;

        Ok(())
    }

    pub fn install_update(&self, installer_path: &PathBuf) -> Result<(), UpdateError> {
        #[cfg(windows)]
        {
            use std::process::Command;

            let extension = installer_path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("");

            match extension.to_lowercase().as_str() {
                "msi" => {
                    // Install MSI package
                    Command::new("msiexec")
                        .args(&["/i", installer_path.to_str().unwrap(), "/quiet"])
                        .spawn()
                        .map_err(|e| UpdateError::NetworkError(e.to_string()))?;
                }
                "exe" => {
                    // Run executable installer
                    Command::new(installer_path)
                        .arg("/S") // Silent install (for NSIS)
                        .spawn()
                        .map_err(|e| UpdateError::NetworkError(e.to_string()))?;
                }
                _ => {
                    return Err(UpdateError::ParseError("Unsupported installer type".to_string()));
                }
            }
        }

        #[cfg(not(windows))]
        {
            return Err(
                UpdateError::ParseError("Auto-install not supported on this platform".to_string())
            );
        }

        Ok(())
    }
}

// Configuration for auto-update settings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AutoUpdateConfig {
    pub last_check: Option<DateTime<Utc>>,
}

impl Default for AutoUpdateConfig {
    fn default() -> Self {
        Self {
            last_check: None,
        }
    }
}

// Even simpler function without parameters
pub async fn check_for_updates_simple() -> Result<UpdateInfo, UpdateError> {
    let updater = AutoUpdater::new();
    updater.check_for_updates().await
}
