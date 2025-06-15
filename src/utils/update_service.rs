use crate::state::config::AppConfig;
use crate::utils::auto_updater::{ check_for_updates_simple, UpdateInfo };
use chrono::{ Duration, Utc };
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{ interval, Duration as TokioDuration };

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
                };

                // Check if it's time to check for updates (every 24 hours)
                if let Some(last_check) = update_config.last_check {
                    let check_interval = Duration::hours(24); // Always 24 hours
                    if Utc::now() - last_check < check_interval {
                        continue;
                    }
                }

                println!("🔄 Checking for updates...");

                match check_for_updates_simple().await {
                    Ok(update_info) => {
                        // Update last check time
                        {
                            let mut config_guard = config.lock().await;
                            config_guard.auto_update.last_check = Some(Utc::now());
                            let _ = config_guard.save();
                        }
                        if update_info.update_available {
                            println!(
                                "🆕 Update available: {} -> {}",
                                update_info.current_version,
                                update_info.latest_version
                            );
                        } else {
                            println!("✅ No updates available");
                        }
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to check for updates: {}", e);
                    }
                }
            }
        });
    }

    async fn download_update(
        download_url: &str,
        update_info: &UpdateInfo
    ) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
        let temp_dir = std::env::temp_dir();
        let default_filename = format!("mechvibes_dx_v{}.exe", update_info.latest_version);
        let filename = download_url.split('/').last().unwrap_or(&default_filename);
        let installer_path = temp_dir.join(filename);

        println!("📥 Downloading update to: {:?}", installer_path);

        let client = reqwest::Client::new();
        let response = client.get(download_url).send().await?;

        if !response.status().is_success() {
            return Err(format!("Download failed: HTTP {}", response.status()).into());
        }

        let content = response.bytes().await?;
        std::fs::write(&installer_path, content)?;

        Ok(installer_path)
    }

    async fn install_update(
        installer_path: &std::path::PathBuf
    ) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(windows)]
        {
            use std::process::Command;

            let extension = installer_path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("");

            match extension.to_lowercase().as_str() {
                "msi" => {
                    println!("🔧 Installing MSI package...");
                    Command::new("msiexec")
                        .args(&["/i", installer_path.to_str().unwrap(), "/quiet"])
                        .spawn()?;
                }
                "exe" => {
                    println!("🔧 Running installer...");
                    Command::new(installer_path)
                        .arg("/S") // Silent install
                        .spawn()?;
                }
                _ => {
                    return Err("Unsupported installer type".into());
                }
            }

            // Schedule app restart
            println!("🔄 Update will complete after restart");
        }

        #[cfg(not(windows))]
        {
            return Err("Auto-install not supported on this platform".into());
        }

        Ok(())
    }
    pub async fn check_now(&self) -> Result<UpdateInfo, Box<dyn std::error::Error>> {
        let update_info = check_for_updates_simple().await?;

        // Update last check time
        {
            let mut config_guard = self.config.lock().await;
            config_guard.auto_update.last_check = Some(Utc::now());
            let _ = config_guard.save();
        }

        Ok(update_info)
    }

    pub async fn download_and_install_update(
        &self,
        update_info: &UpdateInfo
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(download_url) = &update_info.download_url {
            println!("📥 Downloading update...");
            let installer_path = Self::download_update(download_url, update_info).await?;

            println!("🔧 Installing update...");
            Self::install_update(&installer_path).await?;

            println!("✅ Update installed successfully. Please restart the application.");
            Ok(())
        } else {
            Err("No download URL available".into())
        }
    }
}
