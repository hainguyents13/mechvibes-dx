# Linux Wayland Support

MechvibesDX now supports **both X11 and Wayland** display servers on Linux!

## How It Works

The application automatically detects your display server and uses the appropriate input backend:

### **X11 Mode**
- **Unfocused:** Uses `rdev` (global keyboard hooks)
- **Focused:** Uses `device_query` (polling-based)
- No special permissions required

### **Wayland Mode**
- **All states:** Uses `evdev` (direct kernel input events)
- **Requires:** User must be in the `input` group

## Setup for Wayland

### 1. Add Your User to the `input` Group

```bash
sudo usermod -a -G input $USER
```

### 2. Log Out and Log Back In

The group membership change requires a new login session to take effect.

**Verify your group membership:**
```bash
groups
```

You should see `input` in the list.

### 3. Run MechvibesDX

```bash
./mechvibes-dx
```

## Troubleshooting

### "Failed to enumerate devices" Error

**Symptom:**
```
❌ [evdev] Failed to enumerate devices: Permission denied (os error 13)
💡 [evdev] Make sure you're in the 'input' group: sudo usermod -a -G input $USER
```

**Solution:**
1. Make sure you added yourself to the `input` group (see step 1 above)
2. **Log out and log back in** (or reboot)
3. Verify with `groups` command

### "No keyboard devices found" Error

**Symptom:**
```
❌ [evdev] No keyboard devices found!
💡 [evdev] Make sure you have permission to access /dev/input/event*
```

**Solution:**
Check permissions on `/dev/input/event*`:
```bash
ls -l /dev/input/event*
```

You should see files owned by `root:input` with permissions like `crw-rw----`.

If not, check your udev rules:
```bash
cat /etc/udev/rules.d/99-input.rules
```

### Alternative: Run with sudo (Not Recommended)

If you cannot add yourself to the `input` group, you can run with sudo:
```bash
sudo ./mechvibes-dx
```

**Note:** This is not recommended for security reasons.

## Platform Comparison

| Platform | Display Server | Backend | Permissions Required |
|----------|----------------|---------|---------------------|
| **Linux** | X11 | `rdev` + `device_query` | None |
| **Linux** | Wayland | `evdev` | `input` group |
| **Windows** | - | `rdev` + `device_query` | None |
| **macOS** | - | `rdev` + `device_query` | Accessibility |

## Technical Details

### Why evdev on Wayland?

Wayland intentionally blocks global keyboard listeners for security reasons. The only way to capture keyboard events globally on Wayland is to:

1. **Use evdev** - Read input events directly from the kernel (`/dev/input/event*`)
2. **Use Wayland portals** - Only works for specific registered hotkeys, not all keys

Since MechvibesDX needs to capture all keyboard events (not just hotkeys), we use `evdev`.

### How Detection Works

The application checks the `XDG_SESSION_TYPE` environment variable:

```rust
let display_server = std::env::var("XDG_SESSION_TYPE")
    .unwrap_or_else(|_| "x11".to_string());

if display_server == "wayland" {
    // Use evdev
} else {
    // Use rdev + device_query
}
```

### Event Sources in Logs

When running, you'll see different source tags in the logs:

**X11:**
```
⌨️ Key Pressed: KeyA [source: rdev/unfocused]
⌨️ Key Pressed: KeyB [source: device_query/focused]
```

**Wayland:**
```
⌨️ Key Pressed: KeyA [source: evdev/unfocused]
⌨️ Key Pressed: KeyB [source: evdev/focused]
```

## Security Considerations

### Why does evdev require the `input` group?

The `/dev/input/event*` devices contain raw input from all keyboards and mice. Allowing unrestricted access would be a security risk (keyloggers, etc.).

By requiring `input` group membership, the system ensures:
- Only authorized users can read input events
- The user explicitly granted permission
- Audit trail exists (group membership is logged)

### Is it safe to add myself to the `input` group?

**Yes**, if you trust the applications you run. Being in the `input` group allows applications to:
- ✅ Read keyboard and mouse events
- ✅ Read touchpad/touchscreen events
- ❌ Cannot inject events (requires `uinput` group or root)

Most desktop Linux users are already in the `input` group by default on many distributions.

## FAQ

**Q: Will this work on all Wayland compositors?**
A: Yes! `evdev` works at the kernel level, independent of the compositor (GNOME, KDE, Sway, Hyprland, etc.).

**Q: Can I use MechvibesDX without adding myself to the `input` group?**
A: On Wayland, no. On X11, yes (it will use `rdev` + `device_query`).

**Q: Does this work on Wayland + XWayland apps?**
A: Yes! `evdev` captures all keyboard events regardless of whether the app is native Wayland or XWayland.

**Q: Will keyboard events work when the window is focused?**
A: Yes! Unlike X11 where we need a hybrid approach, `evdev` works in both focused and unfocused states.

