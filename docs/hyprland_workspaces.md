# hyprland_workspaces Module

Module for working with Hyprland workspaces, clients, and desktop files.

## Description

The module provides functionality for retrieving information about Hyprland workspaces, their clients, icons, and application desktop files.

## Main Types

### HyprWorkspaces

Main structure containing information about all workspaces.

```rust
pub struct HyprWorkspaces {
    pub map: HyprWorkspacesMap,      // HashMap workspace_id -> HyprWorkspace
    pub active_id: Option<i32>,      // Active workspace ID
}
```

**Methods:**
- `init() -> Result<Self>` - initializes the structure by fetching data from Hyprland API

### HyprWorkspace

Workspace information.

```rust
pub struct HyprWorkspace {
    pub id: i32,                      // Workspace ID
    pub monitor: String,              // Monitor name
    pub monitor_id: Option<i128>,     // Monitor ID
    pub clients: Vec<HyprlandClient>, // Client list
}
```

### HyprlandClient

Information about a client (window) in a workspace.

```rust
pub struct HyprlandClient {
    pub class: String,                // Application class
    pub title: String,                // Current window title
    pub initial_title: String,        // Initial window title
    pub workspace_id: i32,            // Workspace ID the client is bound to
    pub icons: Vec<String>,          // Icon paths
    pub desktop_file: Option<DesktopFile>, // Application desktop file
}
```

### DesktopFile

Information from application .desktop file.

```rust
pub struct DesktopFile {
    pub name: String,     // Application name
    pub exec: String,     // Launch command
    pub icon: Option<String>, // Icon name
}
```

**Methods:**
- `load(app_class_name: &str) -> Result<Option<Self>>` - loads desktop file by application class name

## Usage

```rust
use hyprland_workspaces::HyprWorkspaces;

// Initialization
let workspaces = HyprWorkspaces::init()?;

// Get active workspace
if let Some(active_id) = workspaces.active_id {
    if let Some(workspace) = workspaces.map.get(&active_id) {
        println!("Active workspace: {}", workspace);
        println!("Clients: {}", workspace.clients.len());
    }
}

// Iterate over all workspaces
for (id, workspace) in &workspaces.map {
    println!("Workspace {}: {} clients", id, workspace.clients.len());
}
```

## Features

1. **Desktop file caching** - desktop files are cached for fast access
2. **Automatic icon search** - the module automatically searches for application icons via `helpers::icon_fetcher`
3. **Error handling** - the module uses `anyhow::Result` for error handling

## Desktop File Search

The module searches for desktop files in the following directories:
- `/usr/share/applications`
- `/usr/local/share/applications`
- `~/.local/share/applications`

Search is performed by application class name (exact match and lowercase variant).

## Dependencies

- `hyprland` - for Hyprland API integration
- `ini` - for parsing .desktop files
- `helpers` - for icon search
- `logger` - for logging
- `anyhow` - for error handling
- `once_cell` - for lazy cache initialization
