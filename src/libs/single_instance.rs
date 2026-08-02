//! Stops a second copy of the app from running (Windows).
//!
//! Two UI processes means two sets of input listeners feeding two audio
//! engines, so every keystroke plays twice. That was true of the old rdev
//! listeners too, but the Raw Input worker makes it easier to hit: the
//! worker captures regardless of focus, so a forgotten second instance is
//! audible everywhere rather than only when that window is in the
//! background.
//!
//! Uses a named mutex rather than a lock file because Windows releases it
//! automatically when the process dies, however it dies - a lock file left
//! behind by a crash would keep the app from ever starting again.
#![cfg(target_os = "windows")]

use std::collections::hash_map::DefaultHasher;
use std::hash::{ Hash, Hasher };
use std::ptr::null_mut;

use winapi::um::errhandlingapi::GetLastError;
use winapi::um::handleapi::CloseHandle;
use winapi::um::synchapi::CreateMutexW;
use winapi::um::winnt::HANDLE;

/// Session-local (`Local\`) so two different users logged into the same
/// machine each get their own instance, which is the behavior people expect
/// from a per-user tray app. A `Global\` name would let one user's running
/// copy block another's.
const MUTEX_PREFIX: &str = r"Local\MechvibesDX-SingleInstance-";

/// `ERROR_ALREADY_EXISTS`. Defined here rather than pulled from winapi's
/// `winerror` module, which this crate does not enable as a feature; the
/// value is fixed by the Win32 ABI.
const ERROR_ALREADY_EXISTS: u32 = 183;

/// Builds a lock name unique to this executable, so only *the same build*
/// blocks itself. Two copies of the installed app still refuse to co-run,
/// which is the point; but a `dx serve` dev build and an installed release
/// live at different paths and can run side by side, which is what you want
/// while developing. A single shared name makes `dx serve` fail to start
/// whenever the installed app happens to be running, with an "already
/// running" message that points at the wrong thing entirely.
///
/// The path is hashed rather than embedded: a mutex name has a length limit
/// and treats backslashes as kernel-namespace separators, both of which a
/// raw path would trip over.
fn mutex_name() -> String {
    let exe = std::env::current_exe()
        .map(|p| p.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    let mut hasher = DefaultHasher::new();
    exe.hash(&mut hasher);
    format!("{}{:x}", MUTEX_PREFIX, hasher.finish())
}

/// Holds the instance lock for as long as it is alive. Windows frees the
/// underlying mutex on process exit even if this is never dropped (a crash,
/// a `TerminateProcess`), so a stuck lock cannot outlive the process.
pub struct InstanceGuard(HANDLE);

// The handle is only closed on drop and never used concurrently; Windows
// handles are process-wide, so moving one across threads is fine.
unsafe impl Send for InstanceGuard {}

impl Drop for InstanceGuard {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0);
        }
    }
}

/// Claims the single-instance lock, returning `None` if another copy of the
/// app already holds it - in which case the caller should exit without
/// starting anything.
///
/// The input worker must NOT call this: it is a child of an instance that
/// already holds the lock, and would always be refused.
pub fn acquire() -> Option<InstanceGuard> {
    let name: Vec<u16> = {
        use std::os::windows::ffi::OsStrExt;
        std::ffi::OsStr::new(&mutex_name()).encode_wide().chain(std::iter::once(0)).collect()
    };

    let handle = unsafe { CreateMutexW(null_mut(), 0, name.as_ptr()) };

    if handle.is_null() {
        // Can't tell whether another instance exists, so let this one run:
        // a spurious refusal to start is worse than a possible double.
        return Some(InstanceGuard(null_mut()));
    }

    // The handle is returned even when the mutex already existed, so the
    // error code is what distinguishes "we created it" from "someone else
    // owns it" - and the handle still has to be closed either way.
    if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        unsafe {
            CloseHandle(handle);
        }
        return None;
    }

    Some(InstanceGuard(handle))
}
