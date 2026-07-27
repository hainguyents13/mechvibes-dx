use crate::libs::tray::{ handle_tray_events, set_downloading_pack, TrayManager, TrayMessage };
use crate::libs::tray_service::TRAY_UPDATE_SERVICE;
use crate::libs::window_manager::{ WindowAction, WINDOW_MANAGER };
use crate::{ debug_print, always_eprint };
use dioxus::desktop::use_window;
use dioxus::prelude::*;
use std::sync::mpsc;

#[component]
pub fn WindowController() -> Element {
    let window = use_window();
    let trigger_update = crate::state::app::use_state_trigger();

    // Create a static receiver for window actions
    let mut window_action_receiver = use_signal(|| None::<mpsc::Receiver<WindowAction>>); // Create a signal to hold the tray manager
    let mut tray_manager = use_signal(|| None::<TrayManager>);

    // Initialize the receiver once using use_resource to avoid reactive loops
    let _window_channel = use_resource(move || async move {
        let (tx, rx) = mpsc::channel::<WindowAction>();
        WINDOW_MANAGER.set_action_sender(tx);
        window_action_receiver.set(Some(rx));
    });

    // Initialize tray using use_resource to avoid reactive scope warnings
    let _tray_init = use_resource(move || async move {
        match TrayManager::new() {
            Ok(tray) => {
                debug_print!("✅ System tray initialized successfully");
                tray_manager.set(Some(tray));
            }
            Err(e) => {
                always_eprint!("❌ Failed to initialize system tray: {}", e);
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
                    tray_manager_clone.with_mut(|tray_opt| {
                        if let Some(tray) = tray_opt {
                            if let Err(e) = tray.update_menu() {
                                eprintln!("❌ Failed to update tray menu from global request: {}", e);
                            } else {
                                println!("✅ Tray menu updated from global request");
                            }
                        }
                    });
                }

                // Handle tray events
                if let Some(tray_message) = handle_tray_events() {
                    match tray_message {
                        TrayMessage::Show => {
                            window_clone.set_visible(true);
                            window_clone.set_focus();
                            WINDOW_MANAGER.set_visible(true);
                            debug_print!("🔼 Window shown from tray");
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
                                    // Sync the hot-path AtomicBool flag so keypress checks see the new state immediately
                                    crate::libs::audio::sync_sound_flags(
                                        config.enable_sound,
                                        config.enable_keyboard_sound,
                                        config.enable_mouse_sound,
                                    );
                                    debug_print!("🔇 Sounds {} via tray menu", status); // Update tray menu to reflect new state
                                    tray_manager_clone.with_mut(|tray_opt| {
                                        if let Some(tray) = tray_opt {
                                            if let Err(e) = tray.update_menu() {
                                                always_eprint!("❌ Failed to update tray menu: {}", e);
                                            }
                                        }
                                    });
                                }
                                Err(e) => {
                                    always_eprint!("❌ Failed to save config after mute toggle: {}", e);
                                }
                            }
                        }
                        TrayMessage::OpenGitHub => {
                            let url = "https://github.com/hainguyents13/mechvibes-dx";
                            if let Err(e) = open::that(url) {
                                always_eprint!("❌ Failed to open GitHub URL: {}", e);
                            } else {
                                debug_print!("🐙 Opened GitHub repository in browser");
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
                        TrayMessage::SetKeyboardSoundpack(pack_id) => {
                            let mut config = crate::state::config::AppConfig::load();
                            config.keyboard_soundpack = pack_id.clone();
                            config.last_updated = chrono::Utc::now();
                            if let Ok(_) = config.save() {
                                println!("🎹 Keyboard soundpack set to {} via tray", pack_id);
                                let audio_ctx = crate::libs::audio::get_global_audio_context();
                                let _ = crate::libs::audio::load_keyboard_soundpack(&audio_ctx, &pack_id);
                                crate::libs::tray_service::request_tray_update();
                            }
                        }
                        TrayMessage::SetMouseSoundpack(pack_id) => {
                            let mut config = crate::state::config::AppConfig::load();
                            config.mouse_soundpack = pack_id.clone();
                            config.last_updated = chrono::Utc::now();
                            if let Ok(_) = config.save() {
                                println!("🐭 Mouse soundpack set to {} via tray", pack_id);
                                let audio_ctx = crate::libs::audio::get_global_audio_context();
                                let _ = crate::libs::audio::load_mouse_soundpack(&audio_ctx, &pack_id);
                                crate::libs::tray_service::request_tray_update();
                            }
                        }
                        TrayMessage::DownloadPacks => {
                            // Show main window and signal to open Get Packs tab
                            window_clone.set_visible(true);
                            window_clone.set_focus();
                            WINDOW_MANAGER.set_visible(true);
                            crate::libs::tray_service::request_open_get_packs();
                            debug_print!("🌐 Requesting Get Packs tab from tray");
                        }
                        TrayMessage::DownloadKeyboardPack { name, url } => {
                            println!("📥 Downloading keyboard pack '{}' from {} via tray...", name, url);
                            let trigger_update_clone = trigger_update.clone();
                            spawn(async move {
                                set_downloading_pack(Some(name.clone()));
                                crate::libs::tray_service::request_tray_update();
                                match crate::utils::soundpack_installer::download_and_install_soundpack(
                                    &url,
                                    Some(crate::state::soundpack::SoundpackType::Keyboard)
                                ).await {
                                    Ok(info) => {
                                        println!("✅ Successfully downloaded and installed keyboard pack: {}", info.name);
                                        set_downloading_pack(None);
                                        // Refresh the global cache and trigger UI state reload
                                        crate::state::app::refresh_global_cache();
                                        trigger_update_clone(());

                                        // Auto-select the newly downloaded pack.
                                        // Use the prefixed folder_path ("keyboard/<id>") so the
                                        // soundpack selector matches correctly.
                                        let folder_path = format!("keyboard/{}", info.id);
                                        let mut config = crate::state::config::AppConfig::load();
                                        config.keyboard_soundpack = folder_path.clone();
                                        config.last_updated = chrono::Utc::now();
                                        if let Ok(_) = config.save() {
                                            let audio_ctx = crate::libs::audio::get_global_audio_context();
                                            let _ = crate::libs::audio::load_keyboard_soundpack(&audio_ctx, &folder_path);
                                            crate::libs::tray_service::request_tray_update();
                                        }
                                    }
                                    Err(e) => {
                                        always_eprint!("❌ Failed to download and install keyboard pack: {}", e);
                                        set_downloading_pack(None);
                                        crate::libs::tray_service::request_tray_update();
                                    }
                                }
                            });
                        }
                        TrayMessage::DownloadMousePack { name, url } => {
                            println!("📥 Downloading mouse pack '{}' from {} via tray...", name, url);
                            let trigger_update_clone = trigger_update.clone();
                            spawn(async move {
                                set_downloading_pack(Some(name.clone()));
                                crate::libs::tray_service::request_tray_update();
                                match crate::utils::soundpack_installer::download_and_install_soundpack(
                                    &url,
                                    Some(crate::state::soundpack::SoundpackType::Mouse)
                                ).await {
                                    Ok(info) => {
                                        println!("✅ Successfully downloaded and installed mouse pack: {}", info.name);
                                        set_downloading_pack(None);
                                        // Refresh the global cache and trigger UI state reload
                                        crate::state::app::refresh_global_cache();
                                        trigger_update_clone(());

                                        // Auto-select the newly downloaded pack.
                                        // Use the prefixed folder_path ("mouse/<id>") so the
                                        // soundpack selector matches correctly.
                                        let folder_path = format!("mouse/{}", info.id);
                                        let mut config = crate::state::config::AppConfig::load();
                                        config.mouse_soundpack = folder_path.clone();
                                        config.last_updated = chrono::Utc::now();
                                        if let Ok(_) = config.save() {
                                            let audio_ctx = crate::libs::audio::get_global_audio_context();
                                            let _ = crate::libs::audio::load_mouse_soundpack(&audio_ctx, &folder_path);
                                            crate::libs::tray_service::request_tray_update();
                                        }
                                    }
                                    Err(e) => {
                                        always_eprint!("❌ Failed to download and install mouse pack: {}", e);
                                        set_downloading_pack(None);
                                        crate::libs::tray_service::request_tray_update();
                                    }
                                }
                            });
                        }
                        TrayMessage::Restart => {
                            println!("📢 Tray: Restart requested - restarting application");
                            crate::utils::restart::restart_application();
                        }
                        TrayMessage::Exit => {
                            println!("📢 Tray: Exit requested - closing application");
                            std::process::exit(0);
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
