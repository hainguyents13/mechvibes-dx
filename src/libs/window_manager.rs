use once_cell::sync::Lazy;
use std::sync::{ Arc, Mutex, mpsc };

#[derive(Debug, Clone)]
pub enum WindowAction {
    #[allow(dead_code)]
    Show,
    Hide,
}

#[derive(Clone)]
pub struct WindowManager {
    pub is_visible: Arc<Mutex<bool>>,
    pub action_sender: Arc<Mutex<Option<mpsc::Sender<WindowAction>>>>,
}

impl WindowManager {
    pub fn new() -> Self {
        Self {
            is_visible: Arc::new(Mutex::new(true)),
            action_sender: Arc::new(Mutex::new(None)),
        }
    }

    pub fn set_action_sender(&self, sender: mpsc::Sender<WindowAction>) {
        if let Ok(mut action_sender) = self.action_sender.lock() {
            *action_sender = Some(sender);
        }
    }

    pub fn send_action(&self, action: WindowAction) {
        if let Ok(sender_guard) = self.action_sender.lock() {
            if let Some(sender) = sender_guard.as_ref() {
                let _ = sender.send(action);
            }
        }
    }

    pub fn set_visible(&self, visible: bool) {
        if let Ok(mut is_visible) = self.is_visible.lock() {
            *is_visible = visible;
        }
    }

    pub fn hide(&self) {
        self.set_visible(false);
        self.send_action(WindowAction::Hide);
    }
}

pub static WINDOW_MANAGER: Lazy<WindowManager> = Lazy::new(|| WindowManager::new());
