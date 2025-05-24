use rdev::{listen, Event, EventType};
use std::sync::{mpsc::Sender, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

pub fn start_keyboard_listener(play_sound_tx: Sender<()>) {
    let last_press = Arc::new(Mutex::new(Instant::now()));

    thread::spawn(move || {
        println!("🎹 Keyboard listener started...");

        let result = listen(move |event: Event| {
            if let EventType::KeyPress(key) = event.event_type {
                println!("🛠 Key Pressed: {:?}", key);

                let now = Instant::now();
                let mut last = last_press.lock().unwrap();

                // ✅ Chỉ gọi `play_random_sound()` nếu thời gian giữa 2 lần nhấn lớn hơn X ms
                if now.duration_since(*last) > Duration::from_millis(200) {
                    *last = now;
                    let _ = play_sound_tx.send(());
                }
            }
        });

        if let Err(error) = result {
            eprintln!("❌ Keyboard listener error: {:?}", error);
        }
    });
}
