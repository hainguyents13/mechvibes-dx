# IOHook Keycode Mapping Completeness Report

## Overview

This document summarizes the comprehensive IOHook keycode mapping implemented in MechvibesDX's config converter. The mapping has been cross-referenced with the official libuiohook library to ensure completeness.

## Mapping Coverage

### ✅ Complete Coverage Categories

#### 1. Basic Alphanumeric Keys (0x0001-0x0039)

-   VC_ESCAPE (0x0001) → "Escape"
-   VC_1 to VC_0 (0x0002-0x000B) → "Digit1" to "Digit0"
-   VC_MINUS, VC_EQUALS (0x000C-0x000D) → "Minus", "Equal"
-   VC_BACKSPACE, VC_TAB (0x000E-0x000F) → "Backspace", "Tab"
-   VC_Q to VC_P (0x0010-0x0019) → "KeyQ" to "KeyP"
-   VC_OPEN_BRACKET, VC_CLOSE_BRACKET (0x001A-0x001B) → "BracketLeft", "BracketRight"
-   VC_ENTER, VC_CONTROL_L (0x001C-0x001D) → "Enter", "ControlLeft"
-   VC_A to VC_L (0x001E-0x0026) → "KeyA" to "KeyL"
-   VC_SEMICOLON, VC_QUOTE, VC_BACKQUOTE (0x0027-0x0029) → "Semicolon", "Quote", "Backquote"
-   VC_SHIFT_L, VC_BACK_SLASH (0x002A-0x002B) → "ShiftLeft", "Backslash"
-   VC_Z to VC_M (0x002C-0x0032) → "KeyZ" to "KeyM"
-   VC_COMMA, VC_PERIOD, VC_SLASH (0x0033-0x0035) → "Comma", "Period", "Slash"
-   VC_SHIFT_R, VC_KP_MULTIPLY (0x0036-0x0037) → "ShiftRight", "NumpadMultiply"
-   VC_ALT_L, VC_SPACE, VC_CAPS_LOCK (0x0038-0x003A) → "AltLeft", "Space", "CapsLock"

#### 2. Function Keys (0x003B-0x006B)

-   VC_F1 to VC_F10 (0x003B-0x0044) → "F1" to "F10"
-   VC_F11, VC_F12 (0x0057-0x0058) → "F11", "F12"
-   VC_F13 to VC_F15 (0x005B-0x005D) → "F13" to "F15"
-   VC_F16 to VC_F24 (0x0063-0x006B) → "F16" to "F24"

#### 3. Numeric Keypad (0x0045-0x0053)

-   VC_NUM_LOCK, VC_SCROLL_LOCK (0x0045-0x0046) → "NumLock", "ScrollLock"
-   VC_KP_7 to VC_KP_9 (0x0047-0x0049) → "Numpad7" to "Numpad9"
-   VC_KP_SUBTRACT (0x004A) → "NumpadSubtract"
-   VC_KP_4 to VC_KP_6 (0x004B-0x004D) → "Numpad4" to "Numpad6"
-   VC_KP_ADD (0x004E) → "NumpadAdd"
-   VC_KP_1 to VC_KP_3 (0x004F-0x0051) → "Numpad1" to "Numpad3"
-   VC_KP_0, VC_KP_SEPARATOR (0x0052-0x0053) → "Numpad0", "NumpadDecimal"

#### 4. Extended Keys (0x0E00-0xE0FF range)

-   VC_KP_DIVIDE (0x0E35) → "NumpadDivide"
-   VC_KP_ENTER (0x0E1C) → "NumpadEnter"
-   VC_CONTROL_R (0x0E1D) → "ControlRight"
-   VC_KP_EQUALS (0x0E0D) → "NumpadEquals"
-   VC_PRINTSCREEN (0xE037) → "PrintScreen"
-   VC_ALT_R (0x0E38) → "AltRight"
-   VC_PAUSE (0x0E45) → "Pause"
-   VC_HOME (0xE047) → "Home"
-   VC_UP (0xE048) → "ArrowUp"
-   VC_PAGE_UP (0xE049) → "PageUp"
-   VC_LEFT (0xE04B) → "ArrowLeft"
-   VC_RIGHT (0xE04D) → "ArrowRight"
-   VC_END (0xE04F) → "End"
-   VC_DOWN (0xE050) → "ArrowDown"
-   VC_PAGE_DOWN (0xE051) → "PageDown"
-   VC_INSERT (0xE052) → "Insert"
-   VC_DELETE (0xE053) → "Delete"
-   VC_META_L, VC_META_R (0xE05B-0xE05C) → "MetaLeft", "MetaRight"
-   VC_CONTEXT_MENU (0xE05D) → "ContextMenu"

#### 5. Media Control Keys (0xE000-0xE0FF range)

-   VC_MEDIA_PREVIOUS (0xE010) → "MediaTrackPrevious"
-   VC_MEDIA_NEXT (0xE019) → "MediaTrackNext"
-   VC_VOLUME_MUTE (0xE020) → "AudioVolumeMute"
-   VC_APP_CALCULATOR (0xE021) → "LaunchApp2"
-   VC_MEDIA_PLAY (0xE022) → "MediaPlayPause"
-   VC_MEDIA_STOP (0xE024) → "MediaStop"
-   VC_VOLUME_DOWN (0xE02E) → "AudioVolumeDown"
-   VC_VOLUME_UP (0xE030) → "AudioVolumeUp"
-   VC_BROWSER_HOME (0xE032) → "BrowserHome"
-   VC_APP_MUSIC (0xE03C) → "LaunchApp1"
-   VC_POWER (0xE05E) → "Power"
-   VC_SLEEP (0xE05F) → "Sleep"
-   VC_WAKE (0xE063) → "WakeUp"
-   VC_APP_PICTURES (0xE064) → "LaunchApp3"
-   VC_BROWSER_SEARCH (0xE065) → "BrowserSearch"
-   VC_BROWSER_FAVORITES (0xE066) → "BrowserFavorites"
-   VC_BROWSER_REFRESH (0xE067) → "BrowserRefresh"
-   VC_BROWSER_STOP (0xE068) → "BrowserStop"
-   VC_BROWSER_FORWARD (0xE069) → "BrowserForward"
-   VC_BROWSER_BACK (0xE06A) → "BrowserBack"
-   VC_APP_MAIL (0xE06C) → "LaunchMail"
-   VC_MEDIA_SELECT (0xE06D) → "MediaSelect"

#### 6. Japanese Language Keys (0x0070-0x007E)

-   VC_KATAKANA (0x0070) → "Convert"
-   VC_UNDERSCORE (0x0073) → "Lang1"
-   VC_FURIGANA (0x0077) → "Lang2"
-   VC_KANJI (0x0079) → "KanaMode"
-   VC_HIRAGANA (0x007B) → "HiraganaKatakana"
-   VC_YEN (0x007D) → "IntlYen"
-   VC_KP_COMMA (0x007E) → "NumpadComma"

#### 7. Sun Keyboard Extensions (0xFF75-0xFF7E)

-   VC_SUN_HELP (0xFF75) → "Help"
-   VC_SUN_PROPS (0xFF76) → "Props"
-   VC_SUN_FRONT (0xFF77) → "Front"
-   VC_SUN_STOP (0xFF78) → "Stop"
-   VC_SUN_AGAIN (0xFF79) → "Again"
-   VC_SUN_UNDO (0xFF7A) → "Undo"
-   VC_SUN_CUT (0xFF7B) → "Cut"
-   VC_SUN_COPY (0xFF7C) → "Copy"
-   VC_SUN_INSERT (0xFF7D) → "Paste"
-   VC_SUN_FIND (0xFF7E) → "Find"

### ✅ Compatibility Mappings

#### Legacy V1 Support

Additional mappings for V1 soundpack compatibility using different keycode ranges:

-   Alternative numpad mappings (3597, 3612-3677 range)
-   Alternative extended key mappings (60999-61011 range)

#### Platform-Specific Keys

-   IntlBackslash (94, 58470) - Less/Greater key on international layouts
-   Fn (95) - Function modifier key
-   Clear (96, 58444) - Clear key on some keyboards

## Implementation Details

### Source Reference

The mapping is implemented in `src/utils/config_converter.rs` in the `create_iohook_to_web_key_mapping()` function.

### Cross-Reference Sources

-   libuiohook header file: `include/uiohook.h`
-   Platform implementations: Windows, X11, Darwin input helpers
-   IOHook keycode definitions verified against GitHub repository: kwhat/libuiohook

### Key Features

1. **Complete Coverage**: All standard IOHook keycodes are mapped
2. **Extended Key Support**: Proper handling of 0xE0xx extended scancodes
3. **Multi-Platform**: Works across Windows, Linux, and macOS
4. **Legacy Compatibility**: Supports V1 config migration
5. **International Support**: Japanese and international keyboard layouts
6. **Media Key Support**: Full multimedia and browser key support

## Verification Status

✅ **COMPLETE**: The IOHook keycode mapping now includes:

-   All basic alphanumeric keys (26 letters, 10 digits, punctuation)
-   All function keys (F1-F24)
-   Complete numpad support (numbers, operators, navigation)
-   All extended navigation keys (arrows, home, end, page up/down, insert, delete)
-   All modifier keys (shift, ctrl, alt, meta/windows)
-   Complete media control keys (play, pause, stop, volume, etc.)
-   Browser navigation keys (back, forward, home, search, etc.)
-   Application launcher keys (mail, calculator, etc.)
-   Power management keys (power, sleep, wake)
-   International keyboard support (Japanese IME keys, intl symbols)
-   Sun keyboard extensions (for completeness)
-   Legacy V1 compatibility mappings

This implementation ensures that MechvibesDX can correctly handle keyboard input from any IOHook-compatible input system across all supported platforms and keyboard layouts.
