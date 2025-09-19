use tray_icon::{
    Icon,
    menu::{ Menu, MenuEvent, MenuItem, PredefinedMenuItem, MenuId },
    TrayIcon,
    TrayIconBuilder,
};
use crate::utils::constants::APP_NAME;
use image;
use std::fs;
use std::path::Path;

pub enum TrayMessage {
    Show,
    Exit,
    ToggleMute,
    OpenGitHub,
    OpenDiscord,
    OpenWebsite,
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

        // Load the icon from the specified path
        let icon = load_icon_from_path("assets/icon.ico")?;

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

fn load_icon_from_path(path: &str) -> Result<Icon, Box<dyn std::error::Error>> {
    if !Path::new(path).exists() {
        return Err(format!("Icon file not found: {}", path).into());
    }

    // Read and decode the image
    let img = image::open(path)?;
    let img = img.to_rgba8();
    let (width, height) = img.dimensions();
    let rgba = img.into_raw();

    // Create Icon from RGBA data
    let icon = Icon::from_rgba(rgba, width, height)?;
    Ok(icon)
}
