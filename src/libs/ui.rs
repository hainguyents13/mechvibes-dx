use crate::libs::audio::AudioContext;
use crate::libs::keyboard::start_keyboard_listener;
use dioxus::prelude::*;
use std::sync::mpsc;
use std::sync::Arc;

pub fn app() -> Element {
    // Create AudioContext and store it in a hook
    let audio_context = use_hook(|| Arc::new(AudioContext::new()));
    // Create a channel for keyboard events
    let (tx, rx) = mpsc::channel::<()>();
    let rx = Arc::new(rx);

    // Start keyboard listener in background, passing the sender
    {
        let tx = tx.clone();
        use_future(move || {
            let tx = tx.clone();
            async move {
                spawn(async move {
                    start_keyboard_listener(tx);
                });
            }
        });
    }

    // Poll the receiver and play sound if a message is received
    {
        let ctx = audio_context.clone();
        let rx = rx.clone();
        use_future(move || {
            let ctx = ctx.clone();
            let rx = rx.clone();
            async move {
                loop {
                    if rx.try_recv().is_ok() {
                        ctx.play_random_sound();
                    }
                    // Yield to the async runtime instead of sleeping the thread
                    futures_timer::Delay::new(std::time::Duration::from_millis(10)).await;
                }
            }
        });
    }

    rsx! {
      div { style: "padding: 20px; text-align: center;",
        h1 { "Mechvibes Dioxus" }
        button {
          onclick: {
              let ctx = audio_context.clone();
              move |_| {
                  println!("🛠 Button clicked!");
                  println!("🎵 Playing button click sound...");
                  ctx.play_random_sound();
              }
          },
          "Test âm thanh"
        }
      }
    }
}
