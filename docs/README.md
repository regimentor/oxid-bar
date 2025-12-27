# OxidBar Documentation

Welcome to the OxidBar project documentation - a GTK4 status bar for Wayland environments.

## Documentation Structure

- [bar](bar.md) - Main status bar application
- [hyprland_workspaces](hyprland_workspaces.md) - Module for working with Hyprland workspaces
- [time](time.md) - Time utilities
- [lang](lang.md) - Keyboard layout/locale module
- [tray](tray.md) - System tray module (StatusNotifier)
- [helpers](helpers.md) - Helper utilities
- [logger](logger.md) - Logging module
- [audio](audio.md) - Audio module (PulseAudio backend)

## Quick Start

```bash
# Build all modules
cargo build

# Run main application
cargo run -p bar

# Run standalone audio application
cargo run -p audio
```

## Dependencies

The project uses the following main dependencies:
- GTK4 for UI
- Hyprland API for workspace management
- PulseAudio for audio
- zbus for DBus (StatusNotifier)
- chrono for time handling
