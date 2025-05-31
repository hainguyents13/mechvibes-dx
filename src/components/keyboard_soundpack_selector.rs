use crate::components::soundpack_selector::{SelectorType, SoundpackSelector};
use dioxus::prelude::*;
use lucide_dioxus::Keyboard;

#[allow(non_snake_case)]
pub fn KeyboardSoundpackSelector() -> Element {
    rsx! {
        SoundpackSelector {
            soundpack_type: SelectorType::Keyboard,
            label: "Keyboard".to_string(),
            icon: rsx! { Keyboard { class: "w-4 h-4" } }
        }
    }
}
