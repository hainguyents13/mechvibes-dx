# MechVibesDX

A modern, feature-rich mechanical keyboard sound simulator built with Rust and Dioxus. Experience the satisfying sounds of mechanical keyboards and mouse clicks with customizable soundpacks, themes, and advanced audio controls.

![MechVibesDX](assets/header.svg)

## ✨ Features

### 🎵 Audio Simulation

-   **Real-time keyboard sound simulation** - Every keystroke triggers authentic mechanical keyboard sounds
-   **Mouse button sound effects** - Complete audio feedback for all mouse interactions
-   **Dual soundpack system** - Independent soundpack selection for keyboard and mouse
-   **Volume controls** - Separate volume adjustment for keyboard and mouse sounds
-   **High-quality audio** - Support for OGG, WAV, and MP3 audio formats

### 🎨 Modern Interface

-   **Beautiful UI** - Clean, modern interface built with DaisyUI and Tailwind CSS
-   **Light/Dark themes** - Seamless theme switching with system preference detection
-   **Custom CSS editor** - Advanced styling customization with live preview

### ⚡ System Integration

-   **Global hotkey support** - Toggle sounds anywhere with `Ctrl+Alt+M`
-   **System notifications** - Visual feedback for sound toggle state
-   **Auto-start functionality** - Launch with Windows startup
-   **System tray integration** - Minimize to tray for background operation
-   **Low resource usage** - Efficient Rust-based architecture

### 🔧 Customization

-   **Multiple soundpacks** - Choose from various mechanical keyboard and mouse sound profiles
-   **Easy soundpack creation** - Simple JSON-based configuration system
-   **Custom audio timing** - Precise control over sound segments and timing
-   **Extensible architecture** - Plugin-ready design for future enhancements

## 🚀 Installation

### Pre-built Releases

1. Download the latest release from the [Releases page](https://github.com/hainguyents13/mechvibes-dx/releases)
2. Extract the archive to your desired location
3. Run `mechvibes-dx.exe`

### Building from Source

```bash
# Clone the repository
git clone https://github.com/hainguyents13/mechvibes-dx.git
cd mechvibes-dx

# Install dependencies
cargo build --release

# Run the application
cargo run --release
```

## 🎮 Usage

### Basic Operation

1. **Launch the application** - Run `mechvibes-dx.exe` or use `cargo run`
2. **Select soundpacks** - Choose different soundpacks for keyboard and mouse from the dropdowns
3. **Adjust volume** - Use the volume sliders to control keyboard and mouse sound levels
4. **Toggle sounds** - Press `Ctrl+Alt+M` globally to enable/disable sounds
5. **Customize appearance** - Switch themes or use the custom CSS editor

### Keyboard Shortcuts

-   `Ctrl+Alt+M` - Toggle sound effects on/off (global hotkey)

### Settings

Access the Settings page to configure:

-   **Auto-start** - Launch MechVibesDX with Windows
-   **System tray** - Minimize to system tray
-   **Default volumes** - Set preferred volume levels
-   **Theme preferences** - Choose default theme

## 🎵 Soundpack System

### Understanding Soundpacks

MechVibesDX uses a flexible soundpack system that maps keyboard keys and mouse buttons to specific segments of audio files. Each soundpack contains:

-   **Audio file** - The source audio containing all sounds
-   **Configuration** - JSON mapping of keys/buttons to audio segments
-   **Icon** - Visual representation of the soundpack
-   **Metadata** - Name, description, and other properties

### Mouse Button Mapping

Mouse soundpacks use the following button codes:

| Button Code    | Description                              |
| -------------- | ---------------------------------------- |
| `MouseLeft`    | Left mouse button                        |
| `MouseRight`   | Right mouse button                       |
| `MouseMiddle`  | Middle mouse button (scroll wheel click) |
| `MouseBack`    | Back button (mouse button 4)             |
| `MouseForward` | Forward button (mouse button 5)          |

### Included Soundpacks

MechVibesDX comes with several high-quality soundpacks:

-   **Cherry MX Black PBT** - Deep, satisfying mechanical keyboard sounds
-   **Oreo** - Unique, clicky mechanical switch sounds
-   **Test Mouse** - Sample mouse click sounds for demonstration

## 🛠️ Creating Custom Soundpacks

### Directory Structure

```
soundpacks/
└── my-custom-soundpack/
    ├── config.json        # Configuration file
    ├── sound.ogg          # Audio file
    └── icon.png           # Soundpack icon (optional)
```

### Keyboard Soundpack Configuration

Create a `config.json` file for keyboard soundpacks:

```json
{
    "name": "My Custom Keyboard",
    "sound": "sound.ogg",
    "defs": {
        "keya": [0, 150],
        "keys": [150, 300],
        "keyd": [300, 450],
        "keyf": [450, 600],
        "keyspace": [600, 800],
        "keyenter": [800, 1000],
        "keybackspace": [1000, 1200],
        "keyshift": [1200, 1400]
    }
}
```

### Mouse Soundpack Configuration

Create a `config.json` file for mouse soundpacks:

```json
{
    "name": "My Custom Mouse",
    "sound": "sound.ogg",
    "mouse": true,
    "defs": {
        "MouseLeft": [0, 200],
        "MouseRight": [200, 400],
        "MouseMiddle": [400, 600],
        "MouseBack": [600, 800],
        "MouseForward": [800, 1000]
    }
}
```

### Configuration Properties

| Property | Type    | Description                                       |
| -------- | ------- | ------------------------------------------------- |
| `name`   | string  | Display name of the soundpack                     |
| `sound`  | string  | Audio file name (relative to soundpack directory) |
| `mouse`  | boolean | Set to `true` for mouse soundpacks (optional)     |
| `defs`   | object  | Mapping of keys/buttons to audio segments         |

### Audio Timing Format

The `defs` object maps keys to time ranges in milliseconds:

```json
"keya": [start_time_ms, end_time_ms]
```

For example, `[0, 150]` means the sound for the "A" key starts at 0ms and ends at 150ms in the audio file.

### Supported Key Codes

#### Standard Letter Keys

`keya`, `keyb`, `keyc`, ..., `keyz`

#### Number Keys

`key0`, `key1`, `key2`, ..., `key9`

#### Special Keys

-   `keyspace` - Spacebar
-   `keyenter` - Enter key
-   `keybackspace` - Backspace
-   `keytab` - Tab key
-   `keyshift` - Shift key
-   `keyctrl` - Control key
-   `keyalt` - Alt key
-   `keyescape` - Escape key
-   `keydelete` - Delete key

#### Function Keys

`keyf1`, `keyf2`, `keyf3`, ..., `keyf12`

#### Arrow Keys

`keyup`, `keydown`, `keyleft`, `keyright`

### Testing Your Soundpack

1. **Create the directory** - Add your soundpack folder to `soundpacks/`
2. **Restart the application** - MechVibesDX will automatically detect new soundpacks
3. **Select your soundpack** - Choose it from the dropdown menu
4. **Test the sounds** - Type or click to hear your custom sounds

### Best Practices

-   **Audio quality** - Use high-quality audio files (OGG recommended for best compression)
-   **Consistent timing** - Keep sound segments roughly the same length for consistency
-   **Appropriate volume** - Normalize audio levels to prevent volume jumps
-   **Clear naming** - Use descriptive names for easy identification
-   **Icon design** - Create recognizable icons at 64x64 pixels or higher

## 🎨 Customization

### Theme System

MechVibesDX supports extensive theming:

-   **Built-in themes** - Light and dark themes with automatic system detection
-   **Custom CSS** - Write custom CSS for complete visual control
-   **Live preview** - See changes in real-time as you edit
-   **Theme persistence** - Your preferences are saved automatically

### Custom CSS Editor

Access the Customize page to:

1. **Edit CSS** - Modify appearance with custom CSS rules
2. **Preview changes** - See updates instantly
3. **Reset to defaults** - Restore original styling
4. **Save preferences** - Automatically persist your customizations

Example custom CSS:

```css
/* Change primary accent color */
:root {
    --primary: #ff6b6b;
}

/* Custom button styling */
.btn-primary {
    background: linear-gradient(45deg, #ff6b6b, #4ecdc4);
    border: none;
}

/* Modify soundpack selector */
.soundpack-selector {
    border-radius: 12px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}
```

## 🏗️ Development

### Prerequisites

-   **Rust** - Latest stable version
-   **Dioxus CLI** - For development and building

### Development Setup

```bash
# Clone the repository
git clone https://github.com/hainguyents13/mechvibes-dx.git
cd mechvibes-dx

# Install Dioxus CLI
cargo install dioxus-cli

# Run in development mode
dx serve
```

### Project Structure

```
src/
├── main.rs                 # Application entry point
├── components/             # UI components
│   ├── app_info.rs        # Application information
│   ├── dock.rs            # Navigation dock
│   ├── header.rs          # Header component
│   ├── soundpack_selector.rs # Soundpack selection
│   ├── volume_slider.rs   # Volume controls
│   └── pages/             # Page components
├── libs/                  # Core libraries
│   ├── audio/             # Audio processing
│   └── input_listener.rs  # Input capture
└── state/                 # Application state
    ├── config.rs          # Configuration management
    ├── soundpack.rs       # Soundpack handling
    └── keyboard.rs        # Keyboard state
```

### Building for Release

```bash
# Build optimized release
dx build --release

# The executable will be in dist/
```

### Contributing

1. **Fork the repository**
2. **Create a feature branch** - `git checkout -b feature/amazing-feature`
3. **Make your changes** - Follow Rust best practices
4. **Test thoroughly** - Ensure all functionality works
5. **Commit your changes** - `git commit -m 'Add amazing feature'`
6. **Push to the branch** - `git push origin feature/amazing-feature`
7. **Open a Pull Request**

### Development Guidelines

-   **Code style** - Follow `rustfmt` formatting
-   **Testing** - Add tests for new functionality
-   **Documentation** - Update documentation for API changes
-   **Performance** - Profile audio latency and memory usage
-   **Compatibility** - Test on Windows 10/11

## 🔧 Technical Details

### Architecture

MechVibesDX is built using modern technologies:

-   **Rust** - Systems programming language for performance and safety
-   **Dioxus** - Modern React-like UI framework for Rust
-   **CPAL** - Cross-platform audio library for low-latency audio
-   **Desktop app architecture** - Native desktop app with web UI technologies
-   **DaisyUI** - Component library built on Tailwind CSS

### Performance

-   **Low latency** - Optimized audio pipeline for minimal delay
-   **Memory efficient** - Smart caching of audio segments
-   **CPU friendly** - Event-driven architecture minimizes background usage
-   **Responsive UI** - 60fps interface with smooth animations

### Audio System

-   **Real-time processing** - Immediate audio feedback
-   **Format support** - OGG, WAV, MP3 audio files
-   **Segment caching** - Pre-loaded audio segments for instant playback
-   **Volume mixing** - Independent volume controls for different sound types

## 🐛 Troubleshooting

### Common Issues

**Sounds not playing**

-   Check volume settings in the application
-   Verify soundpack selection
-   Ensure audio device is working
-   Try toggling sounds with `Ctrl+Alt+M`

**High CPU usage**

-   Check for excessive input events
-   Restart the application
-   Update to latest version

**Soundpack not loading**

-   Verify `config.json` syntax
-   Check audio file format compatibility
-   Ensure all required files are present

**Global hotkey not working**

-   Check for conflicting applications
-   Run as administrator if needed
-   Verify hotkey in settings

### Getting Help

-   **GitHub Issues** - Report bugs and request features
-   **Discussions** - Ask questions and share soundpacks
-   **Documentation** - Check this README and code comments

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

-   **Original MechVibes** - Inspiration for this modern rewrite
-   **Rust Community** - Amazing ecosystem and tools
-   **Dioxus Team** - Excellent UI framework
-   **Contributors** - Everyone who helps improve the project

---

**Made with ❤️ and Rust**
