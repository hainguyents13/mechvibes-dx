use crate::debug_print;
use crate::state::config::AppConfig;
use dioxus::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

/// Hook that gets the global config from context
/// Config is loaded once at app startup and shared across all pages
/// This prevents config reset on page navigation
pub fn use_fresh_config() -> Signal<AppConfig> {
    // Get global config from context (provided in ui.rs app())
    use_context::<Signal<AppConfig>>()
}

/// Creates a config updater function that loads fresh config, applies changes, and saves
pub fn create_config_updater(
    config_signal: Signal<AppConfig>,
) -> Rc<dyn Fn(Box<dyn FnOnce(&mut AppConfig)>)> {
    let signal_ref = Rc::new(RefCell::new(config_signal));
    Rc::new(move |updater: Box<dyn FnOnce(&mut AppConfig)>| {
        // Always re-read from disk immediately before mutating: `save()`
        // rewrites the entire struct, so applying an edit to a copy captured
        // earlier would silently revert whatever another path wrote in between.
        let old_config = AppConfig::load();
        let mut new_config = old_config.clone();

        updater(&mut new_config);

        let changed = !new_config.data_equals(&old_config);

        // Only write to disk when something actually changed - callers include
        // mount effects that re-assert the value they already hold, and saving
        // those unconditionally rewrote the config continuously.
        if changed {
            new_config.last_updated = chrono::Utc::now();
            match new_config.save() {
                Ok(_) => debug_print!("✅ [config_utils] Config saved successfully"),
                Err(e) => eprintln!("❌ [config_utils] Failed to save config: {}", e),
            }
        }

        // Publish to the signal regardless of whether *this* call changed
        // anything. The value on disk may already differ from the signal
        // because another path (e.g. `AudioContext::set_*`, which persists on
        // its own) just wrote it; skipping the update here would leave the UI
        // rendering a stale value - that is what made the mute button appear
        // to do nothing after its state had already been applied.
        // Compared with `data_equals` (ignores `last_updated`) so a timestamp
        // bump alone can't trigger a pointless re-render.
        let signal_is_stale = !signal_ref.borrow().peek().data_equals(&new_config);
        if signal_is_stale {
            signal_ref.borrow_mut().set(new_config);
        }
    })
}

/// Hook for managing configuration state with automatic updates
///
/// Returns a tuple of (config_signal, update_config_fn)
/// The update function can be used to make atomic config updates
pub fn use_config() -> (
    Signal<AppConfig>,
    Rc<dyn Fn(Box<dyn FnOnce(&mut AppConfig)>)>,
) {
    // Use fresh config that automatically reloads from file
    let config = use_fresh_config();
    let update_config = create_config_updater(config);
    (config, update_config)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Stands in for the config file plus the shared `Signal<AppConfig>` that
    /// every page reads. The real `create_config_updater` needs a Dioxus
    /// runtime, so the two pieces of state it coordinates are modelled
    /// directly: `disk` is what `AppConfig::load()`/`save()` see, `signal` is
    /// the context value components render from.
    struct Shared {
        disk: AppConfig,
        signal: AppConfig,
    }

    impl Shared {
        fn new() -> Self {
            let config = AppConfig::default();
            Self { disk: config.clone(), signal: config }
        }

        /// Mirrors `create_config_updater`: re-read disk, mutate, save if the
        /// data changed, then publish to the signal.
        fn update_config(&mut self, mutate: impl FnOnce(&mut AppConfig)) {
            let old = self.disk.clone();
            let mut new = old.clone();
            mutate(&mut new);
            if !new.data_equals(&old) {
                self.disk = new.clone();
            }
            self.signal = new;
        }

        /// Mirrors `AudioContext::set_volume` -> `persist_if_changed`: writes
        /// straight to disk and deliberately does not touch the signal.
        fn audio_set_volume(&mut self, volume: f32) {
            if self.disk.volume != volume {
                self.disk.volume = volume;
            }
        }

        /// Mirrors `AudioContext::set_sound_enabled`: same disk-only write.
        fn audio_set_sound_enabled(&mut self, enabled: bool) {
            if self.disk.enable_sound != enabled {
                self.disk.enable_sound = enabled;
            }
        }
    }

    /// Mirrors the startup update check, which loads the config *before* a
    /// network round trip and saves it *after*. Any field the user changed
    /// while the request was in flight is inside the stale struct and gets
    /// rewritten to its old value.
    struct StartupUpdateCheck {
        config_read_before_the_request: AppConfig,
    }

    impl StartupUpdateCheck {
        fn begin(shared: &Shared) -> Self {
            Self { config_read_before_the_request: shared.disk.clone() }
        }

        fn finish_and_save(self, shared: &mut Shared, last_check: u64) {
            // The fix: discard the pre-request copy and re-read, so only this
            // path's own field is carried into the write.
            drop(self.config_read_before_the_request);
            let mut config = shared.disk.clone();
            config.auto_update.last_check = Some(last_check);
            shared.disk = config;
        }
    }

    /// Mounting `HomePage` seeds its local slider signal from the shared
    /// config and then immediately runs `use_effect(|| ctx.set_volume(v))`,
    /// which writes that seeded value straight back to disk.
    fn mount_home_page(shared: &mut Shared) -> f32 {
        // `let mut volume = use_signal(|| config().volume)` - seeded from the
        // shared signal, not from disk.
        let slider = shared.signal.volume;
        // The mount effect re-asserts it into the audio engine, which persists.
        shared.audio_set_volume(slider);
        slider
    }

    /// The user-reported sequence: adjust the Home volume, switch tabs, change
    /// the theme, come back to find the volume reverted.
    ///
    /// The startup update check loads the config, awaits a network request,
    /// and then saves the struct it read *before* the request. `save()`
    /// rewrites every field, so a volume (or theme, or mute) change made while
    /// the request was in flight is reverted to the value it had at launch.
    ///
    /// The tab switch is what makes it reproducible rather than random:
    /// `HomePage`'s debounced save lives in a `spawn`ed task registered in the
    /// scope's `spawned_tasks`. Navigating away calls `Runtime::remove_scope`,
    /// which drains that set through `remove_task`, cancelling the pending
    /// write mid-`Delay`. Only `AudioContext::set_volume`'s immediate write
    /// remains on disk - and that is exactly the write the stale save
    /// overwrites.
    #[test]
    fn a_setting_changed_during_the_startup_update_check_is_not_reverted() {
        let mut shared = Shared::new();
        let original_volume = shared.disk.volume;
        let new_volume = 0.35;
        assert_ne!(original_volume, new_volume, "the test must actually move the slider");

        // The app launches and the startup check reads the config, then blocks
        // on the GitHub request for as long as the network takes.
        let check = StartupUpdateCheck::begin(&shared);

        // While that request is in flight the user drags the volume slider.
        // The mount effect persists it through the audio context immediately.
        shared.audio_set_volume(new_volume);

        // The user switches tabs, cancelling the debounced `update_config`
        // that would otherwise have republished the volume to the signal.

        // The user picks a theme on the other tab.
        shared.update_config(|config| {
            config.theme = crate::libs::theme::Theme::BuiltIn(
                crate::libs::theme::BuiltInTheme::Dark,
            );
        });

        // The update check finally comes back and saves the struct it read at
        // launch, stamped with the new check time.
        check.finish_and_save(&mut shared, 1_700_000_000);

        assert_eq!(
            shared.disk.volume,
            new_volume,
            "the volume changed during the update check must survive it"
        );
        assert_eq!(
            shared.disk.theme,
            crate::libs::theme::Theme::BuiltIn(crate::libs::theme::BuiltInTheme::Dark),
            "and so must the theme"
        );
        assert_eq!(
            shared.disk.auto_update.last_check,
            Some(1_700_000_000),
            "while the update check still records its own result"
        );

        // Returning to Home must show the volume the user set, not the one the
        // stale save restored.
        assert_eq!(
            mount_home_page(&mut shared),
            new_volume,
            "Home must render the volume the user set"
        );
    }

    /// Muting from the tray menu goes through `AudioContext::set_sound_enabled`,
    /// which writes the file without publishing. Without an accompanying
    /// `update_config` the window keeps rendering the pre-toggle mute icon and
    /// leaves the volume sliders enabled until an unrelated write republishes.
    #[test]
    fn the_tray_mute_toggle_publishes_to_the_shared_signal() {
        let mut shared = Shared::new();
        assert!(shared.signal.enable_sound, "sound starts enabled");

        // Tray click: notify the engine (persists), then publish.
        shared.audio_set_sound_enabled(false);
        shared.update_config(|config| {
            config.enable_sound = false;
        });

        assert!(!shared.disk.enable_sound, "the mute reached disk");
        assert!(
            !shared.signal.enable_sound,
            "the window must re-render muted rather than wait for another writer"
        );
    }

    /// Dragging the slider must publish to the shared signal synchronously,
    /// not only from inside the debounced task. The task is cancelled whenever
    /// the user navigates away within the 500ms window, and a signal left
    /// holding the pre-drag volume re-seeds the slider on remount.
    #[test]
    fn dragging_the_slider_publishes_before_the_debounce_can_be_cancelled() {
        let mut shared = Shared::new();

        // The mount effect pushes the value into the audio engine, which
        // persists it...
        shared.audio_set_volume(0.2);
        // ...and HomePage publishes it to the shared signal immediately,
        // rather than waiting for the debounced write.
        shared.update_config(|config| {
            config.volume = 0.2;
        });

        // The debounced task is now cancelled by the tab switch.

        assert_eq!(shared.disk.volume, 0.2, "the audio path persisted the new volume");
        assert_eq!(
            shared.signal.volume,
            0.2,
            "the shared signal must not lag behind what was persisted"
        );
        assert_eq!(
            mount_home_page(&mut shared),
            0.2,
            "so a remount re-seeds the slider with the value the user chose"
        );
    }
}
