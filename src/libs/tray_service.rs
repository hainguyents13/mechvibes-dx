// Global tray service for coordinating tray menu updates across the application
use once_cell::sync::Lazy;
use std::sync::atomic::{ AtomicBool, Ordering };
use std::sync::mpsc::{ self, Receiver, Sender };
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub enum TrayUpdateMessage {
    RefreshMenu,
}

pub struct TrayUpdateService {
    sender: Sender<TrayUpdateMessage>,
    receiver: Mutex<Receiver<TrayUpdateMessage>>,
}

// Global flag: tray wants to open the "Get Packs" tab
static OPEN_GET_PACKS: AtomicBool = AtomicBool::new(false);

pub fn request_open_get_packs() {
    OPEN_GET_PACKS.store(true, Ordering::Relaxed);
}

pub fn peek_open_get_packs() -> bool {
    OPEN_GET_PACKS.load(Ordering::Relaxed)
}

pub fn take_open_get_packs() -> bool {
    OPEN_GET_PACKS.swap(false, Ordering::Relaxed)
}

impl TrayUpdateService {
    fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            sender,
            receiver: Mutex::new(receiver),
        }
    }

    /// Send a request to update the tray menu
    pub fn request_update(&self) {
        if let Err(e) = self.sender.send(TrayUpdateMessage::RefreshMenu) {
            eprintln!("❌ Failed to send tray update request: {}", e);
        }
    }

    /// Try to receive tray update messages (non-blocking)
    pub fn try_receive(&self) -> Option<TrayUpdateMessage> {
        if let Ok(receiver) = self.receiver.lock() { receiver.try_recv().ok() } else { None }
    }
}

// Global tray update service instance
pub static TRAY_UPDATE_SERVICE: Lazy<TrayUpdateService> = Lazy::new(TrayUpdateService::new);

/// Request a tray menu update from anywhere in the application
pub fn request_tray_update() {
    TRAY_UPDATE_SERVICE.request_update();
    println!("🔄 Tray menu update requested");
}
