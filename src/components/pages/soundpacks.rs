use crate::{
    components::ui::{PageHeader, SoundpackImportModal, SoundpackManager, SoundpackTable},
    state::app::{use_app_state, use_state_trigger},
};
use dioxus::document::eval;
use dioxus::prelude::*;
use lucide_dioxus::{Keyboard, Mouse, Music, Settings2};
use std::sync::Arc;

#[component]
pub fn Soundpacks() -> Element {
    // Get app state and trigger function
    let app_state = use_app_state();
    let trigger_update = use_state_trigger();

    // Get all soundpacks (this will be reactive to app_state changes)
    let all_soundpacks = app_state.get_soundpacks();
    println!(
        "🔄 Soundpacks component rendering with {} soundpacks",
        all_soundpacks.len()
    );

    // Filter soundpacks by type (these will update when all_soundpacks changes)
    let keyboard_soundpacks: Vec<_> = all_soundpacks
        .iter()
        .filter(|pack| !pack.mouse)
        .cloned()
        .collect();

    let mouse_soundpacks: Vec<_> = all_soundpacks
        .iter()
        .filter(|pack| pack.mouse)
        .cloned()
        .collect();

    println!(
        "🔄 Filtered: {} keyboard, {} mouse soundpacks",
        keyboard_soundpacks.len(),
        mouse_soundpacks.len()
    );

    // Get access to audio context for reloading soundpacks
    let audio_ctx: Arc<crate::libs::audio::AudioContext> = use_context();

    rsx! {
      div { class: "p-12 pb-32",
        // Page header
        PageHeader {
          title: "Soundpacks".to_string(),
          subtitle: "Manage your soundpacks".to_string(),
          icon: Some(rsx! {
            Music { class: "w-8 h-8 mx-auto" }
          }),
        }

        // Tabs for soundpack types
        div { class: "tabs tabs-lift",
          // Keyboard tab
          label { class: "tab [--tab-border-color:var(--color-base-300)] [--tab-bg:var(--color-base-200)]",
            input {
              r#type: "radio",
              name: "soundpack-tab",
              checked: true,
            }
            Keyboard { class: "w-5 h-5 mr-2" }
            "Keyboard ({keyboard_soundpacks.len()})"
          }
          div { class: "tab-content overflow-hidden bg-base-200 border-base-300 py-4 px-0",
            SoundpackTable {
              soundpacks: keyboard_soundpacks,
              soundpack_type: "Keyboard",
              on_add_click: Some(
                  EventHandler::new(move |_| {
                      eval("soundpack_import_modal.showModal()");
                  }),
              ),
            }
          }

          // Mouse tab
          label { class: "tab [--tab-border-color:var(--color-base-300)] [--tab-bg:var(--color-base-200)]",
            input { r#type: "radio", name: "soundpack-tab" }
            Mouse { class: "w-5 h-5 mr-2" }
            "Mouse ({mouse_soundpacks.len()})"
          }
          div { class: "tab-content overflow-hidden bg-base-200 border-base-300 py-4 px-0",
            SoundpackTable {
              soundpacks: mouse_soundpacks,
              soundpack_type: "Mouse",
              on_add_click: Some(
                  EventHandler::new(move |_| {
                      eval("soundpack_import_modal.showModal()");
                  }),
              ),
            }
          }

          // Manage tab
          label { class: "tab [--tab-border-color:var(--color-base-300)] [--tab-bg:var(--color-base-200)]",
            input { r#type: "radio", name: "soundpack-tab" }
            Settings2 { class: "w-5 h-5 mr-2" }
            "Manage"
          }
          div { class: "tab-content overflow-hidden bg-base-200 border-base-300 p-4",
            SoundpackManager {
              on_import_click: EventHandler::new(move |_| {
                  eval("soundpack_import_modal.showModal()");
              }),
            }
          }
        }

        // Import modal
        SoundpackImportModal {
          modal_id: "soundpack_import_modal".to_string(),
          audio_ctx,
          on_import_success: EventHandler::new(move |_| {
              trigger_update(());
          }),
        }
      }
    }
}
