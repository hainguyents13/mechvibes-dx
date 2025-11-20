use tray_icon::{
    Icon,
    menu::{ Menu, MenuEvent, MenuItem, PredefinedMenuItem, MenuId },
    TrayIcon,
    TrayIconBuilder,
};
use crate::utils::constants::APP_NAME;

pub enum TrayMessage {
    Show,
    Exit,
    ToggleMute,
    OpenGitHub,
    OpenDiscord,
    OpenWebsite,
}

fn load_icon_from_ico(path: &str) -> Result<Icon, Box<dyn std::error::Error>> {
    // Load the ICO file
    let ico_data = std::fs::read(path)?;
    
    // Parse ICO file and extract RGBA data
    // ICO files can contain multiple images, we'll use the first one that's 32x32
    let img = image::load_from_memory(&ico_data)?;
    let img = img.resize_exact(32, 32, image::imageops::FilterType::Lanczos3);
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    
    Icon::from_rgba(rgba.into_raw(), width, height)
        .map_err(|e| format!("Failed to create icon from RGBA: {:?}", e).into())
}

pub struct TrayManager {
    tray_icon: TrayIcon,
}

impl TrayManager {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Load current config to determine sound state
        let config = crate::state::config::AppConfig::load();
        let mute_text = if config.enable_sound { "Mute sounds" } else { "Unmute sounds" };

        // Create the tray menu with specific IDs
        let show_item = MenuItem::with_id(
            MenuId::new("show"),
            &format!("Show {}", APP_NAME),
            true,
            None
        );
        let separator1 = PredefinedMenuItem::separator();

        // Sound control section
        let mute_item = MenuItem::with_id(MenuId::new("toggle_mute"), mute_text, true, None);
        let separator2 = PredefinedMenuItem::separator();

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

        let exit_item = MenuItem::with_id(MenuId::new("exit"), "Exit", true, None);

        // Create the menu with the items
        let menu = Menu::with_items(
            &[
                &show_item,
                &separator1,
                &mute_item,
                &separator2,
                &github_item,
                &discord_item,
                &website_item,
                &separator,
                &exit_item,
            ]
        )?;

        // Load the icon - convert ICO to RGBA
        let icon = load_icon_from_ico("assets/icon.ico")
            .expect("Failed to load icon");

        // Build the tray icon
        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip(APP_NAME)
            .with_icon(icon)
            .build()?;
        Ok(TrayManager {
            tray_icon: tray_icon,
        })
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

        let exit_item = MenuItem::with_id(MenuId::new("exit"), "Exit", true, None);

        // Create the new menu
        let menu = Menu::with_items(
            &[
                &show_item,
                &separator1,
                &mute_item,
                &separator2,
                &github_item,
                &discord_item,
                &website_item,
                &separator,
                &exit_item,
            ]
        )?;

        // Update the tray icon with new menu
        self.tray_icon.set_menu(Some(Box::new(menu)));
        println!("🔄 Tray menu updated with text: {}", mute_text);

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
            "exit" => {
                println!("❌ Tray menu: Exit clicked");
                return Some(TrayMessage::Exit);
            }
            _ => {
                println!("❓ Tray menu: Unknown menu item: {}", event.id.0);
            }
        }
    }

    None
}
