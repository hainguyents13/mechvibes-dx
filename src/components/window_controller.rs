use crate::libs::tray::{ handle_tray_events, TrayManager, TrayMessage };
use crate::libs::tray_service::TRAY_UPDATE_SERVICE;
use crate::libs::window_manager::{ WindowAction, WINDOW_MANAGER };
use dioxus::desktop::use_window;
use dioxus::prelude::*;
use std::sync::mpsc;

#[component]
pub fn WindowController() -> Element {
    let window = use_window();

    // Create a static receiver for window actions
    let mut window_action_receiver = use_signal(|| None::<mpsc::Receiver<WindowAction>>); // Create a signal to hold the tray manager
    let mut tray_manager = use_signal(|| None::<TrayManager>);

    // Initialize the receiver and tray once
    use_effect(move || {
        if window_action_receiver.read().is_none() {
            let (tx, rx) = mpsc::channel::<WindowAction>();
            WINDOW_MANAGER.set_action_sender(tx);
            *window_action_receiver.write() = Some(rx);
        }
    });

    // Initialize tray in a separate effect to avoid reactive scope warnings
    use_effect(move || {
        if tray_manager.read().is_none() {
            match TrayManager::new() {
                Ok(tray) => {
                    println!("✅ System tray initialized successfully");
                    *tray_manager.write() = Some(tray);
                }
                Err(e) => {
                    eprintln!("❌ Failed to initialize system tray: {}", e);
                }
            }
        }
    });

    // Use effect to listen for both window actions and tray events
    use_effect(move || {
        let window_clone = window.clone();
        let mut tray_manager_clone = tray_manager.clone();

        spawn(async move {
            loop {
                // Handle window actions from internal sources
                if let Some(receiver) = window_action_receiver.read().as_ref() {
                    if let Ok(action) = receiver.try_recv() {
                        match action {
                            WindowAction::Show => {
                                window_clone.set_visible(true);
                                window_clone.set_focus();
                                WINDOW_MANAGER.set_visible(true);
                                println!("🔼 Window shown from internal action");
                            }
                            WindowAction::Hide => {
                                window_clone.set_visible(false);
                                WINDOW_MANAGER.set_visible(false);
                                println!("🔽 Window hidden from internal action");
                            }
                        }
                    }
                }

                // Handle tray update requests from other parts of the application
                if let Some(_) = TRAY_UPDATE_SERVICE.try_receive() {
                    if let Some(ref mut tray) = tray_manager_clone.write().as_mut() {
                        if let Err(e) = tray.update_menu() {
                            eprintln!("❌ Failed to update tray menu from global request: {}", e);
                        } else {
                            println!("✅ Tray menu updated from global request");
                        }
                    }
                }

                // Handle tray events
                if let Some(tray_message) = handle_tray_events() {
                    match tray_message {
                        TrayMessage::Show => {
                            window_clone.set_visible(true);
                            window_clone.set_focus();
                            WINDOW_MANAGER.set_visible(true);
                            println!("🔼 Window shown from tray");
                        }
                        TrayMessage::ToggleMute => {
                            // Toggle the global sound enable flag
                            let mut config = crate::state::config::AppConfig::load();
                            config.enable_sound = !config.enable_sound;
                            // Update timestamp to trigger UI refresh
                            config.last_updated = chrono::Utc::now();
                            match config.save() {
                                Ok(_) => {
                                    let status = if config.enable_sound {
                                        "enabled"
                                    } else {
                                        "disabled"
                                    };
                                    println!("🔇 Sounds {} via tray menu", status);

                                    // Update tray menu to reflect new state
                                    if let Some(ref mut tray) = tray_manager_clone.write().as_mut() {
                                        if let Err(e) = tray.update_menu() {
                                            eprintln!("❌ Failed to update tray menu: {}", e);
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!("❌ Failed to save config after mute toggle: {}", e);
                                }
                            }
                        }
                        TrayMessage::OpenGitHub => {
                            let url = "https://github.com/hainguyents13/mechvibes-dx";
                            if let Err(e) = open::that(url) {
                                eprintln!("❌ Failed to open GitHub URL: {}", e);
                            } else {
                                println!("🐙 Opened GitHub repository in browser");
                            }
                        }
                        TrayMessage::OpenDiscord => {
                            let url = "https://discord.com/invite/MMVrhWxa4w";
                            if let Err(e) = open::that(url) {
                                eprintln!("❌ Failed to open Discord URL: {}", e);
                            } else {
                                println!("💬 Opened Discord community in browser");
                            }
                        }
                        TrayMessage::OpenWebsite => {
                            let url = "https://mechvibes.com";
                            if let Err(e) = open::that(url) {
                                eprintln!("❌ Failed to open website URL: {}", e);
                            } else {
                                println!("🌐 Opened official website in browser");
                            }
                        }
                        TrayMessage::Exit => {
                            println!("📢 Tray: Exit requested - closing application");
                            // Close the window which will trigger app exit
                            window_clone.close();
                        }
                    }
                }
                // Small delay to prevent busy-waiting
                futures_timer::Delay::new(std::time::Duration::from_millis(50)).await;
            }
        });
    });

    rsx! {
      // This component doesn't render anything visible
      span { style: "display: none;" }
    }
}
