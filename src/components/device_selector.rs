use crate::libs::device_manager::{ DeviceInfo, DeviceManager };
use crate::libs::input_device_manager::{ InputDeviceInfo, InputDeviceManager };
use crate::utils::config::use_config;
use dioxus::prelude::*;
use lucide_dioxus::{ Check, ChevronDown, Headphones, Keyboard, Mouse, RefreshCw, X };

#[derive(Clone, PartialEq, Copy)]
pub enum DeviceType {
    AudioOutput,
    Keyboard,
    Mouse,
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

    // Load devices on component mount and when refresh is triggered
    let load_devices = {
        let mut audio_devices = audio_devices.clone();
        let mut input_devices = input_devices.clone();
        let mut is_loading = is_loading.clone();
        let mut error_message = error_message.clone();
        let device_type = props.device_type;

        use_callback(move |_| {
            spawn(async move {
                is_loading.set(true);
                error_message.set(String::new());

                match device_type {
                    DeviceType::AudioOutput => {
                        let device_manager = DeviceManager::new();
                        match device_manager.get_output_devices() {
                            Ok(device_list) => {
                                audio_devices.set(device_list);
                            }
                            Err(e) => {
                                error_message.set(format!("Failed to load audio devices: {}", e));
                            }
                        }
                    }
                    DeviceType::Keyboard | DeviceType::Mouse => {
                        let mut input_manager = InputDeviceManager::new();
                        match input_manager.enumerate_devices() {
                            Ok(_) => {
                                let device_list = match device_type {
                                    DeviceType::Keyboard => input_manager.get_keyboards(),
                                    DeviceType::Mouse => input_manager.get_mice(),
                                    _ => Vec::new(),
                                };
                                input_devices.set(device_list);
                            }
                            Err(e) => {
                                error_message.set(format!("Failed to load input devices: {}", e));
                            }
                        }
                    }
                }

                is_loading.set(false);
            });
        })
    };

    // Load devices on mount
    use_effect(move || {
        load_devices.call(());
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
    let handle_device_action = {
        let update_config = update_config.clone();
        let device_type = props.device_type;
        let test_device_status = test_device_status.clone();

        use_callback(move |device_id: String| {
            match device_type {
                DeviceType::AudioOutput => {
                    // Test device before selecting
                    test_device_status.call(device_id.clone());

                    let device_id_clone = device_id.clone();
                    update_config(
                        Box::new(move |config| {
                            config.selected_audio_device = if device_id_clone == "default" {
                                None
                            } else {
                                Some(device_id_clone)
                            };
                        })
                    );
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
                }
            }
        })
    };

    // Get current device name for display
    let current_device_name = use_memo(move || {
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
                    onclick: move |_| load_devices.call(()),
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
                p { class: "text-xs text-base-content/60", "{desc}" }
            }

            // Device selector dropdown using DaisyUI
            details {
                class: "dropdown w-full",

                summary {
                    class: format!(
                        "btn btn-soft w-full justify-between line-clamp-1 max-w-40 truncate gap-3 h-12 rounded-box {}",
                        if is_loading() { "btn-disabled" } else { "" }
                    ),

                    div { class: "flex items-center gap-3 flex-1 min-w-0",
                        div { class: "flex items-center gap-2",
                            {device_icon()}
                            span { 
                              class: "text-sm truncate", 
                              "{current_device_name()}"
                            }                       
                          }

                        // Device status indicator (only for audio output)
                        if show_error_status() {
                            X { class: "w-4 h-4 text-error" }
                        }
                    }

                    ChevronDown { class: "w-4 h-4" }
                }

                ul {
                    class: "menu dropdown-content bg-base-100 rounded-box z-50 w-full p-2 shadow-lg max-h-60 overflow-y-auto",

                    match props.device_type {
                        DeviceType::AudioOutput => rsx! {
                            if audio_devices().is_empty() && !is_loading() {
                                li {
                                    div { class: "p-4 text-center text-base-content/50 text-sm",
                                        "No audio devices found"
                                    }
                                }
                            } else {
                                // System default option
                                li {
                                    a {
                                        class: format!(
                                            "flex items-center gap-3 {}",
                                            if current_selection().0.is_none() { "active" } else { "" }
                                        ),
                                        onclick: move |_| {
                                            handle_device_action.call("default".to_string());
                                        },

                                        {device_icon()}
                                        div { class: "flex-1 min-w-0",
                                            div { class: "text-sm font-medium", "System Default" }
                                            div { class: "text-xs text-base-content/50", "Use system default audio device" }
                                        }
                                        if current_selection().0.is_none() {
                                            Check { class: "w-4 h-4 text-success" }
                                        }
                                    }
                                }

                                // Available audio devices
                                for device in audio_devices().iter() {
                                    li {
                                        key: "{device.id}",
                                        a {
                                            class: format!(
                                                "flex items-center gap-3 {}",
                                                if current_selection().0.as_ref() == Some(&device.id) { "active" } else { "" }
                                            ),
                                            onclick: {
                                                let device_id = device.id.clone();
                                                move |_| {
                                                    handle_device_action.call(device_id.clone());
                                                }
                                            },

                                            {device_icon()}
                                            div { class: "flex-1 min-w-0",
                                                div { class: "text-sm font-medium flex items-center gap-2",
                                                    span { class: "truncate", "{device.name}" }
                                                    if device.is_default {
                                                        span { class: "badge badge-xs badge-outline", "Default" }
                                                    }
                                                }
                                                div { class: "text-xs text-base-content/50", "Device ID: {device.id}" }
                                            }

                                            // Status and selection indicator
                                            div { class: "flex items-center gap-2",
                                                if let Some(status) = device_status().get(&device.id) {
                                                    if !status {
                                                        X { class: "w-4 h-4 text-error" }
                                                    }
                                                }

                                                if current_selection().0.as_ref() == Some(&device.id) {
                                                    Check { class: "w-4 h-4 text-success" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        DeviceType::Keyboard | DeviceType::Mouse => rsx! {                            if input_devices().is_empty() && !is_loading() {
                                li {
                                    div { class: "p-4 text-center text-base-content/50 text-sm",
                                        "{no_devices_message()}"
                                    }
                                }
                            } else {// Available input devices
                                for device in input_devices().iter() {
                                    li {
                                        key: "{device.id}",
                                        a {
                                            class: format!(
                                                "flex items-center gap-3 {}",
                                                if current_selection().1.contains(&device.id) { "active" } else { "" }
                                            ),
                                            onclick: {
                                                let device_id = device.id.clone();
                                                move |_| {
                                                    handle_device_action.call(device_id.clone());
                                                }
                                            },

                                            {device_icon()}
                                            div { class: "flex-1 min-w-0",
                                                div { class: "text-sm font-medium", "{device.name}" }
                                                div { class: "text-xs text-base-content/50", "{device.device_type:?}" }
                                            }

                                            // Selection indicator (checkbox style for multi-select)
                                            if current_selection().1.contains(&device.id) {
                                                Check { class: "w-4 h-4 text-success" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Error message
            if !error_message().is_empty() {
                div { class: "text-xs text-error", "{error_message()}" }
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
