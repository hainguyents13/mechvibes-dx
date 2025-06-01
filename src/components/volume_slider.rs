use dioxus::prelude::*;
use lucide_dioxus::Volume2;

#[component]
pub fn VolumeSlider(volume: Signal<f32>, on_change: Option<EventHandler<f32>>) -> Element {
    rsx! {
      div { class: "grid grid-cols-12",
        div { class: "rounded col-span-4 flex items-center",
          label {
            r#for: "volume-slider",
            class: "label label-text text-base",
            "Volume "
          }
          span { class: "font-bold ml-1", "{(volume() * 100.0) as u8}%" }
        }
        div { class: "col-span-8 flex items-center gap-2",
          input {
            class: "range range-xs grow",
            r#type: "range",
            min: 0.0,
            max: 1.0,
            step: 0.05,
            id: "volume-slider",
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
          div { class: "tooltip", "data-tip": "Mute",
            button {
              class: "btn btn-square btn-ghost",
              onclick: move |_| {},
              Volume2 { class: "w-5 h-5" }
            }
          }
        }
      }
    }
}
