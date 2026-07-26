use tray_icon::{
    Icon,
    menu::{ Menu, MenuEvent, MenuItem, PredefinedMenuItem, MenuId, Submenu, CheckMenuItem },
    TrayIcon,
    TrayIconBuilder,
};
use crate::utils::constants::APP_NAME;
use once_cell::sync::Lazy;
use std::sync::Mutex;

// Embed the icon and online soundpacks at compile time
const EMBEDDED_ICON: &[u8] = include_bytes!("../../assets/icon.ico");
const ONLINE_SOUNDPACKS_JSON: &str = include_str!("../../assets/online_soundpacks.json");

// Global state for tracking which pack is currently downloading
static DOWNLOADING_PACK_ID: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));

pub fn set_downloading_pack(pack_id: Option<String>) {
    if let Ok(mut guard) = DOWNLOADING_PACK_ID.lock() {
        *guard = pack_id;
    }
}

#[derive(serde::Deserialize, Clone)]
struct OnlineSoundpack {
    id: String,
    name: String,
    download_url: String,
    #[serde(rename = "type")]
    pack_type: String,
}

pub enum TrayMessage {
    Show,
    Exit,
    ToggleMute,
    OpenGitHub,
    OpenDiscord,
    OpenWebsite,
    SetKeyboardSoundpack(String),
    SetMouseSoundpack(String),
    DownloadPacks,
    DownloadKeyboardPack { name: String, url: String },
    DownloadMousePack { name: String, url: String },
    Restart,
}

pub struct TrayManager {
    tray_icon: TrayIcon,
}

impl TrayManager {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Load the icon from embedded bytes for cross-platform reliability
        let icon = match image::load_from_memory_with_format(EMBEDDED_ICON, image::ImageFormat::Ico) {
            Ok(img) => {
                let rgba = img.to_rgba8();
                let (width, height) = rgba.dimensions();
                match Icon::from_rgba(rgba.into_raw(), width, height) {
                    Ok(icon) => {
                        println!("✅ Loaded embedded tray icon ({}x{})", width, height);
                        icon
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to create tray icon from embedded data: {}", e);
                        return Err(Box::new(e));
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ Failed to load embedded tray icon data: {}", e);
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Failed to load embedded icon: {}", e)
                )));
            }
        };

        // Build the tray icon
        let tray_icon = TrayIconBuilder::new()
            .with_tooltip(APP_NAME)
            .with_icon(icon)
            .build()?;

        let mut manager = TrayManager {
            tray_icon: tray_icon,
        };

        // Initialize menu
        manager.update_menu()?;

        Ok(manager)
    }

    pub fn update_menu(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Load current config to determine sound state
        let config = crate::state::config::AppConfig::load();
        let mute_text = if config.enable_sound { "Mute sounds" } else { "Unmute sounds" };

        // Create new menu with updated text
        let show_item = MenuItem::with_id(
            MenuId::new("show"),
            &format!("Show {}", APP_NAME),
            true,
            None
        );
        let separator1 = PredefinedMenuItem::separator();

        // Sound control section with updated text
        let mute_item = MenuItem::with_id(MenuId::new("toggle_mute"), mute_text, true, None);
        let separator2 = PredefinedMenuItem::separator();

        // Local Soundpacks section
        let cache = crate::state::soundpack::SoundpackCache::load();
        let mut keyboard_packs = Vec::new();
        let mut mouse_packs = Vec::new();

        for pack in cache.soundpacks.values() {
            match pack.soundpack_type {
                crate::state::soundpack::SoundpackType::Keyboard => {
                    keyboard_packs.push(pack);
                }
                crate::state::soundpack::SoundpackType::Mouse => {
                    mouse_packs.push(pack);
                }
            }
        }

        keyboard_packs.sort_by(|a, b| a.name.cmp(&b.name));
        mouse_packs.sort_by(|a, b| a.name.cmp(&b.name));

        // Build Keyboard Submenu
        let mut kb_items_owned = Vec::new();
        for pack in &keyboard_packs {
            let is_checked = config.keyboard_soundpack == pack.id;
            let item = CheckMenuItem::with_id(
                MenuId::new(format!("kb_pack:{}", pack.id)),
                &pack.name,
                true,
                is_checked,
                None,
            );
            kb_items_owned.push(item);
        }
        let kb_refs: Vec<&dyn tray_icon::menu::IsMenuItem> = kb_items_owned
            .iter()
            .map(|item| item as &dyn tray_icon::menu::IsMenuItem)
            .collect();
        let kb_submenu = Submenu::with_items("Select Keyboard Sound Pack", true, &kb_refs)?;

        // Build Mouse Submenu
        let mut ms_items_owned = Vec::new();
        for pack in &mouse_packs {
            let is_checked = config.mouse_soundpack == pack.id;
            let item = CheckMenuItem::with_id(
                MenuId::new(format!("ms_pack:{}", pack.id)),
                &pack.name,
                true,
                is_checked,
                None,
            );
            ms_items_owned.push(item);
        }
        let ms_refs: Vec<&dyn tray_icon::menu::IsMenuItem> = ms_items_owned
            .iter()
            .map(|item| item as &dyn tray_icon::menu::IsMenuItem)
            .collect();
        let ms_submenu = Submenu::with_items("Select Mouse Sound Pack", true, &ms_refs)?;

        // Parse online soundpacks
        let online_packs: Vec<OnlineSoundpack> = serde_json::from_str(ONLINE_SOUNDPACKS_JSON).unwrap_or_default();
        let mut online_kb_packs = Vec::new();
        let mut online_ms_packs = Vec::new();
        for (idx, pack) in online_packs.iter().enumerate() {
            if pack.pack_type == "Keyboard" {
                online_kb_packs.push((idx, pack));
            } else {
                online_ms_packs.push((idx, pack));
            }
        }
        online_kb_packs.sort_by(|a, b| a.1.name.cmp(&b.1.name));
        online_ms_packs.sort_by(|a, b| a.1.name.cmp(&b.1.name));

        // Build Online Keyboard Submenu
        let downloading_id = DOWNLOADING_PACK_ID.lock().ok().and_then(|g| g.clone());
        let mut dl_kb_items_owned = Vec::new();
        for (_, pack) in &online_kb_packs {
            let is_installed = cache.soundpacks.contains_key(&format!("keyboard/{}", pack.id));
            // Compare by pack.id — matches what set_downloading_pack() stores
            let is_downloading = downloading_id.as_ref() == Some(&pack.id);
            let label = if is_downloading {
                format!("{} (Loading...)", pack.name)
            } else if is_installed {
                format!("{} (Installed)", pack.name)
            } else {
                pack.name.clone()
            };
            // Embed the pack ID (not an index) so event lookup never has ordering bugs
            let item = MenuItem::with_id(
                MenuId::new(format!("dl_kb_pack:{}", pack.id)),
                &label,
                !is_installed && !is_downloading,
                None,
            );
            dl_kb_items_owned.push(item);
        }
        let dl_kb_refs: Vec<&dyn tray_icon::menu::IsMenuItem> = dl_kb_items_owned
            .iter()
            .map(|item| item as &dyn tray_icon::menu::IsMenuItem)
            .collect();
        let dl_kb_submenu = Submenu::with_items("Download Keyboard Packs", true, &dl_kb_refs)?;

        // Build Online Mouse Submenu
        let mut dl_ms_items_owned = Vec::new();
        for (_, pack) in &online_ms_packs {
            let is_installed = cache.soundpacks.contains_key(&format!("mouse/{}", pack.id));
            let is_downloading = downloading_id.as_ref() == Some(&pack.id);
            let label = if is_downloading {
                format!("{} (Loading...)", pack.name)
            } else if is_installed {
                format!("{} (Installed)", pack.name)
            } else {
                pack.name.clone()
            };
            // Embed the pack ID in the menu item key
            let item = MenuItem::with_id(
                MenuId::new(format!("dl_ms_pack:{}", pack.id)),
                &label,
                !is_installed && !is_downloading,
                None,
            );
            dl_ms_items_owned.push(item);
        }
        let dl_ms_refs: Vec<&dyn tray_icon::menu::IsMenuItem> = dl_ms_items_owned
            .iter()
            .map(|item| item as &dyn tray_icon::menu::IsMenuItem)
            .collect();
        let dl_ms_submenu = Submenu::with_items("Download Mouse Packs", true, &dl_ms_refs)?;

        let download_item = MenuItem::with_id(
            MenuId::new("download_packs"),
            "Download More Sound Packs (Website)...",
            true,
            None
        );
        let separator3 = PredefinedMenuItem::separator();

        // External links section
        let github_item = MenuItem::with_id(MenuId::new("github"), "GitHub Repository", true, None);
        let discord_item = MenuItem::with_id(
            MenuId::new("discord"),
            "Discord Community",
            true,
            None
        );
        let website_item = MenuItem::with_id(
            MenuId::new("website"),
            "Official Website",
            true,
            None
        );
        let separator = PredefinedMenuItem::separator();
        let restart_item = MenuItem::with_id(MenuId::new("restart"), "Restart App", true, None);
        let exit_item = MenuItem::with_id(MenuId::new("exit"), "Exit", true, None);

        // Create the new menu
        let menu = Menu::with_items(
            &[
                &show_item,
                &separator1,
                &mute_item,
                &separator2,
                &kb_submenu,
                &ms_submenu,
                &dl_kb_submenu,
                &dl_ms_submenu,
                &download_item,
                &separator3,
                &github_item,
                &discord_item,
                &website_item,
                &separator,
                &restart_item,
                &exit_item,
            ]
        )?;

        // Update the tray icon with new menu
        self.tray_icon.set_menu(Some(Box::new(menu)));
        println!("🔄 Tray menu updated with soundpacks and download lists");

        Ok(())
    }
}

pub fn handle_tray_events() -> Option<TrayMessage> {
    // Handle menu events
    if let Ok(event) = MenuEvent::receiver().try_recv() {
        println!("🖱️ Tray menu event received: {:?}", event);
        match event.id.0.as_str() {
            "show" => {
                println!("🔼 Tray menu: Show {} clicked", APP_NAME);
                return Some(TrayMessage::Show);
            }
            "toggle_mute" => {
                println!("🔇 Tray menu: Toggle Mute clicked");
                return Some(TrayMessage::ToggleMute);
            }
            "download_packs" => {
                println!("🌐 Tray menu: Download More clicked");
                return Some(TrayMessage::DownloadPacks);
            }
            "github" => {
                println!("🐙 Tray menu: GitHub Repository clicked");
                return Some(TrayMessage::OpenGitHub);
            }
            "discord" => {
                println!("💬 Tray menu: Discord Community clicked");
                return Some(TrayMessage::OpenDiscord);
            }
            "website" => {
                println!("🌐 Tray menu: Official Website clicked");
                return Some(TrayMessage::OpenWebsite);
            }
            "restart" => {
                println!("🔄 Tray menu: Restart App clicked");
                return Some(TrayMessage::Restart);
            }
            "exit" => {
                println!("❌ Tray menu: Exit clicked");
                return Some(TrayMessage::Exit);
            }
            other => {
                if other.starts_with("kb_pack:") {
                    let id = other["kb_pack:".len()..].to_string();
                    println!("🎹 Tray menu: Keyboard pack clicked: {}", id);
                    return Some(TrayMessage::SetKeyboardSoundpack(id));
                } else if other.starts_with("ms_pack:") {
                    let id = other["ms_pack:".len()..].to_string();
                    println!("🐭 Tray menu: Mouse pack clicked: {}", id);
                    return Some(TrayMessage::SetMouseSoundpack(id));
                } else if other.starts_with("dl_kb_pack:") {
                    // The suffix IS the pack ID (embedded at menu-build time, no index lookup needed)
                    let pack_id = other["dl_kb_pack:".len()..].to_string();
                    let online_packs: Vec<OnlineSoundpack> =
                        serde_json::from_str(ONLINE_SOUNDPACKS_JSON).unwrap_or_default();
                    if let Some(pack) = online_packs.iter().find(|p| p.id == pack_id) {
                        return Some(TrayMessage::DownloadKeyboardPack {
                            name: pack.name.clone(),
                            url: pack.download_url.clone(),
                        });
                    }
                } else if other.starts_with("dl_ms_pack:") {
                    let pack_id = other["dl_ms_pack:".len()..].to_string();
                    let online_packs: Vec<OnlineSoundpack> =
                        serde_json::from_str(ONLINE_SOUNDPACKS_JSON).unwrap_or_default();
                    if let Some(pack) = online_packs.iter().find(|p| p.id == pack_id) {
                        return Some(TrayMessage::DownloadMousePack {
                            name: pack.name.clone(),
                            url: pack.download_url.clone(),
                        });
                    }
                } else {
                    println!("❓ Tray menu: Unknown menu item: {}", event.id.0);
                }
            }
        }
    }

    None
}