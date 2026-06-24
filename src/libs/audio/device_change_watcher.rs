use crossbeam_channel::{ unbounded, Receiver, Sender };
use cpal::traits::{ DeviceTrait, HostTrait };

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefaultDeviceEvent {
    OutputChanged,
}

pub fn start_default_device_watcher() -> Receiver<DefaultDeviceEvent> {
    let (sender, receiver) = unbounded();
    start_polling_watcher(sender);

    receiver
}

fn start_polling_watcher(sender: Sender<DefaultDeviceEvent>) {
    std::thread::spawn(move || {
        use std::time::Duration;
        let mut last_name = get_default_output_name();

        loop {
            std::thread::sleep(Duration::from_secs(2));
            let current_name = get_default_output_name();
            if current_name != last_name {
                last_name = current_name;
                let _ = sender.send(DefaultDeviceEvent::OutputChanged);
            }
        }
    });
}

fn get_default_output_name() -> String {
    cpal::default_host()
        .default_output_device()
        .and_then(|device| device.name().ok())
        .unwrap_or_default()
}
