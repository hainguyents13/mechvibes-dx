use crate::components::soundpack_selector::{SelectorType, SoundpackSelector};
use dioxus::prelude::*;
use lucide_dioxus::Mouse;

#[allow(non_snake_case)]
pub fn MouseSoundpackSelector() -> Element {
    rsx! {        SoundpackSelector {
            soundpack_type: SelectorType::Mouse,
            label: "Mouse".to_string(),
            icon: rsx! { Mouse { class: "w-4 h-4" } }
        }
    }
}
