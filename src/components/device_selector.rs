use crate::libs::audio::{ AudioCommand, AudioContext };
use crate::libs::device_manager::{ DeviceInfo, DeviceManager };
use crate::libs::input_device_manager::{ InputDeviceInfo, InputDeviceManager };
use crate::utils::config::use_config;
use dioxus::prelude::*;
use lucide_dioxus::{ Headphones, Keyboard, Mouse, RefreshCw };
use std::sync::{ Arc, Mutex, OnceLock };

#[derive(Clone, PartialEq, Copy)]
pub enum DeviceType {
    AudioOutput,
    Keyboard,
    Mouse,
}

/// Input devices enumerated during this app session, kept outside the component
/// tree so they survive the unmount that happens on every tab switch.
/// Audio outputs already have an equivalent cache in `DeviceManager`.
static ENUMERATED_INPUT_DEVICES: OnceLock<Mutex<Vec<InputDeviceInfo>>> = OnceLock::new();

/// Whether the user has enumerated devices at least once this session.
/// Enumeration stays lazy — nothing runs until the refresh button is pressed —
/// but once it has run, re-entering the page shows the result instead of
/// falling back to the "click refresh" placeholder.
static DEVICES_ENUMERATED: OnceLock<Mutex<bool>> = OnceLock::new();

fn input_device_cache() -> &'static Mutex<Vec<InputDeviceInfo>> {
    ENUMERATED_INPUT_DEVICES.get_or_init(|| Mutex::new(Vec::new()))
}

fn devices_enumerated_flag() -> &'static Mutex<bool> {
    DEVICES_ENUMERATED.get_or_init(|| Mutex::new(false))
}

/// Records that an enumeration has completed, so remounts can restore results.
fn mark_devices_enumerated() {
    if let Ok(mut flag) = devices_enumerated_flag().lock() {
        *flag = true;
    }
}

fn have_devices_been_enumerated() -> bool {
    devices_enumerated_flag().lock().map(|flag| *flag).unwrap_or(false)
}

fn store_input_devices(devices: &[InputDeviceInfo]) {
    if let Ok(mut cache) = input_device_cache().lock() {
        *cache = devices.to_vec();
    }
}

fn cached_input_devices() -> Vec<InputDeviceInfo> {
    input_device_cache().lock().map(|cache| cache.clone()).unwrap_or_default()
}

/// Tells the Windows input worker host to reload its enabled-device filter,
/// so toggling a keyboard/mouse here takes effect immediately instead of on
/// the next app start. No-op elsewhere: on Linux/macOS the listeners read
/// the filter themselves.
fn notify_input_filter_changed() {
    #[cfg(target_os = "windows")]
    crate::libs::input_worker_host::notify_config_changed();
}

#[derive(Props, Clone, PartialEq)]
pub struct DeviceSelectorProps {
    device_type: DeviceType,
    label: String,
    description: Option<String>,
}

#[component]
pub fn DeviceSelector(props: DeviceSelectorProps) -> Element {
    let (config, update_config) = use_config();
    let audio_devices = use_signal(|| Vec::<DeviceInfo>::new());
    let input_devices = use_signal(|| Vec::<InputDeviceInfo>::new());
    let is_loading = use_signal(|| false);
    let has_loaded = use_signal(|| false); // Track if devices have been loaded at least once
    let error_message = use_signal(String::new);
    let device_status = use_signal(|| std::collections::HashMap::<String, bool>::new());

    // Get current selected/enabled devices
    let current_selection = use_memo(move || {
        let config = config();
        match props.device_type {
            DeviceType::AudioOutput => (config.selected_audio_device.clone(), Vec::<String>::new()),
            DeviceType::Keyboard => (None, config.enabled_keyboards.clone()),
            DeviceType::Mouse => (None, config.enabled_mice.clone()),
        }
    });

    // Load devices from cache (audio) or fresh (input devices)
    let load_cached_devices = {
        let mut audio_devices = audio_devices.clone();
        let mut input_devices = input_devices.clone();
        let mut is_loading = is_loading.clone();
        let mut has_loaded = has_loaded.clone();
        let mut error_message = error_message.clone();
        let device_type = props.device_type;

        use_callback(move |_| {
            spawn(async move {
                crate::always_print!("📋 [DeviceSelector] Loading devices...");
                is_loading.set(true);
                error_message.set(String::new());

                match device_type {
                    DeviceType::AudioOutput => {
                        // Use cached audio devices to avoid enumeration interference
                        match DeviceManager::get_cached_output_devices() {
                            Ok(device_list) => {
                                crate::always_print!("✅ [DeviceSelector] Loaded {} cached audio devices", device_list.len());
                                audio_devices.set(device_list);
                                mark_devices_enumerated();
                                has_loaded.set(true);
                            }
                            Err(e) => {
                                crate::always_print!("❌ [DeviceSelector] Failed to load cached devices: {}", e);
                                error_message.set(format!("Failed to load audio devices: {}", e));
                                mark_devices_enumerated();
                                has_loaded.set(true);
                            }
                        }
                    }
                    DeviceType::Keyboard | DeviceType::Mouse => {
                        // Input devices don't interfere with audio, load fresh
                        match InputDeviceManager::get_devices() {
                            Ok(device_list) => {
                                crate::always_print!("✅ [DeviceSelector] Loaded {} input devices", device_list.len());
                                store_input_devices(&device_list);
                                input_devices.set(device_list);
                                mark_devices_enumerated();
                                has_loaded.set(true);
                            }
                            Err(e) => {
                                crate::always_print!("❌ [DeviceSelector] Failed to load input devices: {}", e);
                                error_message.set(format!("Failed to load input devices: {}", e));
                                mark_devices_enumerated();
                                has_loaded.set(true);
                            }
                        }
                    }
                }

                is_loading.set(false);
            });
        })
    };

    // Refresh device cache (re-enumerate)
    let refresh_device_cache = {
        let load_cached = load_cached_devices.clone();
        let device_type = props.device_type;

        use_callback(move |_| {
            spawn(async move {
                crate::always_print!("🔄 [DeviceSelector] User clicked refresh - re-enumerating devices...");
                match device_type {
                    DeviceType::AudioOutput => {
                        // Refresh audio device cache
                        match DeviceManager::refresh_cache() {
                            Ok(_) => {
                                crate::always_print!("✅ [DeviceSelector] Audio cache refreshed successfully");
                                // Reload from refreshed cache
                                load_cached.call(());
                            }
                            Err(e) => {
                                crate::always_print!("❌ [DeviceSelector] Failed to refresh audio cache: {}", e);
                            }
                        }
                    }
                    DeviceType::Keyboard | DeviceType::Mouse => {
                        // Input devices - just reload fresh
                        crate::always_print!("🔄 [DeviceSelector] Reloading input devices...");
                        load_cached.call(());
                    }
                }
            });
        })
    };

    // Enumeration stays lazy: nothing is probed on first mount, because ALSA
    // enumeration can interrupt audio playback. But once the user has loaded
    // devices this session, restore that result on remount so switching tabs
    // does not appear to wipe the list.
    use_hook({
        let mut audio_devices = audio_devices;
        let mut input_devices = input_devices;
        let mut has_loaded = has_loaded;
        let device_type = props.device_type;

        move || {
            if !have_devices_been_enumerated() {
                return;
            }

            match device_type {
                DeviceType::AudioOutput => {
                    // Reading the cache is a plain clone; it does not re-enumerate.
                    if let Ok(device_list) = DeviceManager::get_cached_output_devices() {
                        audio_devices.set(device_list);
                    }
                }
                DeviceType::Keyboard | DeviceType::Mouse => {
                    input_devices.set(cached_input_devices());
                }
            }

            has_loaded.set(true);
        }
    });

    // Test device status (only for audio devices)
    let test_device_status = {
        let mut device_status = device_status.clone();
        let device_type = props.device_type;

        use_callback(move |device_id: String| {
            spawn(async move {
                match device_type {
                    DeviceType::AudioOutput => {
                        let device_manager = DeviceManager::new();
                        let is_available = device_manager
                            .test_output_device(&device_id)
                            .unwrap_or(false);
                        device_status.with_mut(|status| {
                            status.insert(device_id, is_available);
                        });
                    }
                    DeviceType::Keyboard | DeviceType::Mouse => {
                        // Input devices are always considered available if enumerated
                        device_status.with_mut(|status| {
                            status.insert(device_id, true);
                        });
                    }
                }
            });
        })
    };

    // Handle device selection/toggling
    let audio_ctx: Arc<AudioContext> = use_context();
    let handle_device_action = {
        let update_config = update_config.clone();
        let device_type = props.device_type;
        let test_device_status = test_device_status.clone();
        let audio_ctx = audio_ctx.clone();

        use_callback(move |device_id: String| {
            match device_type {
                DeviceType::AudioOutput => {
                    // Test device before selecting
                    test_device_status.call(device_id.clone());

                    let device_id_clone = device_id.clone();
                    let switch_target = if device_id_clone == "default" {
                        None
                    } else {
                        Some(device_id_clone.clone())
                    };
                    update_config(
                        Box::new(move |config| {
                            config.selected_audio_device = switch_target.clone();
                        })
                    );

                    // Apply at runtime: the audio engine and ambiance
                    // player both switch immediately, not just on next
                    // startup.
                    let runtime_target = if device_id == "default" {
                        None
                    } else {
                        Some(device_id)
                    };
                    audio_ctx.send(AudioCommand::SwitchDevice(runtime_target.clone()));
                    crate::state::ambiance::switch_ambiance_player_device(runtime_target);
                }
                DeviceType::Keyboard => {
                    let device_id_clone = device_id.clone();
                    update_config(
                        Box::new(move |config| {
                            if config.enabled_keyboards.contains(&device_id_clone) {
                                config.enabled_keyboards.retain(|id| id != &device_id_clone);
                            } else {
                                config.enabled_keyboards.push(device_id_clone);
                            }
                        })
                    );
                    notify_input_filter_changed();
                }
                DeviceType::Mouse => {
                    let device_id_clone = device_id.clone();
                    update_config(
                        Box::new(move |config| {
                            if config.enabled_mice.contains(&device_id_clone) {
                                config.enabled_mice.retain(|id| id != &device_id_clone);
                            } else {
                                config.enabled_mice.push(device_id_clone);
                            }
                        })
                    );
                    notify_input_filter_changed();
                }
            }
        })
    };

    // Get current device name for display
    let _current_device_name = use_memo(move || {
        let (selected_device, enabled_devices) = current_selection();

        match props.device_type {
            DeviceType::AudioOutput => {
                if selected_device.is_none() {
                    return "System Default".to_string();
                }

                let current_id = selected_device.unwrap();
                audio_devices()
                    .iter()
                    .find(|d| d.id == current_id)
                    .map(|d| d.name.clone())
                    .unwrap_or_else(|| "Unknown Device".to_string())
            }
            DeviceType::Keyboard | DeviceType::Mouse => {
                let device_count = enabled_devices.len();
                if device_count == 0 {
                    format!("All {}s", match props.device_type {
                        DeviceType::Keyboard => "Keyboard",
                        DeviceType::Mouse => "Mouse",
                        _ => "Device",
                    })
                } else {
                    format!("{} {} Selected", device_count, match props.device_type {
                        DeviceType::Keyboard => if device_count == 1 {
                            "Keyboard"
                        } else {
                            "Keyboards"
                        }
                        DeviceType::Mouse => if device_count == 1 { "Mouse" } else { "Mice" }
                        _ => "Devices",
                    })
                }
            }
        }
    });

    // Get device status for display
    let show_error_status = use_memo(move || {
        if props.device_type == DeviceType::AudioOutput {
            let (selected_device, _) = current_selection();
            if let Some(current) = selected_device {
                if let Some(status) = device_status().get(&current) {
                    return !status;
                }
            }
        }
        false
    });

    // Get no devices message
    let no_devices_message = use_memo(move || {
        match props.device_type {
            DeviceType::AudioOutput => "No audio devices found".to_string(),
            DeviceType::Keyboard => "No keyboard devices found".to_string(),
            DeviceType::Mouse => "No mouse devices found".to_string(),
        }
    });

    // Combined device list for audio (includes system default)
    let all_audio_devices = use_memo(move || {
        if props.device_type == DeviceType::AudioOutput {
            let mut devices = Vec::new();

            // Add system default as the first "device"
            devices.push((
                "default".to_string(),
                "System Default".to_string(),
                "Use system default audio device".to_string(),
                true,
            ));

            // Add hardware devices
            for device in audio_devices().iter() {
                devices.push((
                    device.id.clone(),
                    device.name.clone(),
                    "".to_string(),
                    device.is_default,
                ));
            }

            devices
        } else {
            Vec::new()
        }
    });

    // Helper function to render device icon
    let device_icon = move || {
        match props.device_type {
            DeviceType::AudioOutput =>
                rsx! {
                    Headphones { class: "w-4 h-4" }
                },
            DeviceType::Keyboard =>
                rsx! {
                    Keyboard { class: "w-4 h-4" }
                },
            DeviceType::Mouse =>
                rsx! {
                    Mouse { class: "w-4 h-4" }
                },
        }
    };

    rsx! {
        div { class: "space-y-2",
            // Label and description
            div { class: "flex items-center gap-2 text-sm font-bold text-base-content/80",
                {device_icon()}
                span { "{props.label}" }
                button {
                    class: "btn btn-ghost btn-xs",
                    onclick: move |_| refresh_device_cache.call(()),
                    disabled: is_loading(),
                    title: "Refresh device list",
                    if is_loading() {
                        RefreshCw { class: "w-3 h-3 animate-spin" }
                    } else {
                        RefreshCw { class: "w-3 h-3" }
                    }
                }
            }

            if let Some(desc) = &props.description {
                p { class: "text-xs text-base-content/60", "{desc}" }            }

            // Device list with radio buttons
            div { class: "bg-base-100 px-4 py-3 rounded-box space-y-2",
                match props.device_type {
                    DeviceType::AudioOutput => rsx! {
                        if audio_devices().is_empty() && !is_loading() {
                            div { class: "text-center text-base-content/50 py-8",
                                {device_icon()}
                                if !has_loaded() {
                                    div { class: "mt-2 text-sm", "Click the refresh button to load available devices" }
                                } else {
                                    div { class: "mt-2 text-sm", "{no_devices_message()}" }
                                }
                            }
                        } else {
                            div { class: "space-y-2",
                                // Unified device list (system default + hardware devices)
                                for (device_id, device_name, badge_text, is_default) in all_audio_devices().iter() {
                                    label {
                                        key: "{device_id}",
                                        class: "flex items-center gap-3 rounded-lg hover:bg-base-100 cursor-pointer transition-colors",
                                        input {
                                            r#type: "radio",
                                            name: "audio-device",
                                            class: "radio radio-xs radio-primary",
                                            checked: if device_id == "default" {
                                                current_selection().0.is_none()
                                            } else {
                                                current_selection().0.as_ref() == Some(device_id)
                                            },
                                            onchange: {
                                                let device_id_clone = device_id.clone();
                                                move |_| {
                                                    handle_device_action.call(device_id_clone.clone());
                                                }
                                            }
                                        }
                                        div { class: "flex items-center gap-2 flex-1",
                                            div { class: "flex-1 min-w-0",
                                                div { class: "text-xs font-medium flex items-center gap-2",
                                                    span { class: "line-clamp-1", "{device_name}" }
                                                    if device_id == "default" {
                                                        span { class: "badge badge-xs badge-outline", "Default" }
                                                    } else if *is_default && !badge_text.is_empty() {
                                                        span { class: "badge badge-xs badge-outline", "{badge_text}" }
                                                    }
                                                }
                                                if device_id == "default" {
                                                    div { class: "text-xs text-base-content/60", "{badge_text}" }
                                                } else {
                                                    div { class: "text-xs text-base-content/60", "Device ID: {device_id}" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    },
                    DeviceType::Keyboard | DeviceType::Mouse => rsx! {
                        if input_devices().is_empty() && !is_loading() {
                            div { class: "text-center text-base-content/50 py-8",
                                {device_icon()}
                                if !has_loaded() {
                                    div { class: "mt-2 text-sm", "Click the refresh button to load available devices" }
                                } else {
                                    div { class: "mt-2 text-sm", "{no_devices_message()}" }
                                }
                            }
                        } else {
                            div { class: "space-y-2",
                                // Available input devices
                                for device in input_devices().iter() {
                                    label { 
                                        key: "{device.id}",
                                        class: "flex items-center gap-3 p-3 rounded-lg hover:bg-base-100 cursor-pointer transition-colors",
                                        input {
                                            r#type: "checkbox",
                                            class: "checkbox checkbox-primary",
                                            checked: current_selection().1.contains(&device.id),
                                            onchange: {
                                                let device_id = device.id.clone();
                                                move |_| {
                                                    handle_device_action.call(device_id.clone());
                                                }
                                            }
                                        }
                                        div { class: "flex items-center gap-2 flex-1",
                                            {device_icon()}
                                            div { class: "flex-1 min-w-0",
                                                div { class: "text-sm font-medium truncate", "{device.name}" }
                                                div { class: "text-xs text-base-content/60", "{device.device_type:?}" }
                                            }
                                            div { class: "badge badge-success badge-sm", "Available" }
                                        }
                                    }
                                }
                            }
                        }
                    },
                }
            }

            // Error message
            if !error_message().is_empty() {
                div { class: "text-xs text-error mt-2", "{error_message()}" }
            }

            // Device status warning
            if show_error_status() {
                div { class: "alert alert-warning mt-2",
                    div { class: "text-sm",
                        "⚠️ Selected device may not be available. Audio may not work properly."
                    }
                }
            }

            // Linux-specific information
            if cfg!(target_os = "linux") && props.device_type == DeviceType::AudioOutput {
                div { class: "alert alert-info mt-2",
                    div { class: "text-xs",
                        if !has_loaded() {
                            "ℹ️ Linux: Click refresh to load available devices. This may briefly interrupt audio playback due to ALSA device enumeration."
                        } else {
                            "ℹ️ Linux: Refresh button will briefly interrupt audio playback (ALSA limitation)."
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn AudioOutputSelector() -> Element {
    rsx! {
        DeviceSelector {
            device_type: DeviceType::AudioOutput,
            label: "Audio Output Device".to_string(),
            description: Some("Select the audio device for soundpack playback".to_string()),
        }
    }
}

#[allow(dead_code)]
#[component]
pub fn KeyboardSelector() -> Element {
    rsx! {
        DeviceSelector {
            device_type: DeviceType::Keyboard,
            label: "Keyboard Devices".to_string(),
            description: Some("Select which keyboards should generate sound effects".to_string()),
        }
    }
}

#[allow(dead_code)]
#[component]
pub fn MouseSelector() -> Element {
    rsx! {
        DeviceSelector {
            device_type: DeviceType::Mouse,
            label: "Mouse Devices".to_string(),
            description: Some("Select which mice should generate sound effects".to_string()),
        }
    }
}
