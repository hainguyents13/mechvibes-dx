use once_cell::sync::Lazy;
use std::sync::{ Arc, Mutex, mpsc };

#[derive(Debug, Clone)]
pub enum WindowAction {
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

    /// Deliver a window action, reporting whether anything is listening.
    ///
    /// The sender installed before the UI mounts has no live receiver - it is
    /// replaced once `WindowController` is running - so an action sent in that
    /// window is dropped. Callers that must not lose one (a second launch
    /// asking for the window) use the return value to retry.
    pub fn send_action(&self, action: WindowAction) -> bool {
        if let Ok(sender_guard) = self.action_sender.lock() {
            if let Some(sender) = sender_guard.as_ref() {
                return sender.send(action).is_ok();
            }
        }
        false
    }

    pub fn set_visible(&self, visible: bool) {
        if let Ok(mut is_visible) = self.is_visible.lock() {
            *is_visible = visible;
        }
    }

    pub fn hide(&self) {
        self.set_visible(false);
        let _ = self.send_action(WindowAction::Hide);
    }

    /// Ask for the window, waiting for the UI to be ready to hear it.
    ///
    /// A second launch can land before `WindowController` has installed its
    /// receiver, and that request must not be dropped - to the user it looks
    /// like clicking the app did nothing at all. Retries briefly, then gives
    /// up rather than blocking a thread forever if the UI never mounts.
    pub fn request_show(&self) {
        const ATTEMPTS: u32 = 100;
        const INTERVAL: std::time::Duration = std::time::Duration::from_millis(50);

        for _ in 0..ATTEMPTS {
            if self.send_action(WindowAction::Show) {
                self.set_visible(true);
                return;
            }
            std::thread::sleep(INTERVAL);
        }

        eprintln!("⚠️ Could not deliver a show request: the window never became ready");
    }
}

pub static WINDOW_MANAGER: Lazy<WindowManager> = Lazy::new(|| WindowManager::new());
