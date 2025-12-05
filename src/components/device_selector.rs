use crate::libs::device_manager::{ DeviceInfo, DeviceManager };
use crate::libs::input_device_manager::{ InputDeviceInfo, InputDeviceManager };
use crate::utils::config::use_config;
use dioxus::prelude::*;
use lucide_dioxus::{ Headphones, Keyboard, Mouse, RefreshCw };

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
                p { class: "text-xs text-base-content/60", "{desc}" }            }

            // Device list with radio buttons
            div { class: "bg-base-100 px-4 py-3 rounded-box space-y-2",
                match props.device_type {
                    DeviceType::AudioOutput => rsx! {
                        if audio_devices().is_empty() && !is_loading() {
                            div { class: "text-center text-base-content/50 py-8",
                                {device_icon()}
                                div { class: "mt-2 text-sm", "{no_devices_message()}" }
                            }                        } else {
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
                                div { class: "mt-2 text-sm", "{no_devices_message()}" }
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
            }        }
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
