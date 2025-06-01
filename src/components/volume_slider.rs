use crate::state::config_utils::use_config;
use dioxus::prelude::*;
use lucide_dioxus::{Volume2, VolumeX};

#[component]
fn VolumeSliderBase(
    volume: Signal<f32>,
    on_change: Option<EventHandler<f32>>,
    id: String,
    label_text: Option<String>,
) -> Element {
    // Use shared config hook for enable_sound
    let (config, update_config) = use_config();
    let mut enable_sound = use_signal(|| config().enable_sound);

    let label_text = label_text.unwrap_or_else(|| "Volume".to_string());

    rsx! {
      div { class: "grid grid-cols-12",
        div { class: "rounded col-span-4 flex items-center",
          label { r#for: "{id}", class: "label label-text text-base", "{label_text} " }
          span { class: "font-bold ml-1", "{(volume() * 100.0) as u8}%" }
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
              class: "btn btn-square btn-ghost",
              onclick: {
                  let update_config = update_config.clone();
                  move |_| {
                      let new_enable_sound = !enable_sound();
                      enable_sound.set(new_enable_sound);
                      update_config(
                          Box::new(move |config| {
                              config.enable_sound = new_enable_sound;
                          }),
                      );
                  }
              },
              if enable_sound() {
                Volume2 { class: "w-5 h-5" }
              } else {
                VolumeX { class: "w-5 h-5" }
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
      }
    }
}
