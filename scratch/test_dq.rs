use device_query::{DeviceQuery, DeviceState};
use std::thread;
use std::time::Duration;

fn main() {
    let device_state = DeviceState::new();
    println!("Starting device_query poller... Type keys anywhere. Press Ctrl+C to exit.");
    loop {
        let keys = device_state.get_keys();
        if !keys.is_empty() {
            println!("Pressed keys: {:?}", keys);
        }
        thread::sleep(Duration::from_millis(100));
    }
}
