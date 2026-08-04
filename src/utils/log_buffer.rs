//! In-RAM ring buffer of the app's recent log lines, for the Debug section in
//! Settings.
//!
//! # Why this exists
//!
//! Release builds are `#![windows_subsystem = "windows"]`, so an installed app
//! has no console at all. A user hitting a bug had literally no way to send
//! back what the app printed. This keeps the most recent lines in memory so
//! the UI can show them and the user can export them on request.
//!
//! # Rules this module enforces
//!
//! - **Nothing is written to disk** until the user presses Export. The buffer
//!   is memory-only and fixed size, so an app left running for a week costs
//!   the same as one just launched.
//! - **No key identities, ever.** Verbose mode routes per-keystroke timing
//!   lines here with the key masked (see [`mask_key_identities`]). Timing is
//!   what latency reports need; which key was struck is not, and a log the
//!   user is about to mail somewhere must never carry it.
//!
//! # Cost on the logging path
//!
//! A `Mutex<VecDeque>` with a short critical section (timestamp format, push,
//! pop-front). That is acceptable here because the app does not log in steady
//! state: the audio engine's `handle_key_event`/`handle_mouse_event` and the
//! worker-host read loop contain no logging on the per-event path, so with the
//! viewer closed and verbose off, typing produces zero pushes. The call sites
//! that do log are startup, config changes, device switches, soundpack loads
//! and errors - all of them rare and none of them latency-critical.

use std::collections::VecDeque;
use std::sync::atomic::{ AtomicBool, AtomicU64, Ordering };
use std::sync::{ Mutex, OnceLock };

/// How many lines are kept. At a typical ~120 bytes per line this is roughly
/// 240 KB worst case, which is a rounding error against the app's soundpack
/// buffers and enough history to cover a whole session's startup plus a
/// reproduction of whatever went wrong.
pub const CAPACITY: usize = 2000;

/// How many lines the live viewer renders. The buffer keeps far more; the
/// viewer is a terminal-style tail, and the export is the full record.
pub const VIEWER_LINES: usize = 100;

/// The buffer itself. `Mutex` rather than a channel + writer thread because
/// the UI has to read the contents synchronously to render them, and the
/// pushes are rare enough that contention is not a real concern.
static BUFFER: OnceLock<Mutex<VecDeque<String>>> = OnceLock::new();

/// Bumped on every push. The viewer polls this instead of cloning the buffer
/// each tick, so an idle app does no work beyond one atomic load per poll.
static GENERATION: AtomicU64 = AtomicU64::new(0);

/// Whether verbose (per-keystroke timing) lines are routed into the buffer.
/// Deliberately not persisted: it resets to off on every launch so nobody
/// can accidentally ship a build with it left on.
static VERBOSE: AtomicBool = AtomicBool::new(false);

fn buffer() -> &'static Mutex<VecDeque<String>> {
    BUFFER.get_or_init(|| Mutex::new(VecDeque::with_capacity(CAPACITY)))
}

/// Wall-clock `HH:MM:SS.mmm` for the moment a line was captured.
///
/// Local time on purpose: the reader of an exported log is the user or the
/// maintainer correlating it against "it froze around 2pm", and UTC would make
/// everyone do the arithmetic themselves.
fn timestamp_now() -> String {
    format_timestamp(chrono::Local::now())
}

/// Split from [`timestamp_now`] so the format is testable without depending on
/// what the clock happens to say.
fn format_timestamp(at: chrono::DateTime<chrono::Local>) -> String {
    use chrono::Timelike;
    format!(
        "{:02}:{:02}:{:02}.{:03}",
        at.hour(),
        at.minute(),
        at.second(),
        at.nanosecond() / 1_000_000
    )
}

/// Appends one line, dropping the oldest when full.
///
/// The console output the caller already produced is untouched: this is a tee,
/// never a replacement, so `dx serve` shows exactly what it always did.
pub fn push(line: &str) {
    let Ok(mut buffer) = buffer().lock() else {
        // A poisoned lock means some other thread panicked mid-push. Losing
        // debug lines is strictly better than propagating that panic into
        // whatever path was merely trying to log.
        return;
    };

    // Multi-line messages become multiple entries so the viewer's "last 100
    // lines" and the export both count what a reader would call a line.
    for part in line.split('\n') {
        if buffer.len() >= CAPACITY {
            buffer.pop_front();
        }
        buffer.push_back(format!("{} {}", timestamp_now(), part.trim_end_matches('\r')));
    }

    drop(buffer);
    GENERATION.fetch_add(1, Ordering::Release);
}

/// Push counter. The viewer compares this against what it last rendered, so
/// an idle app re-renders nothing.
pub fn generation() -> u64 {
    GENERATION.load(Ordering::Acquire)
}

/// The most recent `count` lines, oldest first - what the live viewer draws.
pub fn recent(count: usize) -> Vec<String> {
    let Ok(buffer) = buffer().lock() else {
        return Vec::new();
    };
    let skip = buffer.len().saturating_sub(count);
    buffer.iter().skip(skip).cloned().collect()
}

/// Every buffered line, oldest first - what Export writes.
pub fn snapshot() -> Vec<String> {
    let Ok(buffer) = buffer().lock() else {
        return Vec::new();
    };
    buffer.iter().cloned().collect()
}

/// How many lines are currently held.
pub fn len() -> usize {
    buffer().lock().map(|buffer| buffer.len()).unwrap_or(0)
}

/// Whether verbose per-keystroke timing lines are being captured.
#[inline]
pub fn verbose_enabled() -> bool {
    VERBOSE.load(Ordering::Relaxed)
}

/// Turns verbose capture on or off. Not persisted anywhere.
///
/// Opening this door is only half the job: the per-keystroke timing lines come
/// from the trace facility, whose points are inert unless tracing is enabled.
/// Before this call also drove `trace::set_runtime_tracing`, turning the toggle
/// on produced nothing at all unless the user had launched the app with
/// `MECHVIBES_TRACE=1` - which an installed build gives them no way to do.
pub fn set_verbose(on: bool) {
    VERBOSE.store(on, Ordering::Relaxed);
    // Order matters on the way up: the producer must be live before the first
    // line can arrive, and on the way down the flag above has already closed
    // `push_verbose`, so nothing slips through while the producer stops.
    crate::libs::trace::set_runtime_tracing(on);
    push(if on {
        "🔊 Verbose logging ON - per-keystroke timings will be captured with key identities masked"
    } else {
        "🔇 Verbose logging OFF"
    });
}

/// Captures a per-keystroke timing line, masking the key identity first.
///
/// This is the only door verbose lines come through, so the masking cannot be
/// bypassed by a caller that forgets: [`mask_key_identities`] runs here rather
/// than at the call site.
pub fn push_verbose(line: &str) {
    if !verbose_enabled() {
        return;
    }
    push(&mask_key_identities(line));
}

/// Replaces the value of any `key=...` field with `***`.
///
/// The privacy rule is absolute: a buffer the user may export must never
/// contain which keys were pressed, whatever toggles are set. Timing is the
/// diagnostic payload, and it survives masking intact.
///
/// Handles the trace module's own format (`key=KeyA` padded to a column) as
/// well as the bare `key=X` a future call site might produce.
pub fn mask_key_identities(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut rest = line;

    while let Some(at) = rest.find("key=") {
        // Only mask a real field, not the tail of a word like `monkey=`.
        let is_field_start =
            at == 0 || !rest[..at].ends_with(|c: char| c.is_alphanumeric() || c == '_');

        let (head, tail) = rest.split_at(at + "key=".len());
        out.push_str(head);
        rest = tail;

        if !is_field_start {
            continue;
        }

        // The value runs to the next whitespace; the padding the trace format
        // adds is whitespace too, so it is preserved rather than swallowed.
        let value_end = rest.find(char::is_whitespace).unwrap_or(rest.len());
        out.push_str("***");
        rest = &rest[value_end..];
    }

    out.push_str(rest);
    out
}

/// Header written at the top of an export, so a log arriving in a bug report
/// carries the context that would otherwise have to be asked for.
pub fn export_header() -> String {
    format!(
        "MechvibesDX log export\n\
         App version: {}\n\
         OS: {} ({})\n\
         Exported at: {}\n\
         Verbose logging: {}\n\
         Lines captured: {} (buffer holds up to {})\n\
         {}\n",
        crate::utils::constants::APP_VERSION,
        std::env::consts::OS,
        std::env::consts::ARCH,
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
        if verbose_enabled() {
            "on (key identities masked)"
        } else {
            "off"
        },
        len(),
        CAPACITY,
        "-".repeat(60)
    )
}

/// Full export body: header followed by every buffered line.
pub fn export_contents() -> String {
    let mut out = export_header();
    for line in snapshot() {
        out.push_str(&line);
        out.push('\n');
    }
    out
}

/// Filename for an export, stamped so repeated exports never collide.
fn export_file_name(at: chrono::DateTime<chrono::Local>) -> String {
    format!("mechvibes-log-{}.txt", at.format("%Y%m%d-%H%M%S"))
}

/// Writes the whole buffer to a timestamped file under
/// `%TEMP%/mechvibes-logs` and returns the path.
///
/// The temp directory rather than the config directory: this is a transient
/// artifact the user is about to attach to a bug report, and it should not
/// accumulate inside the app's own data.
pub fn export_to_file() -> Result<std::path::PathBuf, String> {
    let dir = std::env::temp_dir().join("mechvibes-logs");
    std::fs::create_dir_all(&dir).map_err(|e| format!("Could not create {}: {}", dir.display(), e))?;

    let path = dir.join(export_file_name(chrono::Local::now()));
    std::fs::write(&path, export_contents()).map_err(|e|
        format!("Could not write {}: {}", path.display(), e)
    )?;

    Ok(path)
}

/// Opens the folder containing `path` in the system file manager, selecting
/// the file where the platform supports it.
///
/// Best effort by design: a failure here is not a failed export, because the
/// file is already on disk and the UI shows its full path regardless.
pub fn reveal_in_file_manager(path: &std::path::Path) {
    #[cfg(target_os = "windows")]
    let spawned = std::process::Command::new("explorer")
        .arg(format!("/select,{}", path.display()))
        .spawn();

    #[cfg(target_os = "macos")]
    let spawned = std::process::Command::new("open").arg("-R").arg(path).spawn();

    #[cfg(target_os = "linux")]
    let spawned = std::process::Command
        ::new("xdg-open")
        .arg(path.parent().unwrap_or(path))
        .spawn();

    if let Err(e) = spawned {
        crate::debug_eprint!("⚠️ Could not open the log folder: {}", e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The buffer is process-global, so tests that push must not run
    /// concurrently with each other. Each takes this lock rather than relying
    /// on the harness's thread scheduling.
    ///
    /// The lock is NOT enough on its own: tests in OTHER modules exercise app
    /// code that logs through `always_print!`, so foreign lines can land in
    /// the buffer at any moment. Order-sensitive tests therefore tag their own
    /// lines with a unique marker and assert on the filtered subset, never on
    /// absolute buffer positions.
    static TEST_LOCK: Mutex<()> = Mutex::new(());

    fn reset() {
        buffer().lock().unwrap().clear();
        VERBOSE.store(false, Ordering::Relaxed);
    }

    /// The indices of this test's own lines, in buffer order.
    fn our_indices(marker: &str) -> Vec<usize> {
        snapshot()
            .iter()
            .filter_map(|line| line.split(marker).nth(1)?.trim().parse().ok())
            .collect()
    }

    #[test]
    fn lines_are_kept_in_order_with_a_timestamp() {
        let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        reset();

        push("ordertest 0");
        push("ordertest 1");

        let ours = our_indices("ordertest ");
        assert_eq!(ours, vec![0, 1], "own lines must be kept in push order");
        let first = snapshot()
            .into_iter()
            .find(|l| l.contains("ordertest 0"))
            .expect("pushed line must be present");
        // HH:MM:SS.mmm prefix.
        assert!(first.chars().take(2).all(|c| c.is_ascii_digit()), "{}", first);
    }

    #[test]
    fn the_buffer_rotates_and_never_grows_past_capacity() {
        let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        reset();

        // Overfill by a clear margin so both the cap and the drop order are
        // exercised, not just the boundary.
        for i in 0..CAPACITY + 50 {
            push(&format!("rotcap {}", i));
        }

        assert_eq!(len(), CAPACITY, "a long-running app must not grow without bound");

        let ours = our_indices("rotcap ");
        assert!(ours.windows(2).all(|w| w[0] < w[1]), "retained lines must keep push order");
        assert_eq!(
            *ours.last().unwrap(),
            CAPACITY + 49,
            "the newest line must always survive rotation"
        );
        // At least the oldest 50 are gone; foreign lines from parallel tests
        // can push a few more of ours out, never fewer.
        assert!(*ours.first().unwrap() >= 50, "the oldest lines must be dropped first");
    }

    #[test]
    fn recent_returns_the_tail_oldest_first() {
        let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        reset();

        for i in 0..10 {
            push(&format!("tailtest {}", i));
        }

        // Ask for a window big enough that our tail is inside it even if a few
        // foreign lines landed after ours.
        let tail = recent(6);
        let ours: Vec<usize> = tail
            .iter()
            .filter_map(|line| line.split("tailtest ").nth(1)?.trim().parse().ok())
            .collect();
        assert!(ours.len() >= 3, "tail must contain our newest lines: {:?}", tail);
        assert!(ours.windows(2).all(|w| w[0] < w[1]), "tail must be oldest first");
        assert_eq!(*ours.last().unwrap(), 9, "tail must end at the newest line");
    }

    #[test]
    fn recent_asking_for_more_than_exists_returns_everything() {
        let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        reset();

        push("loneline 0");
        let tail = recent(VIEWER_LINES);
        assert!(
            tail.iter().any(|l| l.contains("loneline 0")),
            "a window larger than the buffer must still include everything: {:?}",
            tail
        );
    }

    #[test]
    fn a_multi_line_message_becomes_multiple_entries() {
        // Otherwise the viewer's "last 100 lines" would be a lie for any
        // message carrying an embedded newline, and one entry would render as
        // several rows.
        let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        reset();

        push("alpha\nbeta");
        let lines = snapshot();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].ends_with("alpha"), "{}", lines[0]);
        assert!(lines[1].ends_with("beta"), "{}", lines[1]);
    }

    #[test]
    fn timestamps_are_zero_padded_hms_with_milliseconds() {
        use chrono::TimeZone;

        let at = chrono::Local.with_ymd_and_hms(2026, 1, 2, 3, 4, 5).unwrap();
        assert_eq!(format_timestamp(at), "03:04:05.000");

        let at = chrono::Local.with_ymd_and_hms(2026, 1, 2, 13, 45, 59).unwrap();
        assert_eq!(format_timestamp(at), "13:45:59.000");
    }

    #[test]
    fn every_push_moves_the_generation() {
        let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        reset();

        let before = generation();
        push("something");
        assert!(generation() > before, "the viewer polls this to know it must redraw");
    }

    #[test]
    fn key_identities_are_masked_out_of_trace_lines() {
        // The privacy rule: timing survives, the key does not.
        let line = "🔬 TRACE key=KeyA         total=     3.1ms  worker->engine=0.2ms";
        let masked = mask_key_identities(line);

        assert!(!masked.contains("KeyA"), "{}", masked);
        assert!(masked.contains("key=***"), "{}", masked);
        assert!(masked.contains("total=     3.1ms"), "timing must survive: {}", masked);
        assert!(masked.contains("worker->engine=0.2ms"), "{}", masked);
    }

    #[test]
    fn masking_covers_every_key_field_on_a_line() {
        let masked = mask_key_identities("key=KeyA then key=ControlLeft done");
        assert_eq!(masked, "key=*** then key=*** done");
    }

    #[test]
    fn masking_leaves_lookalike_fields_alone() {
        // `monkey=` ends in "key=" but is not the field being protected;
        // rewriting it would corrupt unrelated messages.
        let masked = mask_key_identities("monkey=banana");
        assert_eq!(masked, "monkey=banana");
    }

    #[test]
    fn masking_handles_a_key_field_at_end_of_line() {
        assert_eq!(mask_key_identities("timing key=KeyZ"), "timing key=***");
    }

    #[test]
    fn verbose_lines_are_dropped_entirely_while_verbose_is_off() {
        let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        reset();

        push_verbose("🔬 TRACE key=KeyA total=3.1ms");
        assert_eq!(len(), 0, "verbose off must cost nothing and capture nothing");
    }

    #[test]
    fn verbose_lines_reach_the_buffer_masked_when_on() {
        let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        reset();

        VERBOSE.store(true, Ordering::Relaxed);
        push_verbose("🔬 TRACE key=KeyA total=3.1ms");

        let lines = snapshot();
        assert_eq!(lines.len(), 1);
        assert!(!lines[0].contains("KeyA"), "raw key reached the buffer: {}", lines[0]);
        assert!(lines[0].contains("key=***"), "{}", lines[0]);
        assert!(lines[0].contains("total=3.1ms"), "{}", lines[0]);

        VERBOSE.store(false, Ordering::Relaxed);
    }

    #[test]
    fn no_raw_key_code_can_reach_the_buffer_through_the_verbose_door() {
        // Guards the hard rule across a spread of real key codes rather than
        // one example, since this is the promise made to the user.
        let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        reset();
        VERBOSE.store(true, Ordering::Relaxed);

        for code in ["KeyA", "KeyZ", "ControlLeft", "MouseLeft", "Numpad7"] {
            push_verbose(&format!("🔬 TRACE key={} total=1.0ms", code));
        }

        let joined = snapshot().join("\n");
        for code in ["KeyA", "KeyZ", "ControlLeft", "MouseLeft", "Numpad7"] {
            assert!(!joined.contains(code), "`{}` leaked into the buffer:\n{}", code, joined);
        }

        VERBOSE.store(false, Ordering::Relaxed);
    }

    #[test]
    fn the_export_carries_the_header_and_every_line() {
        let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        reset();

        for i in 0..5 {
            push(&format!("event {}", i));
        }

        let contents = export_contents();

        // Header: the context a bug report would otherwise have to ask for.
        assert!(contents.contains("MechvibesDX log export"), "{}", contents);
        assert!(
            contents.contains(&format!("App version: {}", crate::utils::constants::APP_VERSION)),
            "{}",
            contents
        );
        assert!(contents.contains(std::env::consts::OS), "{}", contents);
        assert!(contents.contains("Verbose logging: off"), "{}", contents);
        assert!(contents.contains("Lines captured: 5"), "{}", contents);

        // Body: all of it, not just the viewer's window.
        for i in 0..5 {
            assert!(contents.contains(&format!("event {}", i)), "missing event {}", i);
        }
    }

    #[test]
    fn the_export_contains_more_than_the_viewer_shows() {
        // The whole point of Export: the viewer is a 100-line tail, the file
        // is the entire buffer.
        let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        reset();

        let total = VIEWER_LINES + 250;
        for i in 0..total {
            push(&format!("event {}", i));
        }

        assert_eq!(recent(VIEWER_LINES).len(), VIEWER_LINES);

        let contents = export_contents();
        // A line the viewer would never have shown must still be exported.
        assert!(contents.contains("event 0"), "export must not be limited to the viewer window");
        assert!(contents.contains(&format!("event {}", total - 1)));
        assert!(contents.contains(&format!("Lines captured: {}", total)));
    }

    #[test]
    fn the_export_reports_verbose_state_in_its_header() {
        let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        reset();

        VERBOSE.store(true, Ordering::Relaxed);
        assert!(export_header().contains("Verbose logging: on (key identities masked)"));

        VERBOSE.store(false, Ordering::Relaxed);
        assert!(export_header().contains("Verbose logging: off"));
    }

    #[test]
    fn export_file_names_are_timestamped_and_do_not_collide() {
        use chrono::TimeZone;

        let at = chrono::Local.with_ymd_and_hms(2026, 8, 4, 15, 30, 45).unwrap();
        assert_eq!(export_file_name(at), "mechvibes-log-20260804-153045.txt");

        let later = chrono::Local.with_ymd_and_hms(2026, 8, 4, 15, 30, 46).unwrap();
        assert_ne!(export_file_name(at), export_file_name(later));
    }

    /// The steady-state promise, checked against the shipping source rather
    /// than a description of it: with verbose off, an input event must not
    /// reach a single logging call. Any push per keystroke would put a mutex
    /// on the audio path this app spent whole phases getting off.
    #[test]
    fn no_per_event_code_path_logs() {
        const ENGINE: &str = include_str!("../libs/audio/engine.rs");
        const WORKER_HOST: &str = include_str!("../libs/input_worker_host.rs");

        // Runtime code only - the test modules mention these names themselves.
        let engine = ENGINE.split("#[cfg(test)]").next().unwrap();
        let worker_host = WORKER_HOST.split("#[cfg(test)]").next().unwrap();

        let logging = ["always_print!", "always_eprint!", "debug_print!", "debug_eprint!"];

        // Every keystroke and click runs through these two.
        for handler in ["fn handle_key_event(", "fn handle_mouse_event("] {
            let body = engine
                .split(handler)
                .nth(1)
                .expect("handler must exist")
                .split("\n    fn ")
                .next()
                .unwrap();
            for macro_name in logging {
                assert!(
                    !body.contains(macro_name),
                    "{handler} must not log: a call per keystroke would take the buffer \
                     mutex on the audio path"
                );
            }
        }

        // The loop that reads one line per input event off the worker.
        let read_loop = worker_host
            .split("for line in BufReader::new(stdout).lines()")
            .nth(1)
            .expect("worker read loop must exist")
            .split("\n    reap(")
            .next()
            .unwrap();
        for macro_name in logging {
            assert!(
                !read_loop.contains(macro_name),
                "the worker read loop must not log per event: stalling this reader \
                 back-pressures the worker's message pump"
            );
        }
    }

    /// The reported bug, end to end through the real producer: flipping the
    /// Settings toggle must make an actual recorded trace point arrive in the
    /// buffer, with no environment variable involved.
    ///
    /// The earlier verbose tests all called `push_verbose` directly, which is
    /// why they passed while the feature was broken - they exercised the door
    /// and never checked that anything was on the other side of it.
    #[test]
    fn the_verbose_toggle_makes_real_trace_points_reach_the_buffer() {
        use crate::libs::trace::{ Point, record };

        let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        reset();

        assert!(
            std::env::var("MECHVIBES_TRACE").is_err(),
            "this test must prove the toggle works with no env var set"
        );

        set_verbose(true);

        // A full keystroke's worth of points, the same sequence the worker
        // host, engine and UI loop emit for one press.
        record(Point::WorkerSend, "KeyA", 0.0);
        record(Point::EngineDequeue, "KeyA", 0.0);
        record(Point::PlayedSound, "KeyA", 0.4);
        record(Point::UiEventSent, "KeyA", 0.0);
        record(Point::UiWrite, "KeyA", 0.0);

        // The writer is a separate thread; give it a moment to drain.
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        let mut captured = String::new();
        while std::time::Instant::now() < deadline {
            captured = snapshot().join("\n");
            if captured.contains("TRACE") {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(25));
        }

        assert!(
            captured.contains("TRACE"),
            "a recorded keystroke must reach the buffer once verbose is on:\n{captured}"
        );
        assert!(captured.contains("total="), "the timing is the payload:\n{captured}");
        // The masking invariant holds on this path too.
        assert!(!captured.contains("KeyA"), "the key identity leaked:\n{captured}");
        assert!(captured.contains("key=***"), "{captured}");

        set_verbose(false);
    }

    /// And the other direction: once off, further recorded points must not
    /// reach the buffer at all.
    #[test]
    fn turning_verbose_off_stops_the_flow() {
        use crate::libs::trace::{ Point, record };

        let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        reset();

        set_verbose(true);
        record(Point::PackLoad, "somepack", 12.0);

        // Let the writer drain whatever the enabled phase produced.
        std::thread::sleep(std::time::Duration::from_millis(300));

        set_verbose(false);
        // Clear everything the enabled phase left, including the toggle's own
        // "verbose OFF" line, so what follows is measured from empty.
        buffer().lock().unwrap().clear();
        let generation_before = generation();

        for _ in 0..200 {
            record(Point::PackLoad, "somepack", 12.0);
        }
        std::thread::sleep(std::time::Duration::from_millis(300));

        assert_eq!(len(), 0, "verbose off must capture nothing: {:?}", snapshot());
        assert_eq!(generation_before, generation(), "an idle app must not move the generation");
    }

    /// The same promise, checked behaviorally: pushing input-shaped verbose
    /// lines while verbose is off must leave the buffer untouched.
    #[test]
    fn typing_with_verbose_off_produces_no_pushes() {
        let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        reset();

        let generation_before = generation();
        for _ in 0..500 {
            push_verbose("🔬 TRACE key=KeyA total=2.0ms");
        }

        assert_eq!(len(), 0, "verbose off must capture nothing");
        assert_eq!(
            generation(),
            generation_before,
            "an idle app must not move the generation, or the viewer would redraw for nothing"
        );
    }

    /// End-to-end demonstration against the real producer, printing what a
    /// user would see in the Debug section. Ignored by default; run with
    /// `cargo test --release -- --ignored --nocapture demonstrate_verbose_toggle`.
    ///
    /// Exists because the original bug was invisible to assertions that only
    /// exercised `push_verbose`: this drives `trace::record` exactly as the
    /// worker host, engine and UI loop do, with no environment variable.
    #[test]
    #[ignore = "diagnostic: demonstrates the verbose toggle end to end"]
    fn demonstrate_verbose_toggle() {
        use crate::libs::trace::{ Point, record };

        let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        reset();

        println!(
            "MECHVIBES_TRACE env var: {:?}",
            std::env::var("MECHVIBES_TRACE").unwrap_or_else(|_| "<unset>".to_string())
        );

        println!("\n--- toggle OFF, simulating 50 keystrokes ---");
        for _ in 0..50 {
            record(Point::WorkerSend, "KeyA", 0.0);
            record(Point::UiWrite, "KeyA", 0.0);
        }
        std::thread::sleep(std::time::Duration::from_millis(300));
        println!("buffer lines captured: {} (expected 0)", len());

        println!("\n--- toggle ON, simulating 3 keystrokes ---");
        set_verbose(true);
        for key in ["KeyA", "KeyS", "KeyD"] {
            record(Point::WorkerSend, key, 0.0);
            record(Point::EngineDequeue, key, 0.0);
            record(Point::PlayedSound, key, 0.4);
            record(Point::UiEventSent, key, 0.0);
            record(Point::UiWrite, key, 0.0);
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
        println!("buffer now holds {} line(s):", len());
        for line in snapshot() {
            println!("  {}", line);
        }

        println!("\n--- toggle OFF again, simulating 50 more keystrokes ---");
        set_verbose(false);
        let after_off = len();
        for _ in 0..50 {
            record(Point::WorkerSend, "KeyA", 0.0);
            record(Point::UiWrite, "KeyA", 0.0);
        }
        std::thread::sleep(std::time::Duration::from_millis(300));
        println!("lines at toggle-off: {}, lines now: {} (must be equal)", after_off, len());
    }

    /// Prints a real export to stdout so the shipped format can be read by a
    /// human rather than inferred from assertions. Ignored by default; run
    /// with `cargo test -- --ignored --nocapture show_a_real_export`.
    #[test]
    #[ignore = "diagnostic: prints a sample export"]
    fn show_a_real_export() {
        let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        reset();

        push("🐛 Debug logging enabled");
        push("🚀 Initializing MechvibesDX...");
        push("📂 App root (from exe): D:\\mechvibes-dx\\target\\release");
        push("🎧 Audio engine thread started");
        push("🎮 Starting Raw Input worker process...");
        VERBOSE.store(true, Ordering::Relaxed);
        push_verbose("🔬 TRACE key=KeyA         total=     3.1ms  worker->engine=0.2ms");

        let path = export_to_file().expect("export must succeed");
        println!("--- exported to: {} ---", path.display());
        println!("{}", std::fs::read_to_string(&path).unwrap());
        println!("--- end ---");

        VERBOSE.store(false, Ordering::Relaxed);
    }

    #[test]
    fn exporting_writes_a_readable_file_containing_the_buffer() {
        let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        reset();

        push("a line that must survive the round trip");

        let path = export_to_file().expect("export must succeed");
        let written = std::fs::read_to_string(&path).expect("exported file must be readable");

        assert!(written.contains("MechvibesDX log export"));
        assert!(written.contains("a line that must survive the round trip"));

        let _ = std::fs::remove_file(&path);
    }
}
