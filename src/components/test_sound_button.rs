use dioxus::prelude::*;
use lucide_dioxus::ArrowRight;

#[component]
pub fn TestSoundButton() -> Element {
    rsx! {
        button {
            class: "btn btn-secondary",
            onclick: move |_| {
                println!("🛠 Button clicked!");
            },
            ArrowRight { size: 16 }
            "Test âm thanh"
        }
    }
}
