![image](https://github.com/user-attachments/assets/5aa36739-76c8-4a34-9a9b-7e9272927f22)

# MechvibesDX

**A fun and practical way to bring your favorite sounds anywhere!**

MechvibesDX lets you play any sound when you type or click. Use it for education, presentations, gaming, or just for fun.

## Features

-   Play sounds on every keystroke (keydown/keyup) and mouse click (press/release)
-   Works with education, business, gaming, and accessibility needs
-   Global hotkey toggle (`Ctrl+Alt+M`)
-   System tray integration (Download packs, mute, change sounds on the fly)
-   In-app soundpack catalog to browse and download official soundpacks instantly
-   Custom local soundpack support
-   Multiple themes available
-   Logo and background customizations

## Installation

1. Download from [Releases](https://github.com/hainguyents13/mechvibes-dx/releases)
2. Run installer
3. Select soundpacks
4. Enjoy the sounds or playing with Customizations

## Use cases

**Education** - Musical scales, animal sounds, language learning

**Business** - Professional typewriter sounds, meeting-friendly modes

**Gaming** - Retro arcade sounds, custom sound effects

**Accessibility** - Audio feedback for visually impaired users

## Getting Soundpacks

### 1. Browse and Download In-App (New!)
You can now download soundpacks directly from the application!
- Open the app or use the system tray menu.
- Click **"Get Packs"** or browse the list from the tray.
- Find a pack you like, click download, and it will be applied instantly.

### 2. Creating Custom Soundpacks

1. Record audio files (OGG, WAV, MP3)
2. Create config.json mapping keys to sounds
3. Drag and drop folder into app

```
Piano pack/
├── config.json
├── piano.ogg
└── icon.png
```

## Documentation

Complete architecture, design, code standards, and deployment guides are available in the [`./docs`](./docs) folder:

- [Project Overview & PDR](./docs/project-overview-pdr.md)
- [System Architecture](./docs/system-architecture.md)
- [Codebase Summary & Directory Map](./docs/codebase-summary.md)
- [Code Standards](./docs/code-standards.md)
- [Design Guidelines](./docs/design-guidelines.md)
- [Deployment & Packaging Guide](./docs/deployment-guide.md)
- [Project Roadmap](./docs/project-roadmap.md)

## Development & Testing

```bash
# Check code compilation
cargo check

# Run automated unit test suite
cargo test

# Launch app in development mode
cargo run
```

## Troubleshooting

**No sounds?** Check if muted (`Ctrl+Alt+M`), soundpack selected, system volume

**Hotkey not working?** Run as administrator / check Accessibility permissions on macOS

**Soundpack won't load?** Verify config.json syntax, supported audio formats

## License

MIT License - do whatever you want with it.

Report bugs or request features via GitHub Issues.
