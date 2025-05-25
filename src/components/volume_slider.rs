use dioxus::prelude::*;

#[component]
pub fn VolumeSlider(volume: Signal<f32>) -> Element {
    rsx! {
      div { class: "grid grid-cols-12 gap-4",
        div { class: "rounded col-span-4 flex items-center",
          label {
            r#for: "volume-slider",
            class: "label label-text text-base",
            "Volume "
          }
          span { class: "font-bold ml-1", "{(volume() * 100.0) as u8}%" }
        }
        div { class: "col-span-8",
          input {
            class: "range range-neutral range-xs",
            r#type: "range",
            min: 0.0,
            max: 1.0,
            step: 0.05,
            id: "volume-slider",
            value: volume(),
            oninput: move |evt| {
                if let Ok(val) = evt.value().parse::<f32>() {
                    volume.set(val);
                }
            },
          }
        }
      }
    }
}
