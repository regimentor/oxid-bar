# bar Module

Main GTK4 status bar application for Wayland environments.

## Description

`bar` is the main application of the OxidBar project that displays system information as a status bar. The application uses GTK4 to create the interface and integrates with various system services.

## Structure

```
bar/
├── src/
│   ├── main.rs          # Entry point, GTK4 application initialization
│   ├── app.rs           # Main application logic (BarApp)
│   ├── config.rs        # Application configuration
│   ├── services/        # Services
│   │   └── hyprland.rs  # Hyprland API integration
│   └── ui/              # UI components
│       ├── components/  # UI components
│       │   ├── clock.rs      # Clock component
│       │   ├── lang.rs       # Keyboard layout component
│       │   ├── tray.rs       # System tray component
│       │   └── workspaces.rs # Workspaces component
│       ├── styles.rs     # Styles
│       └── window.rs    # Application window
└── resources/
    └── styles.css       # CSS styles
```

## Main Components

### BarApp

Main application structure that manages the lifecycle and UI.

```rust
pub struct BarApp {
    config: BarConfig,
}
```

**Methods:**
- `new() -> Self` - creates a new application with default configuration
- `build_ui(&self, app: &Application)` - builds the application UI

### BarConfig

Application configuration with customizable parameters.

```rust
pub struct BarConfig {
    pub height: i32,                        // Bar height in pixels
    pub workspaces_check_interval_ms: u64, // Workspaces check interval
    pub lang_update_interval_ms: u64,       // Keyboard layout update interval
    pub clock_update_interval_ms: u64,      // Clock update interval
    pub tray_update_interval_secs: u64,    // Tray update interval
    pub icon_size: i32,                     // Icon size
    pub spacing: i32,                       // Spacing between elements
}
```

**Default values:**
- `height`: 32px
- `workspaces_check_interval_ms`: 100ms
- `lang_update_interval_ms`: 200ms
- `clock_update_interval_ms`: 1000ms
- `tray_update_interval_secs`: 1s
- `icon_size`: 20px
- `spacing`: 12px

## UI Components

### WorkspacesComponent

Displays active Hyprland workspaces with clients and their icons.

### ClockComponent

Displays current time with customizable format.

### LangComponent

Displays current keyboard layout with country flag.

### TrayComponent

Displays system tray icons (StatusNotifier).

## Services

### start_hyprland_event_listener

Starts a Hyprland event listener in a separate thread. Handles the following events:
- Workspace change
- Workspace add/delete
- Workspace move/rename
- Window open/close

## Usage

```bash
# Run application
cargo run -p bar
```

The application automatically:
1. Initializes logging
2. Creates GTK4 Application
3. Sets up window and UI components
4. Starts event listeners for data updates

## Dependencies

- `gtk4` - UI framework
- `glib` - for timers and async operations
- `hyprland` - for Hyprland API integration
- `tray` - system tray module
- `hyprland_workspaces` - workspace management module
- `time` - time handling module
- `lang` - keyboard layout module
- `logger` - logging module
