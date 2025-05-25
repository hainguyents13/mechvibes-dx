use crate::libs::audio::AudioContext;
use crate::state::soundpack::SoundPack;
use dioxus::prelude::*;
use std::sync::Arc;

#[derive(Clone, Props)]
pub struct SoundpackSelectorProps {
    pub audio_ctx: Arc<AudioContext>,
}

impl PartialEq for SoundpackSelectorProps {
    fn eq(&self, _other: &Self) -> bool {
        true // Arc comparison not needed for this component
    }
}

#[allow(non_snake_case)]
pub fn SoundpackSelector(props: SoundpackSelectorProps) -> Element {
    let mut error = use_signal(String::new);
    let paths = use_signal(|| {
        // Load và convert tất cả các soundpack thành (id, name, dirname)
        std::fs::read_dir("./sounds")
            .map(|entries| {
                entries
                    .filter_map(|entry| {
                        entry.ok().and_then(|e| {
                            let path = e.path();
                            if path.join("config.json").exists() {
                                if let Ok(content) =
                                    std::fs::read_to_string(path.join("config.json"))
                                {
                                    if let Ok(pack) = serde_json::from_str::<SoundPack>(&content) {
                                        return Some((pack.id, pack.name));
                                    }
                                }
                            }
                            None
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    });

    // Lấy soundpack hiện tại từ config
    let config = crate::state::config::AppConfig::load();
    let mut current = use_signal(|| config.current_soundpack);

    rsx! {
      div { class: "p-4 rounded-lg 0 mb-4",
        div { class: "text-lg font-bold mb-2", "Sound Pack" }
        select {
          class: "w-full p-2  rounded border border-gray-600",
          value: "{current}",
          onchange: move |evt| {
              let selected_id = evt.data.value();
              error.set(String::new());
              if let Some((id, name)) = paths().iter().find(|(id, _)| id == &selected_id) {
                  let mut config = crate::state::config::AppConfig::load();
                  config.current_soundpack = id.clone();
                  if let Err(e) = config.save() {
                      error.set(format!("Failed to save config: {}", e));
                      return;
                  }
                  match props.audio_ctx.try_reload_soundpack() {
                      Ok(_) => {
                          current.set(id.clone());
                          println!("✅ Soundpack changed to: {}", name);
                      }
                      Err(e) => {
                          error.set(format!("Failed to load soundpack: {}", e));
                      }
                  }
              } else {
                  error.set("Invalid soundpack selected".to_string());
              }
          },
          {
              paths()
                  .iter()
                  .map(|(id, name)| {
                      rsx! {
                        option { key: "{id}", value: "{id}", "{name}" }
                      }
                  })
          }
        }
        {
            error
                .with(|err| {
                    (!err.is_empty()).then(|| rsx! {
                      div { class: "text-xs text-red-400 mt-2", "{err}" }
                    })
                })
        }
      }
    }
}
