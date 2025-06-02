use crate::state::config_utils::use_config;
use dioxus::prelude::*;
use lucide_dioxus::{Volume2, VolumeOff};

#[derive(Clone, PartialEq, Copy)]
pub enum VolumeType {
    Keyboard, // Controls enable_keyboard_sound
    Mouse,    // Controls enable_mouse_sound
}

#[component]
fn VolumeSliderBase(
    volume: Signal<f32>,
    on_change: Option<EventHandler<f32>>,
    id: String,
    label_text: Option<String>,
    volume_type: VolumeType,
) -> Element {
    // Use shared config hook for enable_sound
    let (config, update_config) = use_config();

    // Get the appropriate enable state based on volume type
    let enable_sound = use_memo(move || {
        let config = config();
        match volume_type {
            VolumeType::Keyboard => config.enable_keyboard_sound,
            VolumeType::Mouse => config.enable_mouse_sound,
        }
    });

    let label_text = label_text.unwrap_or_else(|| "Volume".to_string());

    rsx! {
      div { class: "grid grid-cols-12",
        div { class: "rounded col-span-4 flex items-center",
          label { r#for: "{id}", class: "label label-text text-base", "{label_text} " }
          span {
            class: format!(
                "font-bold ml-1 {}",
                if enable_sound() { "text-base-content" } else { "text-base-content/50" },
            ),
            "{(volume() * 100.0) as u8}%"
          }
        }
        div { class: "col-span-8 flex items-center gap-2",
          input {
            class: "range range-xs grow",
            r#type: "range",
            min: 0.0,
            max: 1.0,
            step: 0.05,
            id: "{id}",
            value: volume(),
            disabled: !enable_sound(),
            oninput: move |evt| {
                if let Ok(val) = evt.value().parse::<f32>() {
                    volume.set(val);
                    if let Some(handler) = on_change {
                        handler.call(val);
                    }
                }
            },
          }
          div {
            class: "tooltip",
            "data-tip": if enable_sound() { "Mute" } else { "Unmute" },
            button {
              class: format!(
                  "btn btn-square btn-ghost rounded-lg {}",
                  if !enable_sound() { "btn-active" } else { "" },
              ),
              onclick: {
                  let update_config = update_config.clone();
                  let volume_type = volume_type.clone();
                  move |_| {
                      match volume_type {
                          VolumeType::Keyboard => {
                              let config = config();
                              if config.enable_sound {
                                  let new_enable_keyboard = !config.enable_keyboard_sound;
                                  update_config(
                                      Box::new(move |config| {
                                          config.enable_keyboard_sound = new_enable_keyboard;
                                      }),
                                  );
                              }
                          }
                          VolumeType::Mouse => {
                              let config = config();
                              if config.enable_sound {
                                  let new_enable_mouse = !config.enable_mouse_sound;
                                  update_config(
                                      Box::new(move |config| {
                                          config.enable_mouse_sound = new_enable_mouse;
                                      }),
                                  );
                              }
                          }
                      }
                  }
              },
              if enable_sound() {
                Volume2 { class: "w-5 h-5" }
              } else {
                VolumeOff { class: "w-5 h-5" }
              }
            }
          }
        }
      }
    }
}

#[component]
pub fn VolumeSlider(volume: Signal<f32>, on_change: Option<EventHandler<f32>>) -> Element {
    rsx! {
      VolumeSliderBase {
        volume,
        on_change,
        id: "volume-slider".to_string(),
        label_text: Some("Volume".to_string()),
        volume_type: VolumeType::Keyboard,
      }
    }
}

#[component]
pub fn MouseVolumeSlider(volume: Signal<f32>, on_change: Option<EventHandler<f32>>) -> Element {
    rsx! {
      VolumeSliderBase {
        volume,
        on_change,
        id: "mouse-volume-slider".to_string(),
        label_text: Some("Volume".to_string()),
        volume_type: VolumeType::Mouse,
      }
    }
}

#[component]
pub fn KeyboardVolumeSlider(volume: Signal<f32>, on_change: Option<EventHandler<f32>>) -> Element {
    rsx! {
      VolumeSliderBase {
        volume,
        on_change,
        id: "keyboard-volume-slider".to_string(),
        label_text: Some("Volume".to_string()),
        volume_type: VolumeType::Keyboard,
      }
    }
}
