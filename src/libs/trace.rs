//! Opt-in latency tracing for the input -> engine -> UI path.
//!
//! Enabled by setting `MECHVIBES_TRACE=1` before launching. When it is unset
//! (the shipping default) every entry point below is an atomic load that fails
//! immediately, so the hot paths pay a predictable branch and nothing else.
//!
//! # Why it is built this way
//!
//! The paths being measured are exactly the paths that must never block: the
//! audio engine loop, the worker-stdout reader, and the UI poll loop. Doing
//! I/O on any of them would create the very stall we are trying to find, so
//! recording a point only pushes a small `Copy` record onto an unbounded
//! channel; a dedicated writer thread owns all formatting and output.
//!
//! # What comes out
//!
//! One compact line per keystroke on stderr, so it lands in the `dx serve`
//! console the user is already watching, with each hop's duration and the slow
//! hop marked. A full per-hop log is written to a file at the same time for
//! offline analysis - the console line is the product, the file is the detail.

use std::sync::OnceLock;
use std::sync::atomic::{ AtomicBool, Ordering };
use std::time::Instant;

/// Master switch, read on every trace point. Set once at startup.
static ENABLED: AtomicBool = AtomicBool::new(false);

/// Sender to the writer thread. `None` until `init` runs with tracing on.
static SINK: OnceLock<crossbeam_channel::Sender<Record>> = OnceLock::new();

/// Process start, so every timestamp is a small monotonic offset rather than
/// an absolute clock read that would need formatting on the hot path.
static ORIGIN: OnceLock<Instant> = OnceLock::new();

/// A hop in the input -> sound -> pixel path. Ordered as the event travels, so
/// the writer can lay a keystroke's hops out in sequence.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Point {
    /// The worker host read a line off the worker's stdout and is about to put
    /// it on the channel the engine reads.
    WorkerSend,
    /// The engine's `select!` handed this event to the engine loop.
    EngineDequeue,
    /// `handle_key_event` returned - i.e. the sound has been queued.
    PlayedSound,
    /// The engine pushed the matching `UiEvent` out to the UI.
    UiEventSent,
    /// The UI poll loop wrote the new state into the Dioxus signal.
    UiWrite,
    /// A soundpack finished decoding into the engine.
    PackLoad,
    /// An output-device switch completed.
    DeviceSwitch,
}

impl Point {
    fn label(self) -> &'static str {
        match self {
            Point::WorkerSend => "worker",
            Point::EngineDequeue => "engine_recv",
            Point::PlayedSound => "sound",
            Point::UiEventSent => "ui_event",
            Point::UiWrite => "ui_write",
            Point::PackLoad => "pack_load",
            Point::DeviceSwitch => "device_switch",
        }
    }
}

/// One observation. Deliberately `Copy` and allocation-light: `key` is a
/// fixed-size inline buffer so recording never touches the allocator on the
/// engine or worker threads.
#[derive(Clone, Copy)]
struct Record {
    point: Point,
    /// Milliseconds since process start.
    at_ms: f64,
    /// How long the traced operation itself took, when the point measures a
    /// duration (`PlayedSound`, `PackLoad`, ...) rather than an instant.
    dur_ms: f64,
    key: InlineKey,
}

/// Key codes are short (`"KeyA"`, `"ControlLeft"`, `"MouseLeft"`). Storing
/// them inline keeps `Record: Copy` and avoids a `String` allocation per
/// keystroke on the hot path.
#[derive(Clone, Copy)]
struct InlineKey {
    bytes: [u8; 24],
    len: u8,
}

impl InlineKey {
    fn new(s: &str) -> Self {
        let mut bytes = [0u8; 24];
        // Truncate on a char boundary so the result is always valid UTF-8.
        let mut end = s.len().min(bytes.len());
        while end > 0 && !s.is_char_boundary(end) {
            end -= 1;
        }
        bytes[..end].copy_from_slice(&s.as_bytes()[..end]);
        Self { bytes, len: end as u8 }
    }

    fn as_str(&self) -> &str {
        // Safe: `new` only ever copies a UTF-8 prefix ending on a char
        // boundary, and `len` is set to that prefix's length.
        std::str::from_utf8(&self.bytes[..self.len as usize]).unwrap_or("?")
    }
}

/// Whether tracing is on. Cheap enough to call per keystroke.
#[inline]
pub fn enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

/// Milliseconds since process start, for callers timing their own spans.
#[inline]
pub fn now_ms() -> f64 {
    match ORIGIN.get() {
        Some(origin) => origin.elapsed().as_secs_f64() * 1000.0,
        None => 0.0,
    }
}

/// Records one trace point. A no-op unless `MECHVIBES_TRACE=1`.
///
/// `dur_ms` is the duration of the operation for points that measure a span,
/// and `0.0` for points that mark an instant.
#[inline]
pub fn record(point: Point, key: &str, dur_ms: f64) {
    if !enabled() {
        return;
    }
    if let Some(tx) = SINK.get() {
        // Unbounded send never blocks, which is the whole point: a slow or
        // wedged writer must not be able to stall the engine loop.
        let _ = tx.send(Record {
            point,
            at_ms: now_ms(),
            dur_ms,
            key: InlineKey::new(key),
        });
    }
}

/// Times `f`, records it against `point`, and returns its value. Falls through
/// to a plain call when tracing is off, so the timing itself costs nothing in
/// the shipping configuration.
#[inline]
pub fn time<T>(point: Point, key: &str, f: impl FnOnce() -> T) -> T {
    if !enabled() {
        return f();
    }
    let started = Instant::now();
    let out = f();
    record(point, key, started.elapsed().as_secs_f64() * 1000.0);
    out
}

/// Turns tracing on when `MECHVIBES_TRACE=1` is set, spawning the writer
/// thread. Call once, early in `main`, before any traced thread starts.
/// Without the env var this returns immediately and the app is unchanged.
pub fn init() {
    let on = std::env::var("MECHVIBES_TRACE").map(|v| v == "1").unwrap_or(false);
    if !on {
        return;
    }

    let _ = ORIGIN.set(Instant::now());
    let (tx, rx) = crossbeam_channel::unbounded::<Record>();
    if SINK.set(tx).is_err() {
        return; // already initialized
    }

    let path = std::env::temp_dir().join(format!("mechvibes-trace-{}.log", std::process::id()));
    eprintln!("🔬 [trace] MECHVIBES_TRACE=1 - per-keystroke timings below; detail log: {}", path.display());
    eprintln!("🔬 [trace] columns: total = worker->ui_write. A hop over {:.0}ms is marked <== SLOW", SLOW_HOP_MS);

    std::thread::spawn(move || writer_loop(rx, path));

    ENABLED.store(true, Ordering::Relaxed);
}

/// A hop at or above this is called out on the console line - well above the
/// ~15.6ms Windows timer quantum the UI poll rides on, so normal scheduling
/// jitter does not trip it.
const SLOW_HOP_MS: f64 = 50.0;

/// Accumulates the hops of a single keystroke so the console gets one summary
/// line instead of a hop per line.
struct Pending {
    key: InlineKey,
    worker_at: f64,
    engine_at: Option<f64>,
    sound_at: Option<f64>,
    sound_dur: f64,
    ui_event_at: Option<f64>,
}

impl Pending {
    /// Formats the finished keystroke as one console line, marking whichever
    /// hop dominated if any crossed the slow threshold.
    fn summarize(&self, ui_write_at: f64) -> String {
        let engine = self.engine_at.unwrap_or(self.worker_at);
        let sound = self.sound_at.unwrap_or(engine);
        let ui_event = self.ui_event_at.unwrap_or(sound);

        let hops = [
            ("worker->engine", engine - self.worker_at),
            ("handle_key", self.sound_dur.max(sound - engine)),
            ("->ui_event", ui_event - sound),
            ("->ui_write", ui_write_at - ui_event),
        ];
        let total = ui_write_at - self.worker_at;

        let mut line = format!("🔬 TRACE key={:<12} total={:>8.1}ms", self.key.as_str(), total);
        for (name, ms) in hops {
            line.push_str(&format!("  {}={:.1}ms", name, ms));
        }
        if let Some((name, ms)) = hops.iter().copied().find(|(_, ms)| *ms >= SLOW_HOP_MS) {
            line.push_str(&format!("   <== SLOW: {} took {:.0}ms", name, ms));
        }
        line
    }
}

/// Owns all trace I/O. Runs on its own thread so no traced path ever formats
/// or writes anything itself.
fn writer_loop(rx: crossbeam_channel::Receiver<Record>, path: std::path::PathBuf) {
    use std::io::Write;

    let mut file = std::fs::File::create(&path).ok();
    // At most one keystroke is in flight per key code; a small vec beats a map
    // here because it is only ever a handful of entries.
    let mut pending: Vec<Pending> = Vec::new();

    while let Ok(rec) = rx.recv() {
        if let Some(f) = file.as_mut() {
            let _ = writeln!(
                f,
                "{:>12.3}\t{}\t{}\t{:.3}",
                rec.at_ms,
                rec.point.label(),
                rec.key.as_str(),
                rec.dur_ms
            );
        }

        let key = rec.key.as_str().to_string();
        let find = |p: &Pending| p.key.as_str() == key;

        match rec.point {
            Point::WorkerSend => {
                // A repeat before the previous one finished means the earlier
                // keystroke never reached the UI; drop it rather than pair
                // hops from two different presses.
                pending.retain(|p| !find(p));
                pending.push(Pending {
                    key: rec.key,
                    worker_at: rec.at_ms,
                    engine_at: None,
                    sound_at: None,
                    sound_dur: 0.0,
                    ui_event_at: None,
                });
            }
            Point::EngineDequeue => {
                if let Some(p) = pending.iter_mut().find(|p| find(p)) {
                    p.engine_at = Some(rec.at_ms);
                }
            }
            Point::PlayedSound => {
                if let Some(p) = pending.iter_mut().find(|p| find(p)) {
                    p.sound_at = Some(rec.at_ms);
                    p.sound_dur = rec.dur_ms;
                }
            }
            Point::UiEventSent => {
                if let Some(p) = pending.iter_mut().find(|p| find(p)) {
                    p.ui_event_at = Some(rec.at_ms);
                }
            }
            Point::UiWrite => {
                if let Some(idx) = pending.iter().position(find) {
                    let done = pending.remove(idx);
                    let line = done.summarize(rec.at_ms);
                    eprintln!("{}", line);
                    // Tee into the Debug section's buffer when the user asked
                    // for verbose. `push_verbose` masks the key identity, so
                    // the timing is captured and the key never is. Done here,
                    // on the writer thread, so no traced path pays for it.
                    crate::utils::log_buffer::push_verbose(&line);
                }
            }
            // Points that stand alone rather than belonging to a keystroke.
            Point::PackLoad => {
                let line = format!(
                    "🔬 TRACE pack_load {} took {:.0}ms",
                    rec.key.as_str(),
                    rec.dur_ms
                );
                eprintln!("{}", line);
                crate::utils::log_buffer::push_verbose(&line);
            }
            Point::DeviceSwitch => {
                let line = format!(
                    "🔬 TRACE device_switch {} took {:.0}ms",
                    rec.key.as_str(),
                    rec.dur_ms
                );
                eprintln!("{}", line);
                crate::utils::log_buffer::push_verbose(&line);
            }
        }

        // A keystroke whose UI write never arrives would otherwise sit here
        // forever; cap the backlog so this thread's memory stays bounded.
        if pending.len() > 64 {
            pending.drain(..32);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inline_key_roundtrips_normal_codes() {
        for code in ["KeyA", "ControlLeft", "MouseLeft", ""] {
            assert_eq!(InlineKey::new(code).as_str(), code);
        }
    }

    #[test]
    fn inline_key_truncates_without_splitting_a_char() {
        // A multi-byte char straddling the buffer end must not produce
        // invalid UTF-8 - `as_str` would otherwise fall back to "?" and the
        // trace would lose the key identity.
        let long = "é".repeat(40);
        let stored = InlineKey::new(&long);
        assert!(stored.as_str().len() <= 24);
        assert!(long.starts_with(stored.as_str()), "must be a clean prefix");
        assert_ne!(stored.as_str(), "?");
    }

    #[test]
    fn tracing_is_off_until_initialized() {
        // The shipping default: every trace point must be inert, so the hot
        // paths carry no cost for code that only exists to diagnose.
        assert!(!enabled());
        record(Point::WorkerSend, "KeyA", 0.0);
        assert_eq!(time(Point::PlayedSound, "KeyA", || 7), 7);
    }

    #[test]
    fn summary_marks_the_hop_that_stalled() {
        // The console line is the deliverable, so the slow hop has to be
        // named rather than left for the reader to spot among four numbers.
        let pending = Pending {
            key: InlineKey::new("KeyA"),
            worker_at: 0.0,
            engine_at: Some(1.0),
            sound_at: Some(801.0),
            sound_dur: 0.0,
            ui_event_at: Some(801.5),
        };
        let line = pending.summarize(803.0);
        assert!(line.contains("key=KeyA"), "{}", line);
        assert!(line.contains("total=   803.0ms"), "{}", line);
        assert!(line.contains("<== SLOW: handle_key"), "{}", line);
    }

    #[test]
    fn a_clean_keystroke_is_not_flagged() {
        let pending = Pending {
            key: InlineKey::new("KeyB"),
            worker_at: 100.0,
            engine_at: Some(100.1),
            sound_at: Some(100.4),
            sound_dur: 0.3,
            ui_event_at: Some(100.5),
        };
        let line = pending.summarize(109.0);
        assert!(!line.contains("SLOW"), "{}", line);
    }
}
