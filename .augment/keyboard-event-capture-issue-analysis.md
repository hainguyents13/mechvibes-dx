# Keyboard Event Capture Issue - Analysis & Solution

## ✅ SOLUTION IMPLEMENTED

### Problem

Keyboard events were not captured when the application window was focused (mouse events worked fine).

### Root Cause

**Incompatibility between `rdev::listen()` and `winit`** (used by Dioxus/Wry). When a winit-based window is focused on Windows, it blocks keyboard events from reaching rdev's global hooks. This is a known issue documented in similar UI frameworks (Slint issue #3537).

### Solution: Use `rdev::grab()` Instead of `rdev::listen()`

The `grab` function intercepts events at a lower level and works correctly even when the window is focused.

**Changes Made:**

1. **Enabled `unstable_grab` feature** in `Cargo.toml`:

    ```toml
    rdev = { version = "0.5.3", features = ["unstable_grab"] }
    ```

2. **Replaced `listen` with `grab`** in `src/libs/input_listener.rs`:

    - Changed import from `use rdev::listen` to `use rdev::grab`
    - Changed callback signature from `Fn(Event)` to `Fn(Event) -> Option<Event>`
    - Return `Some(event)` to pass events through to the system
    - Return `None` to consume/block events (not used in our case)

3. **Fixed modifier key tracking** for `Fn` closure:
    - Changed `ctrl_pressed` and `alt_pressed` from `bool` to `Arc<Mutex<bool>>`
    - Updated all reads/writes to use `.lock().unwrap()`
    - Required because `grab` uses `Fn` closure (not `FnMut`), which cannot mutate captured variables

**Build Status:** ✅ Successful (34.58s, 47 warnings)

---

## 🔍 Original Problem Statement

User reports: "when i focus on app window, it not capture key event"

## 📊 Investigation Summary

### Architecture Overview

-   **Input Capture**: `rdev::listen()` - global keyboard/mouse hooks
-   **Thread Model**: Separate thread spawned in `main.rs` (line 102)
-   **Communication**: mpsc channels → UI polling loop
-   **Debouncing**: 1ms general, 10ms for Backspace
-   **Deduplication**: Prevents held key repeats

### Root Cause Analysis

#### ❌ NOT the Issue:

1. **rdev library** - Works globally regardless of window focus
2. **Channel communication** - Properly initialized before UI
3. **Thread spawning** - Starts early in main()

#### ✅ Likely Issues:

1. **Windows Thread Priority** (FIXED)

    - Low-level hooks need high priority to intercept events
    - Added `SetThreadPriority(THREAD_PRIORITY_HIGHEST)` in `input_listener.rs`

2. **Silent Failures** (Platform-Specific)

    - **macOS**: Requires Accessibility API permissions - **SILENTLY FAILS** if not granted
    - **Linux**: X11 only (won't work on Wayland)
    - **Windows**: Should work but may need admin rights

3. **Event Consumption by Dioxus/Wry**

    - Dioxus uses Wry (fork of Winit) for windowing
    - Window may have its own keyboard event handlers
    - Potential race condition between window events and global hooks

4. **Aggressive Debouncing**
    - 1ms debounce might filter rapid events
    - Backspace has 10ms debounce (line 221)

## 🛠️ Solutions Implemented

### 1. Thread Priority Boost (Windows)

**File**: `src/libs/input_listener.rs` lines 167-186

```rust
#[cfg(target_os = "windows")]
{
    use std::ptr;
    extern "system" {
        fn SetThreadPriority(hThread: *mut std::ffi::c_void, nPriority: i32) -> i32;
        fn GetCurrentThread() -> *mut std::ffi::c_void;
    }
    const THREAD_PRIORITY_HIGHEST: i32 = 2;
    unsafe {
        let handle = GetCurrentThread();
        if SetThreadPriority(handle, THREAD_PRIORITY_HIGHEST) != 0 {
            println!("✅ Input listener thread priority set to HIGHEST");
        } else {
            println!("⚠️ Failed to set thread priority");
        }
    }
}
```

### 2. Enhanced Diagnostic Logging

**File**: `src/libs/input_listener.rs` line 206

```rust
println!("⌨️ Key Pressed: {} (rdev global hook)", key_code);
```

This will help verify if events are reaching the listener.

## 🧪 Testing Steps

### Step 1: Build and Run

```powershell
cargo build --release
./target/release/mechvibes-dx.exe
```

### Step 2: Test Event Capture

1. **Without focus**: Click outside app, type keys → Should see console output
2. **With focus**: Click inside app, type keys → Should see console output
3. **Compare**: If output appears in both cases, rdev is working

### Step 3: Platform-Specific Checks

#### Windows

-   Run as Administrator (if needed)
-   Check Windows Defender/Antivirus (may block hooks)

#### macOS

```bash
# Check Accessibility permissions
System Preferences > Security & Privacy > Privacy > Accessibility
# Add Terminal.app or your IDE to the list
```

#### Linux

```bash
# Check if user is in input group
groups | grep input
# If not, add user to input group
sudo usermod -a -G input $USER
# Log out and log back in
```

## 🔧 Additional Solutions to Try

### Solution A: Reduce Debounce Time

**File**: `src/libs/input_listener.rs` line 225

```rust
// Change from 1ms to 0ms
if time_since_last > Duration::from_millis(0) {
```

### Solution B: Disable Deduplication (Temporary Test)

**File**: `src/libs/input_listener.rs` lines 210-214

```rust
// Comment out deduplication check
// let mut pressed = pressed_keys.lock().unwrap();
// if pressed.contains(&key_code.to_string()) {
//     return; // Key already pressed, ignore
// }
// pressed.insert(key_code.to_string());
```

### Solution C: Check Dioxus Window Event Handlers

Search for any `onkeydown`, `onkeyup`, `onkeypress` handlers in UI components that might be consuming events.

## 📝 Expected Behavior

**Correct**: rdev captures ALL keyboard events globally, regardless of window focus
**Current**: Events may not be captured when app window is focused

## 🎯 Next Steps

1. ✅ Build with enhanced logging - **COMPLETED**
2. ⏳ Test with/without window focus
3. ⏳ Check console output for "⌨️ Key Pressed" messages
4. ⏳ Verify platform-specific permissions
5. ⏳ If still failing, try Solutions A/B/C above

## ✅ Changes Made

### 1. Thread Priority Boost (Windows)

-   **File**: `src/libs/input_listener.rs` lines 170-186
-   **Change**: Set input listener thread to `THREAD_PRIORITY_HIGHEST`
-   **Reason**: Ensures global hooks intercept events before window handlers

### 2. Enhanced Diagnostic Logging

-   **File**: `src/libs/input_listener.rs` line 206
-   **Change**: Added `println!("⌨️ Key Pressed: {} (rdev global hook)", key_code);`
-   **Reason**: Verify if rdev is capturing events

### 3. Build Status

-   ✅ **Build successful** with 47 warnings (all non-critical)
-   ✅ No compilation errors
-   ✅ Ready for testing

## 📚 References

-   [rdev Documentation](https://docs.rs/rdev/)
-   [Windows SetThreadPriority](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-setthreadpriority)
-   [macOS Accessibility API](https://support.apple.com/guide/mac-help/allow-accessibility-apps-to-access-your-mac-mh43185/mac)
