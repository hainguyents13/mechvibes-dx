use rdev::{listen, Event, EventType};
use std::sync::{mpsc::Sender, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

pub fn start_keyboard_listener(play_sound_tx: Sender<String>) {
    let last_press = Arc::new(Mutex::new(Instant::now()));

    thread::spawn(move || {
        println!("🎹 Keyboard listener started...");

        let result = listen(move |event: Event| {
            match event.event_type {
                EventType::KeyPress(key) => {
                    println!("🛠 Key Pressed: {:?}", key);
                    let now = Instant::now();
                    let mut last = last_press.lock().unwrap();
                    if now.duration_since(*last) > Duration::from_millis(1) {
                        *last = now;
                        let key_str = format!("{:?}", key);
                        let _ = play_sound_tx.send(key_str);
                    }
                }
                EventType::KeyRelease(key) => {
                    println!("🛠 Key Released: {:?}", key);
                    let key_str = format!("{:?}", key);
                    // Gửi keyup với tiền tố "UP:" để phân biệt
                    let _ = play_sound_tx.send(format!("UP:{}", key_str));
                }
                _ => {}
            }
        });

        if let Err(error) = result {
            eprintln!("❌ Keyboard listener error: {:?}", error);
        }
    });
}
