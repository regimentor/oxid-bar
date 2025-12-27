# tray Module

Module for working with system tray via StatusNotifier protocol (DBus).

## Description

The module provides functionality for getting and displaying system tray icons via StatusNotifier protocol using DBus and zbus.

## Main Types

### Tray

Main structure for working with system tray.

```rust
pub struct Tray {
    connection: zbus::Connection,
}
```

**Methods:**
- `new() -> zbus::Result<Self>` - creates a new DBus connection (async)
- `connection(&self) -> &zbus::Connection` - returns a reference to DBus connection
- `get_items(&self) -> zbus::Result<Vec<TrayItem>>` - gets list of all tray items (async)
- `get_item_menu(&self, item: &TrayItem) -> zbus::Result<Option<MenuNode>>` - gets tray item menu (async)

### TrayItem

System tray item.

```rust
pub struct TrayItem {
    pub id: String,                    // Unique item ID
    pub title: String,                  // Item title
    pub status: TrayItemStatus,         // Item status
    pub category: String,               // Item category
    pub icon: TrayIcon,                // Item icon
    pub tooltip: ToolTip,              // Tooltip
    pub menu_path: Option<OwnedObjectPath>, // Menu path
    pub is_menu: bool,                 // Whether item is a menu
    pub window_id: u32,                // Window ID (if any)
    pub bus_name: String,              // DBus service name
    pub object_path: String,           // DBus object path
}
```

### TrayItemStatus

Tray item status.

```rust
pub enum TrayItemStatus {
    Passive,        // Passive
    Active,         // Active
    NeedsAttention, // Needs attention
}
```

### TrayIcon

Tray item icon information.

```rust
pub struct TrayIcon {
    pub name: Option<String>,              // Icon name
    pub pixmap: Option<(i32, i32, Vec<u8>)>, // Icon pixmap (width, height, data)
    pub attention_name: Option<String>,    // Attention icon name
    pub overlay_name: Option<String>,     // Overlay icon name
    pub icon_paths: Vec<String>,          // Icon file paths
}
```

### ToolTip

Tray item tooltip.

```rust
pub struct ToolTip {
    icon_name: String,
    icon_pixmap: Vec<(i32, i32, Vec<u8>)>,
    title: String,
    description: String,
}
```

**Methods:**
- `new(...)` - creates a new tooltip
- `icon_name(&self) -> &str` - returns icon name
- `icon_pixmap(&self) -> &IconPixmap` - returns icon pixmap
- `title(&self) -> &str` - returns title
- `description(&self) -> &str` - returns description

### MenuNode

Tray item menu node.

```rust
pub struct MenuNode {
    pub id: i32,                          // Node ID
    pub props: HashMap<String, OwnedValue>, // Node properties
    pub children: Vec<MenuNode>,          // Child nodes
}
```

## Usage

### Basic Usage

```rust
use tray::Tray;

// Create Tray instance
let tray = Tray::new().await?;

// Get all tray items
let items = tray.get_items().await?;

for item in items {
    println!("Item: {} - {}", item.id, item.title);
    println!("Status: {:?}", item.status);
    
    // Get item menu (if available)
    if let Ok(Some(menu)) = tray.get_item_menu(&item).await {
        println!("Menu has {} children", menu.children.len());
    }
}
```

### Working with Icons

```rust
use tray::Tray;

let tray = Tray::new().await?;
let items = tray.get_items().await?;

for item in items {
    let icon = &item.icon;
    
    // Using pixmap icon
    if let Some((width, height, data)) = &icon.pixmap {
        println!("Pixmap: {}x{}", width, height);
    }
    
    // Using icon file paths
    for path in &icon.icon_paths {
        println!("Icon path: {}", path);
    }
    
    // Using icon name
    if let Some(name) = &icon.name {
        println!("Icon name: {}", name);
    }
}
```

### Handling Statuses

```rust
use tray::{Tray, TrayItemStatus};

let tray = Tray::new().await?;
let items = tray.get_items().await?;

for item in items {
    match item.status {
        TrayItemStatus::Passive => {
            // Normal display
        }
        TrayItemStatus::Active => {
            // Active display
        }
        TrayItemStatus::NeedsAttention => {
            // Use attention_icon
            if let Some(name) = &item.icon.attention_name {
                println!("Attention icon: {}", name);
            }
        }
    }
}
```

## Features

1. **Automatic icon search** - the module automatically searches for icon file paths via `helpers::icon_fetcher`
2. **Icon priority** - the following priority is used:
   - `icon_pixmap` property
   - `tooltip.icon_pixmap`
   - icon files by name
3. **Error handling** - the module handles missing properties and continues operation
4. **Caching** - icons are cached via `helpers::icon_fetcher`

## DBus Interfaces

The module uses the following DBus interfaces:

- `org.kde.StatusNotifierWatcher` - for getting list of registered items
- `org.kde.StatusNotifierItem` - for getting tray item properties
- `com.canonical.dbusmenu` - for working with item menus

## Dependencies

- `zbus` - for DBus integration
- `zvariant` - for DBus types
- `serde` - for serialization/deserialization
- `helpers` - for icon search
- `logger` - for logging

## Error Handling

The module uses `zbus::Result` for DBus error handling. Missing properties are handled gracefully without interrupting application operation.
