use crate::{components::ui::PageHeader, state::app::use_app_state};
use dioxus::prelude::*;
use lucide_dioxus::{ExternalLink, FolderOpen, Keyboard, Mouse, Music, Plus, Trash};

#[component]
pub fn SoundpacksPage() -> Element {
    // Get global app state for soundpacks
    let app_state = use_app_state();
    let soundpacks = use_memo(move || app_state.with(|state| state.get_soundpacks()));

    // Filter soundpacks by type
    let keyboard_soundpacks = use_memo(move || {
        soundpacks()
            .into_iter()
            .filter(|pack| !pack.mouse)
            .collect::<Vec<_>>()
    });

    let mouse_soundpacks = use_memo(move || {
        soundpacks()
            .into_iter()
            .filter(|pack| pack.mouse)
            .collect::<Vec<_>>()
    });

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

        // Content based on active tab
        div { class: "tabs tabs-lift",
          label { class: "tab [--tab-border-color:var(--color-base-content)]",
            input {
              r#type: "radio",
              name: "soundpack-tab",
              checked: true,
            }
            Keyboard { class: "w-5 h-5 mr-2" }
            "Keyboard ({keyboard_soundpacks().len()})"
          }
          div { class: format!("tab-content  overflow-hidden border-base-content py-4 px-0"),
            SoundpackTable {
              soundpacks: keyboard_soundpacks(),
              soundpack_type: "Keyboard",
            }
          }
          label { class: "tab [--tab-border-color:var(--color-base-content)]",
            input { r#type: "radio", name: "soundpack-tab" }
            Mouse { class: "w-5 h-5 mr-2" }
            "Mouse ({mouse_soundpacks().len()})"
          }
          div { class: format!("tab-content overflow-hidden border-base-content py-4 px-0"),
            SoundpackTable {
              soundpacks: mouse_soundpacks(),
              soundpack_type: "Mouse",
            }
          }
        }

        div { class: "mt-4",
          div { class: "text-center",
            button { class: "btn btn-primary",
              Plus { class: "w-4 h-4 mr-2" }
              "Add"
            }
          }
          div { class: "text-center mt-2",
            a {
              class: "btn btn-ghost btn-sm",
              href: "https://mechvibes.com/soundpacks",
              target: "_blank",
              "Get Soundpacks from website"
              ExternalLink { class: "w-3 h-3" }
            }
          }
        }
      }
    }
}

#[component]
fn SoundpackTable(
    soundpacks: Vec<crate::state::soundpack_cache::SoundpackMetadata>,
    soundpack_type: &'static str,
) -> Element {
    if soundpacks.is_empty() {
        rsx! {
          div { class: "p-4 text-center text-base-content/70",
            "No {soundpack_type} soundpack found!"
          }
        }
    } else {
        rsx! {
          div { class: "overflow-x-auto",
            table { class: "table table-sm w-full",
              tbody {
                for pack in soundpacks {
                  SoundpackTableRow { soundpack: pack }
                }
              }
            }
          }
        }
    }
}

#[component]
fn SoundpackTableRow(soundpack: crate::state::soundpack_cache::SoundpackMetadata) -> Element {
    rsx! {
      tr { class: "hover:bg-base-200/50",
        td { class: "flex items-center gap-4",
          // Icon
          div { class: "flex items-center justify-center",
            if let Some(icon) = &soundpack.icon {
              if !icon.is_empty() {
                img {
                  class: "w-8 h-8 rounded object-cover",
                  src: "{icon}",
                  alt: "{soundpack.name}",
                }
              } else {
                div { class: "w-8 h-8 rounded bg-base-300 flex items-center justify-center",
                  Music { class: "w-4 h-4 text-base-content/40" }
                }
              }
            } else {
              div { class: "w-8 h-8 rounded bg-base-300 flex items-center justify-center",
                Music { class: "w-4 h-4 text-base-content/40" }
              }
            }
          }
          // Name
          div {
            div { class: "font-medium text-sm text-base-content", "{soundpack.name}" }
            if let Some(author) = &soundpack.author {
              div { class: "text-xs text-base-content/50", "by {author}" }
            }
          }
        }
        // Actions
        td {
          div { class: "flex items-center justify-end gap-2",
            button {
              class: "btn btn-soft btn-xs",
              title: "Edit this soundpack",
              FolderOpen { class: "w-4 h-4" }
            }
            button {
              class: "btn btn-error btn-xs",
              title: "Delete this soundpack",
              Trash { class: "w-4 h-4" }
            }
          }
        }
      }
    }
}
