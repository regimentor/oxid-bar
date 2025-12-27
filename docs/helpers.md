# helpers Module

Helper utilities for finding application icons.

## Description

The module provides functionality for finding paths to application icon files in the Linux filesystem.

## API

### icon_fetcher

Gets paths to icons by icon name or application class.

```rust
pub fn icon_fetcher(app_class_name: &str) -> Result<Vec<String>>
```

**Parameters:**
- `app_class_name` - application class name or icon name (e.g., "firefox", "firefox-browser")

**Returns:**
- `Ok(Vec<String>)` - vector of paths to found icons
- `Err` - error during search

**Example:**
```rust
use helpers::icon_fetcher;

match icon_fetcher("firefox") {
    Ok(paths) => {
        for path in paths {
            println!("Found icon: {}", path);
        }
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

## Icon Search

The module searches for icons in the following directories:

1. `/usr/share/icons` (HICOLOR) - recursive search
2. `/usr/share/pixmaps` - recursive search

Search is performed by file name (without extension) matching `app_class_name`.

## Caching

The module uses a global cache to store icon search results. This avoids repeated searches for the same application name.

Cache is implemented via `once_cell::sync::Lazy` and `Mutex<HashMap<String, Vec<String>>>`.

## Usage

```rust
use helpers::icon_fetcher;

// Search for application icons
let icon_paths = icon_fetcher("firefox")?;

// Use first found icon
if let Some(first_icon) = icon_paths.first() {
    println!("Using icon: {}", first_icon);
}
```

## Features

1. **Recursive search** - the module recursively traverses directories to find icons
2. **Caching** - search results are cached for performance
3. **Multiple results** - the function returns all found icons, allowing selection of the most appropriate one

## Performance

Thanks to caching, repeated calls to `icon_fetcher` with the same application name execute instantly. The first call may take some time as it requires filesystem traversal.

## Dependencies

- `anyhow` - for error handling
- `once_cell` - for lazy cache initialization

## Error Handling

The module uses `anyhow::Result` for error handling. Main error types:

- Directory read errors
- Filesystem access problems
- Path conversion errors
