use crate::debug_print;
use crate::state::config::AppConfig;
use crate::state::config_writer;
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

/// Creates a config updater function that submits a mutation to the single
/// writer and republishes the result to this scope's config signal.
///
/// The mutation is applied by `config_writer::apply` to the authoritative
/// state, so a field another subsystem changed in the meantime is already
/// present and survives. Nothing here holds an `AppConfig` across the call.
pub fn create_config_updater(
    config_signal: Signal<AppConfig>,
) -> Rc<dyn Fn(Box<dyn FnOnce(&mut AppConfig)>)> {
    let signal_ref = Rc::new(RefCell::new(config_signal));
    Rc::new(move |updater: Box<dyn FnOnce(&mut AppConfig)>| {
        if config_writer::apply(updater) {
            debug_print!("✅ [config_utils] Config saved successfully");
        }

        // Publish regardless of whether *this* call changed anything. The
        // authority may already differ from this signal because another path
        // wrote it (the tray mute, or the engine's Ctrl+Alt+M); skipping the
        // update here would leave the UI rendering a stale value - that is what
        // made the mute button appear to do nothing after its state had already
        // been applied.
        //
        // Compared with `data_equals` (which ignores `last_updated`) so a
        // timestamp bump alone can't trigger a pointless re-render.
        let latest = config_writer::current();
        let signal_is_stale = !signal_ref.borrow().peek().data_equals(&latest);
        if signal_is_stale {
            signal_ref.borrow_mut().set(latest);
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

    /// Stands in for the single writer's authoritative config plus the shared
    /// `Signal<AppConfig>` that every page renders from. The real
    /// `create_config_updater` needs a Dioxus runtime, so the two pieces of
    /// state it coordinates are modelled directly: `authority` is what
    /// `config_writer` owns and persists, `signal` is the context value.
    struct Shared {
        authority: AppConfig,
        signal: AppConfig,
    }

    impl Shared {
        fn new() -> Self {
            let config = AppConfig::default();
            Self { authority: config.clone(), signal: config }
        }

        /// Mirrors `config_writer::apply`: mutate the authoritative state,
        /// persist only a real change.
        fn apply(&mut self, mutate: impl FnOnce(&mut AppConfig)) {
            let before = self.authority.clone();
            mutate(&mut self.authority);
            if self.authority.data_equals(&before) {
                self.authority = before;
            }
        }

        /// Mirrors `create_config_updater`: submit the mutation to the writer,
        /// then republish the writer's state to this scope's signal.
        fn update_config(&mut self, mutate: impl FnOnce(&mut AppConfig)) {
            self.apply(mutate);
            self.signal = self.authority.clone();
        }

        /// Mirrors `AudioContext::set_volume`: submits a mutation and
        /// deliberately does not touch the signal itself.
        fn audio_set_volume(&mut self, volume: f32) {
            self.apply(|config| {
                config.volume = volume;
            });
        }

        /// Mirrors `AudioContext::set_sound_enabled`: same, without publishing.
        fn audio_set_sound_enabled(&mut self, enabled: bool) {
            self.apply(|config| {
                config.enable_sound = enabled;
            });
        }

        /// Mirrors the poll loop in `libs/ui.rs`, which republishes whenever the
        /// writer reports a change - the path by which an off-thread write (the
        /// Ctrl+Alt+M hotkey, the update checker) reaches the window.
        fn poll_for_config_changes(&mut self) {
            if !self.signal.data_equals(&self.authority) {
                self.signal = self.authority.clone();
            }
        }
    }

    /// Mirrors the startup update check: it begins before a network round trip
    /// and records its result after. It carries no config across that gap -
    /// only the value it intends to write - which is what makes it harmless.
    struct StartupUpdateCheck;

    impl StartupUpdateCheck {
        fn begin(_shared: &Shared) -> Self {
            Self
        }

        fn finish_and_save(self, shared: &mut Shared, last_check: u64) {
            shared.apply(|config| {
                config.auto_update.last_check = Some(last_check);
            });
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
        let original_volume = shared.authority.volume;
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
            shared.authority.volume,
            new_volume,
            "the volume changed during the update check must survive it"
        );
        assert_eq!(
            shared.authority.theme,
            crate::libs::theme::Theme::BuiltIn(crate::libs::theme::BuiltInTheme::Dark),
            "and so must the theme"
        );
        assert_eq!(
            shared.authority.auto_update.last_check,
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

        assert!(!shared.authority.enable_sound, "the mute reached disk");
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

        assert_eq!(shared.authority.volume, 0.2, "the audio path persisted the new volume");
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

    /// Ctrl+Alt+M is handled on the audio engine thread, which cannot touch the
    /// shared signal - it uses Dioxus's default non-`Send` storage. The write
    /// therefore reaches the window only through the UI's poll of the writer's
    /// change counter. Until that poll existed, the app muted while the window
    /// kept rendering the unmuted icon.
    #[test]
    fn a_write_from_the_engine_thread_reaches_the_window() {
        let mut shared = Shared::new();
        assert!(shared.signal.enable_sound, "sound starts enabled");

        // The hotkey handler flips the flag inside the mutation, so the value
        // it negates is the writer's, not a copy read earlier.
        shared.apply(|config| {
            config.enable_sound = !config.enable_sound;
        });

        assert!(!shared.authority.enable_sound, "the hotkey muted the app");
        assert!(
            shared.signal.enable_sound,
            "the engine thread cannot publish, so the signal is briefly behind"
        );

        shared.poll_for_config_changes();

        assert!(
            !shared.signal.enable_sound,
            "the UI poll must republish, or the window renders the wrong mute icon"
        );
    }

    /// The poll must not republish when the value it would push is the one
    /// already rendered - a UI-originated write publishes synchronously, and a
    /// second push would re-render the whole tree for no visible difference.
    #[test]
    fn the_poll_does_not_republish_a_ui_write_that_already_published() {
        let mut shared = Shared::new();

        shared.update_config(|config| {
            config.volume = 0.42;
        });
        let published = shared.signal.clone();

        shared.poll_for_config_changes();

        assert!(
            shared.signal.data_equals(&published),
            "the poll must find nothing to do after a UI write"
        );
    }
}
