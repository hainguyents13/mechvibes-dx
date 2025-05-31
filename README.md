# MechvibesDX

MechvibesDX is an upgraded version of Mechvibes, rewritten from scratch in Rust using Dioxus.

The codebase is now available for inspection and learning purposes. MechvibesDX provides a modern, performant implementation with improved features and stability compared to the original.

## Features

### 🎹 Keyboard Sound Simulation

-   Real-time mechanical keyboard sound effects
-   Support for various switch types and soundpacks
-   Global hotkey detection across all applications

### 🖱️ Mouse Click Sounds

-   **NEW**: Full mouse button support with dedicated sound effects
-   Supports all standard mouse buttons: Left, Right, Middle, Side buttons (Mouse4/5)
-   Mouse wheel scroll sound effects (MouseWheelUp/Down)
-   Automatic fallback to keyboard sounds for compatibility with existing soundpacks
-   Dedicated mouse soundpack support with `mouse_def` configuration

### 📦 Soundpack System

-   Compatible with existing Mechvibes soundpacks
-   Extended soundpack format supporting both keyboard and mouse definitions
-   Automatic soundpack caching and optimization
-   Hot-swappable soundpacks without restart

## Mouse Button Mapping

MechvibesDX supports the following mouse buttons:

| Button         | Code             | Description                                  |
| -------------- | ---------------- | -------------------------------------------- |
| Left Click     | `MouseLeft`      | Primary mouse button                         |
| Right Click    | `MouseRight`     | Secondary mouse button (context menu)        |
| Middle Click   | `MouseMiddle`    | Scroll wheel button                          |
| Scroll Up      | `MouseWheelUp`   | Mouse wheel scroll up                        |
| Scroll Down    | `MouseWheelDown` | Mouse wheel scroll down                      |
| Side Button 1  | `Mouse4`         | Forward/Back button (typically thumb button) |
| Side Button 2  | `Mouse5`         | Forward/Back button (typically thumb button) |
| Extra Button 1 | `Mouse6`         | Additional button (gaming mice)              |
| Extra Button 2 | `Mouse7`         | Additional button (gaming mice)              |
| Extra Button 3 | `Mouse8`         | Additional button (gaming mice)              |

## Creating Mouse-Compatible Soundpacks

To add mouse support to your soundpacks, include a `mouse_defs` section in your `config.json`:

```json
{
    "name": "My Soundpack",
    "author": "Your Name",
    "defs": {
        "Space": [
            [1000, 80],
            [1080, 80]
        ]
    },
    "mouse_defs": {
        "MouseLeft": [
            [1000, 80],
            [1080, 80]
        ],
        "MouseRight": [
            [2000, 80],
            [2080, 80]
        ],
        "MouseMiddle": [
            [3000, 80],
            [3080, 80]
        ],
        "MouseWheelUp": [
            [4000, 40],
            [4040, 40]
        ],
        "MouseWheelDown": [
            [5000, 40],
            [5040, 40]
        ],
        "Mouse4": [
            [6000, 60],
            [6060, 60]
        ],
        "Mouse5": [
            [7000, 60],
            [7060, 60]
        ],
        "Mouse6": [
            [8000, 60],
            [8060, 60]
        ],
        "Mouse7": [
            [9000, 60],
            [9060, 60]
        ],
        "Mouse8": [
            [10000, 60],
            [10060, 60]
        ]
    }
}
```

Each mouse button definition uses the same format as keyboard keys: `[start_ms, duration_ms]` for press and release sounds.

**Note:** If your soundpack doesn't include `mouse_defs`, the system will automatically create fallback mappings using keyboard sounds from the `defs` section.

## Quick Start

You can try it out right now by cloning the repo and building with Dioxus CLI:

```bash
git clone https://github.com/hainguyents13/mechvibes-dx.git
cd mechvibes-dx
dx serve
```

Please note that pull requests and issues are currently disabled while development continues.
Feel free to browse the code and follow progress!

Thank you for your support!
